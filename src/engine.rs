use wasmedge_sdk::{ Executor, Module, config::ConfigBuilder, Store };
use wasmedge_sys::Config;
use std::fmt::{self, Debug};

use anyhow::Result;
use derive_builder::Builder;
use tracing::debug;

use fluvio_smartmodule::dataplane::smartmodule::{
    SmartModuleExtraParams, SmartModuleInput, SmartModuleOutput,
};

use crate::{transforms::{self, create_transform}, instance::{SmartModuleInstanceContext, SmartModuleInstance}};

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

            let transform = create_transform(&ctx, config.initial_data, &mut store)?;
            let mut instance = SmartModuleInstance::new(ctx, transform);
            // instance.init(&mut store)?;
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