use std::convert::TryFrom;
use std::fmt::Debug;

use wasmedge_sdk::{Store, WasmValue};
use anyhow::{Result, Ok};
use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleInitInput, SmartModuleInitOutput, SmartModuleInitErrorStatus,
};
use wasmedge_sdk::Func;

use crate::{instance::SmartModuleInstanceContext, SmartEngine};

pub(crate) const INIT_FN_NAME: &str = "init";
pub(crate) struct SmartModuleInit(Func);

impl Debug for SmartModuleInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitFn")
    }
}


impl SmartModuleInit {
    /// Try to create filter by matching function, if function is not found, then return empty
    pub fn try_instantiate(
        ctx: &SmartModuleInstanceContext,
        _store: &mut Store,
    ) -> Result<Option<Self>> {
        // ctx.get_wasm_func(name)
        match ctx.get_wasm_func(INIT_FN_NAME) {
            Some(func) => Ok(Some(Self(func))),
            None => Ok(None)
        }
    }
}

impl SmartModuleInit {
    /// initialize SmartModule
    pub(crate) fn initialize(
        &mut self,
        input: SmartModuleInitInput,
        ctx: &mut SmartModuleInstanceContext,
        store: &mut Store,
        engine: &mut SmartEngine
    ) -> Result<()> {
        let slice = ctx.write_input(&input, &mut *store,engine)?;
        println!("Before init_output");
        let init_output = self.0.call(&mut engine.executor, vec![WasmValue::from_i32(slice.0),WasmValue::from_i32(slice.1),WasmValue::from_i32(slice.2 as i32)])?;
        let init_output = init_output[0].to_i32();
        println!("Afterinit_output: {}",init_output);
        if init_output < 0 {
            let internal_error = SmartModuleInitErrorStatus::try_from(init_output)
                .unwrap_or(SmartModuleInitErrorStatus::UnknownError);

            // match internal_error {
            //     SmartModuleInitErrorStatus::InitError => {
            //         let output: SmartModuleInitOutput = ctx.read_output(store)?;
            //         Err(output.error.into())
            //     }
            //     _ => Err(internal_error.into()),
            // }
            Err(internal_error.into())
        } else {
            Ok(())
        }
    }
}
