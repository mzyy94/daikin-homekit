use crate::response::{Property, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DaikinInfo {
    pub responses: Vec<Response>,
}

impl DaikinInfo {
    fn find_property(&self, name: &str) -> Option<&Property> {
        self.responses.iter().find(|&r| r.fr == name).map(|r| &r.pc)
    }

    pub fn get(&self, name: &str, path: &str) -> Option<&Property> {
        let root = self.find_property(name);
        root.and_then(|prop| prop.search(path))
    }

    pub fn name(&self) -> String {
        self.get("/dsiot/edge.adp_d", "name")
            .map_or("".to_string(), |p| p.get_string().unwrap_or("".to_string()))
    }

    pub fn mac(&self) -> String {
        self.get("/dsiot/edge.adp_i", "mac")
            .map_or("".to_string(), |p| p.get_string().unwrap_or("".to_string()))
    }

    pub fn version(&self) -> String {
        self.get("/dsiot/edge.adp_i", "ver")
            .map_or("".to_string(), |p| p.get_string().unwrap_or("".to_string()))
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
