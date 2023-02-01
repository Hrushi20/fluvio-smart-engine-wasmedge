use wasmedge_sdk::{ Executor, Module, Store };
use std::fmt::{self, Debug};

use anyhow::Result;
use derive_builder::Builder;
use tracing::debug;

use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleExtraParams, SmartModuleInput, SmartModuleOutput,
};

use crate::{transforms::{create_transform}, instance::{SmartModuleInstanceContext, SmartModuleInstance}, init::SmartModuleInit, metrics::SmartModuleChainMetrics};

const DEFAULT_SMARTENGINE_VERSION: i16 = 17;
pub struct SmartEngine{
    pub executor: Executor,
}

#[allow(clippy::new_without_default)]
impl SmartEngine{
    // pub fn new() -> Self {
    //     let mut config = ConfigBuilder::default().build().expect("Cloudn't create a configuration");
    //     // let mut executor = Executor::new().expect("CLoudn't create an executor");
    //     Self(Executor::new(Some(&config), None).expect("Couldn't create an executor"))
    // }

}

impl Debug for SmartEngine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SmartModuleEngine")
    }
}


#[derive(Default)]
pub struct SmartModuleChainBuilder {
    smart_module: Vec<(SmartModuleConfig,Vec<u8>)>
}

impl SmartModuleChainBuilder {
    pub fn add_smart_module(&mut self, config: SmartModuleConfig,bytes:Vec<u8>){
        self.smart_module.push((config,bytes))
    }

    pub fn initialize(self,engine:&mut SmartEngine) -> Result<SmartModuleChainInstance> {
        let mut instances = Vec::with_capacity(self.smart_module.len());
        let mut store = Store::new()?;

        for (config, bytes) in self.smart_module {
            let module = Module::from_bytes(None,&bytes).expect("Couldn't create module");
            let version = config.version();

            let ctx = SmartModuleInstanceContext::instantiate(&mut store, module, config.params, version,engine).expect("Smart module context");

            let init = SmartModuleInit::try_instantiate(&ctx, &mut store)?;
            let transform = create_transform(&ctx, config.initial_data, &mut store)?;
            let mut instance = SmartModuleInstance::new(ctx,init, transform);
            instance.init(&mut store,engine)?;
            instances.push(instance);
        }
        
        Ok(SmartModuleChainInstance { store,instances })
    }
}

impl<T: Into<Vec<u8>>> From<(SmartModuleConfig, T)> for SmartModuleChainBuilder {
    fn from(pair: (SmartModuleConfig, T)) -> Self {
        let mut result = Self::default();
        result.add_smart_module(pair.0, pair.1.into());
        result
    }
}

/// SmartModule Chain Instance that can be executed
pub struct SmartModuleChainInstance {
    store: Store,
    instances: Vec<SmartModuleInstance>,
}

impl Debug for SmartModuleChainInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SmartModuleChainInstance")
    }
}

impl SmartModuleChainInstance{
    #[cfg(test)]
    pub(crate) fn instances(&self) -> &Vec<SmartModuleInstance> {
        &self.instances
    }

     /// A single record is processed thru all smartmodules in the chain.
    /// The output of one smartmodule is the input of the next smartmodule.
    /// A single record may result in multiple records.
    /// The output of the last smartmodule is added to the output of the chain.
    pub fn process(
        &mut self,
        input: SmartModuleInput,
        metric: &SmartModuleChainMetrics,
        engine: &mut SmartEngine
    ) -> Result<SmartModuleOutput> {
        let raw_len = input.raw_bytes().len();
        debug!(raw_len, "sm raw input");
        metric.add_bytes_in(raw_len as u64);

        let base_offset = input.base_offset();

        if let Some((last, instances)) = self.instances.split_last_mut() {
            let mut next_input = input;

            for instance in instances {
                // pass raw inputs to transform instance
                // each raw input may result in multiple records
                // self.store.top_up_fuel();
                let output = instance.process(next_input, &mut self.store,engine)?;
                // let fuel_used = self.store.get_used_fuel();
                // debug!(fuel_used, "fuel used");
                // metric.add_fuel_used(fuel_used);

                if output.error.is_some() {
                    // encountered error, we stop processing and return partial output
                    return Ok(output);
                } else {
                    next_input = output.successes.try_into()?;
                    next_input.set_base_offset(base_offset);
                }
            }

            // self.store.top_up_fuel();
            let output = last.process(next_input, &mut self.store,engine)?;
            // let fuel_used = self.store.get_used_fuel();
            // debug!(fuel_used, "fuel used");
            // metric.add_fuel_used(fuel_used);
            let records_out = output.successes.len();
            metric.add_records_out(records_out as u64);
            debug!(records_out, "sm records out");
            Ok(output)
        } else {
            Ok(SmartModuleOutput::new(input.try_into()?))
        }
    }
}

/// Initial seed data to passed, this will be send back as part of the output
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SmartModuleInitialData {
    None,
    Aggregate { accumulator: Vec<u8> },
}

impl SmartModuleInitialData {
    pub fn with_aggregate(accumulator: Vec<u8>) -> Self {
        Self::Aggregate { accumulator }
    }
}

impl Default for SmartModuleInitialData {
    fn default() -> Self {
        Self::None
    }
}


/// SmartModule configuration
#[derive(Builder)]
pub struct SmartModuleConfig {
    #[builder(default, setter(strip_option))]
    initial_data: SmartModuleInitialData,
    #[builder(default)]
    params: SmartModuleExtraParams,
    // this will be deprecated in the future
    #[builder(default, setter(into, strip_option))]
    version: Option<i16>,
}

impl SmartModuleConfigBuilder {
    /// add initial parameters
    pub fn param(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        let mut new = self;
        let mut params = new.params.take().unwrap_or_default();
        params.insert(key.into(), value.into());
        new.params = Some(params);
        new
    }
}


impl SmartModuleConfig {
    pub fn builder() -> SmartModuleConfigBuilder {
        SmartModuleConfigBuilder::default()
    }

    pub(crate) fn version(&self) -> i16 {
        self.version.unwrap_or(DEFAULT_SMARTENGINE_VERSION)
    }
}