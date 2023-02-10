use wasmedge_sdk::{config::{ConfigBuilder, StatisticsConfigOptions}, WasmValue, ImportObjectBuilder, Module, Executor,Store, Memory,CallingFrame, error::HostFuncError, Caller, Instance, Statistics};
use std::{sync::{Arc, Mutex}, vec};
use fluvio_smartmodule::{
    Record 
};


use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleInput, SmartModuleOutput
};

use fluvio_protocol::{Encoder, Decoder};
use anyhow::{Result, Error, anyhow};



fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cb = Arc::new(RecordsCallBack::new());
    let records_cb = cb.clone();

    let copy_records_fn = move |_caller: CallingFrame,
            inputs: Vec<WasmValue>|
        -> Result<Vec<WasmValue>, HostFuncError> {

        let caller = Caller::new(_caller);
        let memory = caller.memory(0).unwrap();

        let ptr = inputs[0].to_i32() as i32;
        let len = inputs[1].to_i32() as i32;

        let records = RecordsMemory { ptr, len, memory };
        cb.set(records);
        Ok(vec![])
    };

    let mut smart_module_instance_context = SmartModuleInstanceContext::new(records_cb);
    
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);

    let wasm_lib_file = &args[1];

    let bytes = read_wasm_module(&wasm_lib_file);


    let import = ImportObjectBuilder::new()
    .with_func::<(i32,i32),()>("copy_records", copy_records_fn)?
    .build("env")?;


    let stats_config = StatisticsConfigOptions::new().count_instructions(true).measure_cost(true).measure_time(true);

    //     // create an executor
    let config = ConfigBuilder::with_statistics_config(ConfigBuilder::default(),stats_config).build()?;

    let mut stats = Statistics::new().expect("Unable to init statistics struct");

    let mut executor = Executor::new(Some(&config), Some(&mut stats))?;
    let module = Module::from_bytes(None, bytes)?;

    let mut store = Store::new().unwrap();

    store.register_import_module(&mut executor, &import).unwrap();
    let instance = store.register_active_module(&mut executor, &module).unwrap();


    let fnc = instance.func("filter").unwrap();

    let input = SmartModuleInput::try_from(vec![Record::new("hello world")])?;
    let input = smart_module_instance_context.write_input(&input, &instance, &mut executor).unwrap();
    
    println!("Input func: {:?}", input);
    let output = fnc.call(&mut executor, vec![WasmValue::from_i32(input.0),WasmValue::from_i32(input.1),WasmValue::from_i32(input.2 as i32)])?;
    let output = output[0].to_i32();
    
    println!("Output: {:?}",output);
    let output = smart_module_instance_context.read_output::<SmartModuleOutput>().unwrap();

    println!("Output value: {:?}",output);
    
    Ok(())
}

use std::{
    path::{PathBuf, Path}
};

pub(crate) fn read_wasm_module(module_name: &str) -> Vec<u8> {
    let spu_dir = std::env::var("CARGO_MANIFEST_DIR").expect("target");
    let wasm_path = PathBuf::from(spu_dir)
        .join(format!(
            "{module_name}.wasm"
        ));
    read_module_from_path(wasm_path)
}

pub(crate) fn read_module_from_path(filter_path: impl AsRef<Path>) -> Vec<u8> {
    let path = filter_path.as_ref();
    std::fs::read(path).unwrap_or_else(|_| panic!("Unable to read file {}", path.display()))
}


#[derive(Clone)]
pub struct RecordsMemory {
    ptr: i32,
    len: i32,
    memory: Memory,
}

impl RecordsMemory {
    fn copy_memory_from(&self) ->anyhow::Result<Vec<u8>> {
        let bytes = self.memory.read(self.ptr as u32, self.len as u32)?;
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

    pub(crate) fn get(&self) -> Option<RecordsMemory> {
        let reader = self.0.lock().unwrap();
        reader.clone()
    }
}

const DEFAULT_SMARTENGINE_VERSION: i16 = 17;

pub(crate) struct SmartModuleInstanceContext {
    records_cb: Arc<RecordsCallBack>,
}



impl SmartModuleInstanceContext {

    pub fn new(records_cb: Arc<RecordsCallBack>) -> Self {
        Self{ records_cb }
    }

    pub(crate) fn write_input<E: Encoder>(
        &mut self,
        input: &E,
        instance: &Instance,
        engine : &mut Executor
    ) -> Result<(i32,i32,u32)> {
        self.records_cb.clear();
        let mut input_data = Vec::new();
        input.encode(&mut input_data, DEFAULT_SMARTENGINE_VERSION)?;

        let array_ptr = copy_memory_to_instance(instance, &input_data,engine)?;
        let length = input_data.len();
        println!("Array_Ptr: {}, Length: {}, Version: {}",array_ptr,length,DEFAULT_SMARTENGINE_VERSION);
        Ok((array_ptr as i32, length as i32, DEFAULT_SMARTENGINE_VERSION as u32))
    }

    pub fn read_output<D: Decoder + Default>(&mut self) -> Result<D> {
        let bytes = self
            .records_cb
            .get()
            .and_then(|m| m.copy_memory_from().ok())
            .unwrap_or_default();
        let mut output = D::default();
        output.decode(&mut std::io::Cursor::new(bytes), DEFAULT_SMARTENGINE_VERSION)?;
        Ok(output)
    }
}

const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";

pub(crate) fn copy_memory_to_instance(
    instance: &Instance,
    bytes: &[u8],
    executor : &mut Executor
) -> Result<i32, Error> {

    let mut memory = instance.memory(MEMORY).expect("Couldn't instantiate memory");
    let alloc = instance.func(ALLOC_FN).ok_or_else(|| anyhow!("Missing alloc"))?;

    let alloc_result = alloc.call(executor,vec![WasmValue::from_i32(bytes.len() as i32)])?;

    println!("Alloc Result: {:?}", alloc_result[0].to_i32());

    let guest_ptr_offset =  alloc_result[0].to_i32();

    memory.write(bytes, guest_ptr_offset as u32).expect("Coudlnt' write data to memory");
    println!("Data written at: {}",guest_ptr_offset);

    Ok(guest_ptr_offset)
}

