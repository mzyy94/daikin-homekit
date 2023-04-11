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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let info: DaikinInfo =
            serde_json::from_str(include_str!("./fixtures/info.json")).expect("Invalid JSON file.");

        assert_eq!(info.name(), Some("display_name".into()));
        assert_eq!(info.mac(), Some("00005E005342".into()));
        assert_eq!(info.version(), Some("2_7_0".into()));
    }

    #[test]
    fn debug_display() {
        let info: DaikinInfo =
            serde_json::from_str(include_str!("./fixtures/info.json")).expect("Invalid JSON file.");

        assert_eq!(
            format!("{:?}", info),
            r#"DaikinInfo { name: Some("display_name"), mac: Some("00005E005342"), version: Some("2_7_0") }"#
        );
    }
}
