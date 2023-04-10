use crate::response::{Property, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DaikinStatus {
    responses: Vec<Response>,
}

impl DaikinStatus {
    fn find_property(&self, name: &str) -> Option<&Property> {
        self.responses.iter().find(|&r| r.fr == name).map(|r| &r.pc)
    }

    pub fn get(&self, name: &str, path: &str) -> Option<&Property> {
        let root = self.find_property(name);
        root.and_then(|prop| prop.search(path))
    }

    pub fn power(&self) -> bool {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_A002/p_01")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
            != 0.0
    }

    pub fn current_temperature(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_A00B/p_01")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn current_humidity(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_A00B/p_02")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn current_outside_temperature(&self) -> f64 {
        self.get("/dsiot/edge/adr_0200.dgc_status", "e_1003/e_A00D/p_01")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn mode(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_3001/p_01")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn target_cooling_temperature(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_3001/p_02")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn target_heating_temperature(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_3001/p_03")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }

    pub fn target_automatic_temperature(&self) -> f64 {
        self.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_3001/p_1F")
            .map_or(0.0, |p| p.get_f64().unwrap_or(0.0))
    }
}

impl std::fmt::Debug for DaikinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DaikinStatus")
            .field("power", &self.power())
            .field("current_temperature", &self.current_temperature())
            .field("current_humidity", &self.current_humidity())
            .field(
                "current_outside_temperature",
                &self.current_outside_temperature(),
            )
            .field("mode", &self.mode())
            .field(
                "target_cooling_temperature",
                &self.target_cooling_temperature(),
            )
            .field(
                "target_heating_temperature",
                &self.target_heating_temperature(),
            )
            .field(
                "target_automatic_temperature",
                &self.target_automatic_temperature(),
            )
            .finish()
    }
}
