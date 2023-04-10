use crate::response::Response;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DaikinInfo {
    pub responses: Vec<Response>,
}

impl DaikinInfo {
    pub fn name(&self) -> Option<String> {
        get_prop!(self."/dsiot/edge.adp_d".name -> str)
    }

    pub fn mac(&self) -> Option<String> {
        get_prop!(self."/dsiot/edge.adp_i".mac -> str)
    }

    pub fn version(&self) -> Option<String> {
        get_prop!(self."/dsiot/edge.adp_i".ver -> str)
    }
}

impl std::fmt::Debug for DaikinInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DaikinInfo")
            .field("name", &self.name())
            .field("mac", &self.mac())
            .field("version", &self.version())
            .finish()
    }
}
