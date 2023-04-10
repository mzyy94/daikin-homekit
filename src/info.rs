use crate::response::Response;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DaikinInfo {
    pub responses: Vec<Response>,
}

impl DaikinInfo {
    pub fn name(&self) -> String {
        get_prop!(self."/dsiot/edge.adp_d".name -> str).unwrap_or("".to_string())
    }

    pub fn mac(&self) -> String {
        get_prop!(self."/dsiot/edge.adp_i".mac -> str).unwrap_or("".to_string())
    }

    pub fn version(&self) -> String {
        get_prop!(self."/dsiot/edge.adp_i".ver -> str).unwrap_or("".to_string())
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
