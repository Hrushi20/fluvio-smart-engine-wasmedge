pub(crate) mod filter;

pub(crate) use instance::create_transform;

mod instance {

    use anyhow::{Result};
    use wasmedge_sdk::*;

    use crate::{
        instance::{SmartModuleInstanceContext, DowncastableTransform},
        error::EngineError,
        SmartModuleInitialData,
    };

    use super::{
        filter::SmartModuleFilter,
    };

    pub(crate) fn create_transform(
        ctx: &SmartModuleInstanceContext,
        _initial_data: SmartModuleInitialData,
        store: &mut Store,
    ) -> Result<Box<dyn DowncastableTransform>> {
        if let Some(tr) = SmartModuleFilter::try_instantiate(ctx, store)?
            .map(|transform| Box::new(transform) as Box<dyn DowncastableTransform>)
        {
            Ok(tr)
        } else {
            Err(EngineError::UnknownSmartModule.into())
        }
    }
}
