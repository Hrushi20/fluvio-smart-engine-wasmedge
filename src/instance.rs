use std::any::Any;
use std::sync::{Arc, Mutex};
use std::fmt::{self, Debug};

use wasmedge_sdk::{ImportObject, ImportObjectBuilder, Func};
use wasmedge_sdk::{Module,Store,Instance,CallingFrame,WasmValue,error::HostFuncError};
use wasmedge_sys::{Memory};

use tracing::{debug};
use anyhow::{Error, Result};
use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleExtraParams, SmartModuleInput, SmartModuleOutput, SmartModuleInitInput,
};

use crate::error::EngineError;
// use crate::init::SmartModuleInit;
use crate::{WasmSlice, memory, SmartEngine};
// use crate::state::WasmState;


pub(crate) struct SmartModuleInstance {
    ctx: SmartModuleInstanceContext,
    transform: Box<dyn DowncastableTransform>,
}


impl SmartModuleInstance {
    #[cfg(test)]
    #[allow(clippy::borrowed_box)]
    pub(crate) fn transform(&self) -> &Box<dyn DowncastableTransform> {
        &self.transform
    }

    pub(crate) fn new(
        ctx: SmartModuleInstanceContext,
        transform: Box<dyn DowncastableTransform>,
    ) -> Self {
        Self {
            ctx,
            transform,
        }
    }

    pub(crate) fn process(
        &mut self,
        input: SmartModuleInput,
        store: &mut Store,
    ) -> Result<SmartModuleOutput> {
        self.transform.process(input, &mut self.ctx, store)
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

            let memory = _caller.memory_mut(0).unwrap();

            let ptr = inputs[0].to_i32() as i32;
            let len = inputs[1].to_i32() as i32;

            let records = RecordsMemory { ptr, len, memory };
            cb.set(records);
            Ok(vec![])
        };

        debug!("instantiating WASMtime");
        let import = ImportObjectBuilder::new()
        .with_func::<((i32), (i32)), ()>("copy_records", copy_records_fn).expect("Coudn't initialize import func")
        .build("env").expect("Couldn't build import object");

        state.register_import_module(&mut engine.executor, &import);
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

    pub(crate) fn get_wasm_func(&self, store: &mut Store, name: &str) -> Option<Func> {
        self.instance.func(name)
    }
}

pub(crate) trait SmartModuleTransform: Send + Sync {
    /// transform records
    fn process(
        &mut self,
        input: SmartModuleInput,
        ctx: &mut SmartModuleInstanceContext,
        store: &mut Store,
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
}