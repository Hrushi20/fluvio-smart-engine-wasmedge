use std::any::Any;
use std::sync::{Arc, Mutex};
use std::fmt::{self, Debug};

use fluvio_protocol::{Encoder, Decoder};
use wasmedge_sdk::{Module,Store,Instance,CallingFrame,WasmValue,error::HostFuncError,Memory,Caller,ImportObjectBuilder,Func};

use tracing::{debug};
use anyhow::{Error, Result};
use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleExtraParams, SmartModuleInput, SmartModuleOutput, SmartModuleInitInput,
};

use crate::error::EngineError;
use crate::init::SmartModuleInit;
// use crate::init::SmartModuleInit;
use crate::{WasmSlice, memory, SmartEngine};
// use crate::state::WasmState;


pub(crate) struct SmartModuleInstance {
    ctx: SmartModuleInstanceContext,
    init: Option<SmartModuleInit>,
    transform: Box<dyn DowncastableTransform>,
}


impl SmartModuleInstance {
    #[cfg(test)]
    #[allow(clippy::borrowed_box)]
    pub(crate) fn transform(&self) -> &Box<dyn DowncastableTransform> {
        &self.transform
    }

    #[cfg(test)]
    pub(crate) fn get_init(&self) -> &Option<SmartModuleInit> {
        &self.init
    }

    pub(crate) fn new(
        ctx: SmartModuleInstanceContext,
        init: Option<SmartModuleInit>,
        transform: Box<dyn DowncastableTransform>,
    ) -> Self {
        Self {
            ctx,
            init,
            transform,
        }
    }

    pub(crate) fn process(
        &mut self,
        input: SmartModuleInput,
        store: &mut Store,
        engine: &mut SmartEngine
    ) -> Result<SmartModuleOutput> {
        self.transform.process(input, &mut self.ctx, store,engine)
    }

    pub fn init(&mut self, store: &mut Store,engine:&mut SmartEngine) -> Result<(), Error> {
        if let Some(init) = &mut self.init {
            let input = SmartModuleInitInput {
                params: self.ctx.params.clone(),
            };
            init.initialize(input, &mut self.ctx, store,engine)
        } else {
            Ok(())
        }
    }

}

pub(crate) struct SmartModuleInstanceContext {
    instance: Instance,
    records_cb: Arc<RecordsCallBack>,
    params: SmartModuleExtraParams,
    version: i16,
}

impl Debug for SmartModuleInstanceContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SmartModuleInstanceBase")
    }
}

impl SmartModuleInstanceContext{

    #[tracing::instrument(skip(state, module, params))]
    pub(crate) fn instantiate(
        state: &mut Store,
        module: Module,
        params: SmartModuleExtraParams,
        version: i16,
        engine: & mut SmartEngine
    ) -> Result<Self, EngineError> {
        debug!("creating WasmModuleInstance");
        let cb = Arc::new(RecordsCallBack::new());
        let records_cb = cb.clone();
       
        let copy_records_fn = move |_caller: CallingFrame, inputs: Vec<WasmValue>| -> Result<Vec<WasmValue>, HostFuncError> {

            let caller = Caller::new(_caller);

            let memory = caller.instance().unwrap().memory("test").unwrap();

            let ptr = inputs[0].to_i32() as i32;
            let len = inputs[1].to_i32() as i32;

            let records = RecordsMemory { ptr, len, memory };
            cb.set(records);
            Ok(vec![])
        };

        debug!("instantiating WASMtime");
        let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), ()>("copy_records", copy_records_fn).expect("Coudn't initialize import func")
        .build("env").expect("Couldn't build import object");

        state.register_import_module(&mut engine.executor, &import).expect("Cloudn't register import module");
        let instance = state.register_named_module(&mut engine.executor, "test", &module).expect("CLouddnl't create instance");
        // let instance = state
        //     .instantiate(&module, copy_records_fn)
        //     .map_err(EngineError::Instantiate)?;
        Ok(Self {
            instance,
            records_cb,
            params,
            version,
        })
    }

    pub(crate) fn get_wasm_func(&self, name: &str) -> Option<Func> {
        println!("{:?}",self.instance.func_names());
        self.instance.func(name)
    }


    pub(crate) fn write_input<E: Encoder>(
        &mut self,
        input: &E,
        _store: &mut Store,
        engine: &mut SmartEngine
    ) -> Result<WasmSlice> {
        self.records_cb.clear();
        let mut input_data = Vec::new();
        input.encode(&mut input_data, self.version)?;
        debug!(
            len = input_data.len(),
            version = self.version,
            "input encoded"
        );
        let array_ptr = memory::copy_memory_to_instance(&self.instance, &input_data,engine)?;
        let length = input_data.len();
        println!("Array_Ptr: {}, Length: {}, Version: {}",array_ptr,length,self.version);
        Ok((array_ptr as i32, length as i32, self.version as u32))
    }

    // pub(crate) fn read_output<D: Decoder + Default>(&mut self, store: &mut Store) -> Result<D> {
    //     let bytes = self
    //         .records_cb
    //         .get()
    //         .and_then(|m| m.copy_memory_from(store).ok())
    //         .unwrap_or_default();
    //     let mut output = D::default();
    //     output.decode(&mut std::io::Cursor::new(bytes), self.version)?;
    //     Ok(output)
    // }
}

pub(crate) trait SmartModuleTransform: Send + Sync {
    /// transform records
    fn process(
        &mut self,
        input: SmartModuleInput,
        ctx: &mut SmartModuleInstanceContext,
        store: &mut Store,
        engine: &mut SmartEngine
    ) -> Result<SmartModuleOutput>;

    /// return name of transform, this is used for identifying transform and debugging
    fn name(&self) -> &str;
}

// In order turn to any, need following magic trick
pub(crate) trait DowncastableTransform: SmartModuleTransform + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: SmartModuleTransform + Any> DowncastableTransform for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct RecordsMemory {
    ptr: i32,
    len: i32,
    memory: Memory,
}

impl RecordsMemory {
    fn copy_memory_from(&self, _store: &mut Store) -> Result<Vec<u8>> {
        let bytes = self.memory.read(self.ptr as u32, self.len as u32).unwrap();
        // self.memory.read(store, self.ptr as usize, &mut bytes)?;
        Ok(bytes)
    }
}

pub struct RecordsCallBack(Mutex<Option<RecordsMemory>>);

impl RecordsCallBack {
    pub(crate) fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub(crate) fn set(&self, records: RecordsMemory) {
        let mut write_inner = self.0.lock().unwrap();
        write_inner.replace(records);
    }

    pub(crate) fn clear(&self) {
        let mut write_inner = self.0.lock().unwrap();
        write_inner.take();
    }

    // pub(crate) fn get(&self,store:&mut Store) -> Option<RecordsMemory> {
    //     let mut record = self.0.lock().unwrap();
    //     return record.clone()
    // }
}