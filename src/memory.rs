use wasmedge_sdk::{*, types::Val};

use anyhow::{Result, Error, anyhow};

use crate::SmartEngine;

const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";
// const ARRAY_SUM_FN: &str = "array_sum";
// const UPPER_FN: &str = "upper";
// const DEALLOC_FN: &str = "dealloc";

/// Copy a byte array into an instance's linear memory
/// and return the offset relative to the module's memory.
pub(crate) fn copy_memory_to_instance(
    store: &mut Store,
    instance: &Instance,
    bytes: &[u8],
    engine: &mut SmartEngine
) -> Result<isize, Error> {
    // Get the "memory" export of the module.
    // If the module does not export it, just panic,
    // since we are not going to be able to copy the data.
    let mut memory = instance
        .memory(MEMORY)
        .ok_or_else(|| anyhow!("Missing memory"))?;

    // The module is not using any bindgen libraries,
    // so it should export its own alloc function.
    //
    // Get the guest's exported alloc function, and call it with the
    // length of the byte array we are trying to copy.
    // The result is an offset relative to the module's linear memory,
    // which is used to copy the bytes into the module's memory.
    // Then, return the offset.
    let alloc = instance.func(ALLOC_FN).ok_or_else(|| anyhow!("Missing alloc"))?;

    let alloc_result = alloc.call(&mut engine.executor,vec![WasmValue::from_i32(bytes.len() as i32)])?;

    println!("Alloc Result: {:?}", alloc_result[0].to_i32());

    // Check the size of the guest_ptr_offset (Could be i32 or i64)
    let guest_ptr_offset = alloc_result[0].to_i32();
    // let guest_ptr_offset = match alloc_result
    //     .get(0)
    //     .ok_or_else(|| anyhow!("missing alloc"))?
    // {
    //     Val::I32(val) => *val as isize,
    //     _ => return Err(Error::msg("guest pointer must be Val::I32")),
    // };
    // unsafe {
        // let raw = memory.data_pointer_mut(guest_ptr_offset as u32, bytes.len() as u32)?;
        // let raw = memory.data_ptr(store).offset(guest_ptr_offset);
        memory.write(bytes, guest_ptr_offset as u32).expect("Coudlnt' write data to memory");
    // }
    let str = memory.read_string(guest_ptr_offset as u32, bytes.len() as u32)?;

    println!("String written in memory is : {}",str);

    Ok(guest_ptr_offset as isize)
}
