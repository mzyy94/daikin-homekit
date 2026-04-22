use daikin_client::{Daikin, ReqwestClient};
use dsiot::DaikinStatus;

#[derive(Clone)]
pub struct Device {
    dk: Daikin<ReqwestClient>,
    rt: tokio::runtime::Handle,
}

impl Device {
    pub fn new(dk: Daikin<ReqwestClient>, rt: tokio::runtime::Handle) -> Self {
        Self { dk, rt }
    }

    pub fn get_status(&self) -> anyhow::Result<DaikinStatus> {
        self.rt.block_on(self.dk.get_status())
    }

    pub fn update(&self, status: DaikinStatus) -> anyhow::Result<()> {
        self.rt.block_on(self.dk.update(status))
    }
}
