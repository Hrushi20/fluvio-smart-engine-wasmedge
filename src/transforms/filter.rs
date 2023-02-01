use std::convert::TryFrom;
use std::fmt::Debug;

use anyhow::Result;
use wasmedge_sdk::*;
// use wasmtime::{AsContextMut, TypedFunc};

use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleInput, SmartModuleOutput, SmartModuleTransformErrorStatus,
};

use crate::{
    instance::{SmartModuleInstanceContext, SmartModuleTransform}, SmartEngine,
};

const FILTER_FN_NAME: &str = "filter";

pub(crate) struct SmartModuleFilter(Func);

impl Debug for SmartModuleFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FilterFn")
    }
}

impl SmartModuleFilter {
    /// Try to create filter by matching function, if function is not found, then return empty
    pub fn try_instantiate(
        ctx: &SmartModuleInstanceContext,
        _store: &mut Store 
    ) -> Result<Option<Self>> {
        match ctx.get_wasm_func(FILTER_FN_NAME) {
            Some(func) => Ok(Some(Self(func))),
            None => Ok(None)
        }
    }
}

impl SmartModuleTransform for SmartModuleFilter {
    fn process(
        &mut self,
        input: SmartModuleInput,
        ctx: &mut SmartModuleInstanceContext,
        store: &mut Store,
        engine: &mut SmartEngine
    ) -> Result<SmartModuleOutput> {

        let slice = ctx.write_input(&input, &mut *store,engine)?;
        println!("Slice: {:?}",slice);
        println!("Before Filter_output");
        let filter_output = self.0.call(&mut engine.executor, vec![WasmValue::from_i32(slice.0),WasmValue::from_i32(slice.1),WasmValue::from_i32(slice.2 as i32)])?;
        println!("After Filter_output");

        let filter_output = filter_output[0].to_i32();

        if filter_output < 0 {
            let internal_error = SmartModuleTransformErrorStatus::try_from(filter_output)
                .unwrap_or(SmartModuleTransformErrorStatus::UnknownError);
            return Err(internal_error.into());
        }

        Ok(SmartModuleOutput::default())
        // let output: SmartModuleOutput = ctx.read_output(store)?;
        // Ok(output)
    }

    fn name(&self) -> &str {
        FILTER_FN_NAME
    }
}

#[cfg(test)]
mod test {

    use wasmedge_sdk::{config::ConfigBuilder, Executor};



    use std::{convert::TryFrom};

    use fluvio_smartmodule::{
        dataplane::smartmodule::{SmartModuleInput},
        Record,
    };

    use crate::{
        SmartEngine, SmartModuleChainBuilder, SmartModuleConfig, metrics::SmartModuleChainMetrics 
    };

    const SM_FILTER: &str = "fluvio_smartmodule_filter";
    const SM_FILTER_INIT: &str = "fluvio_smartmodule_filter_init";

    use crate::fixture::read_wasm_module;

    #[ignore]
    #[test]
    fn test_filter() {
        let config = ConfigBuilder::default().build().unwrap();
        let executor = Executor::new(Some(&config),None).unwrap();
        let mut engine = SmartEngine {
            executor 
        };

        let mut chain_builder = SmartModuleChainBuilder::default();

        chain_builder.add_smart_module(
            SmartModuleConfig::builder().build().unwrap(),
            read_wasm_module(SM_FILTER),
        );

        let mut chain = chain_builder
            .initialize(&mut engine)
            .expect("failed to build chain");

        assert_eq!(
            chain.instances().first().expect("first").transform().name(),
            super::FILTER_FN_NAME
        );

        let metrics = SmartModuleChainMetrics::default();
        let input = vec![Record::new("hello world")];
        let output = chain
            .process(SmartModuleInput::try_from(input).expect("input"), &metrics,&mut engine)
            .expect("process");
        // assert_eq!(output.successes.len(), 0); // no records passed

        // let input = vec![Record::new("apple"), Record::new("fruit")];
        // let output = chain
        //     .process(SmartModuleInput::try_from(input).expect("input"), &metrics)
        //     .expect("process");
        // assert_eq!(output.successes.len(), 1); // one record passed
        // assert_eq!(output.successes[0].value.as_ref(), b"apple");
    }

    #[ignore]
    #[test]
    fn test_filter_with_init_invalid_param() {
        let config = ConfigBuilder::default().build().unwrap();
        let executor = Executor::new(Some(&config),None).unwrap();
        let mut engine = SmartEngine {
            executor 
        };

        let mut chain_builder = SmartModuleChainBuilder::default();

        chain_builder.add_smart_module(
            SmartModuleConfig::builder().build().unwrap(),
            read_wasm_module(SM_FILTER_INIT),
        );
        assert_eq!(
            chain_builder
                .initialize(&mut engine)
                .expect_err("should return param error")
                .to_string(),
            "Missing param key\n\nSmartModule Init Error: \n"
        );
    }

    #[ignore]
    #[test]
    fn test_filter_with_init_ok() {

        let config = ConfigBuilder::default().build().unwrap();
        let executor = Executor::new(Some(&config),None).unwrap();
        let mut engine = SmartEngine {
            executor 
        };

        let mut chain_builder = SmartModuleChainBuilder::default();

        chain_builder.add_smart_module(
            SmartModuleConfig::builder()
                .param("key", "a")
                .build()
                .unwrap(),
            read_wasm_module(SM_FILTER_INIT),
        );

        let mut chain = chain_builder
            .initialize(&mut engine)
            .expect("failed to build chain");
        
        let instance = chain.instances().first().expect("first");


        assert_eq!(
            instance.transform().name(),
            super::FILTER_FN_NAME
        );

        assert!(instance.get_init().is_some());

        let metrics = SmartModuleChainMetrics::default();

        let input = vec![Record::new("hello world")];
        let output = chain
            .process(SmartModuleInput::try_from(input).expect("input"), &metrics,&mut engine)
            .expect("process");
        // assert_eq!(output.successes.len(), 0); // no records passed

        // let input = vec![
        //     Record::new("apple"),
        //     Record::new("fruit"),
        //     Record::new("banana"),
        // ];

        // let output = chain
        //     .process(SmartModuleInput::try_from(input).expect("input"), &metrics)
        //     .expect("process");
        // assert_eq!(output.successes.len(), 2); // one record passed
        // assert_eq!(output.successes[0].value.as_ref(), b"apple");
        // assert_eq!(output.successes[1].value.as_ref(), b"banana");

        // // build 2nd chain with different parameter
        // let mut chain_builder = SmartModuleChainBuilder::default();
        // chain_builder.add_smart_module(
        //     SmartModuleConfig::builder()
        //         .param("key", "b")
        //         .build()
        //         .unwrap(),
        //     read_wasm_module(SM_FILTER_INIT),
        // );

        // let mut chain = chain_builder
        //     .initialize(&engine)
        //     .expect("failed to build chain");

        // let input = vec![
        //     Record::new("apple"),
        //     Record::new("fruit"),
        //     Record::new("banana"),
        // ];
        // let output = chain
        //     .process(SmartModuleInput::try_from(input).expect("input"), &metrics)
        //     .expect("process");
        // assert_eq!(output.successes.len(), 1); // only banana
        // assert_eq!(output.successes[0].value.as_ref(), b"banana");
    }
}
