use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use daikin_client::{Daikin, ReqwestClient};
use dsiot::DaikinStatus;
use dsiot::protocol::DaikinInfo;

#[derive(Clone)]
pub struct Device {
    dk: Daikin<ReqwestClient>,
    rt: tokio::runtime::Handle,
    reachable: Arc<AtomicBool>,
}

impl Device {
    pub fn new(dk: Daikin<ReqwestClient>, rt: tokio::runtime::Handle) -> Self {
        Self {
            dk,
            rt,
            reachable: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn get_status(&self) -> anyhow::Result<DaikinStatus> {
        let result = self.rt.block_on(self.dk.get_status());
        self.reachable.store(result.is_ok(), Ordering::Relaxed);
        result
    }

    pub fn update(&self, status: DaikinStatus) -> anyhow::Result<()> {
        let result = self.rt.block_on(self.dk.update(status));
        self.reachable.store(result.is_ok(), Ordering::Relaxed);
        result
    }

    pub fn get_info(&self) -> anyhow::Result<DaikinInfo> {
        self.rt.block_on(self.dk.get_info())
    }

    pub fn is_reachable(&self) -> bool {
        self.reachable.load(Ordering::Relaxed)
    }
}
