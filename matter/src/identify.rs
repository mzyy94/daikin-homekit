use rs_matter::dm::clusters::decl::identify;
use rs_matter::dm::{Cluster, Dataver, InvokeContext, ReadContext, WriteContext};
use rs_matter::error::Error;

pub(crate) struct StubIdentify {
    pub(crate) dataver: Dataver,
}

impl StubIdentify {
    pub(crate) const CLUSTER: Cluster<'static> = identify::FULL_CLUSTER.with_features(0);

    pub(crate) fn new(dataver: Dataver) -> Self {
        Self { dataver }
    }
}

impl identify::ClusterHandler for StubIdentify {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn identify_time(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        Ok(0)
    }

    fn identify_type(&self, _ctx: impl ReadContext) -> Result<identify::IdentifyTypeEnum, Error> {
        Ok(identify::IdentifyTypeEnum::None)
    }

    fn set_identify_time(&self, _ctx: impl WriteContext, _value: u16) -> Result<(), Error> {
        Ok(())
    }

    fn handle_identify(
        &self,
        _ctx: impl InvokeContext,
        _req: identify::IdentifyRequest<'_>,
    ) -> Result<(), Error> {
        info!("Identify requested");
        Ok(())
    }

    fn handle_trigger_effect(
        &self,
        _ctx: impl InvokeContext,
        _req: identify::TriggerEffectRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
