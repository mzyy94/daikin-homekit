use crate::response::DaikinResponse;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

pub struct DaikinInfo {
    name: Option<String>,
    mac: Option<String>,
    version: Option<String>,
    edid: Option<String>,
}

impl DaikinInfo {
    pub fn new(
        name: Option<String>,
        mac: Option<String>,
        version: Option<String>,
        edid: Option<String>,
    ) -> DaikinInfo {
        DaikinInfo {
            name,
            mac,
            version,
            edid,
        }
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn mac(&self) -> Option<String> {
        self.mac.clone()
    }

    pub fn version(&self) -> Option<String> {
        self.version.clone().map(|v| v.replace('_', "."))
    }

    pub fn edid(&self) -> Option<u64> {
        self.edid.clone().and_then(|s| {
            let mut bytes = vec![0u8; 8];
            match hex::decode_to_slice(s, &mut bytes as &mut [u8]) {
                Ok(_) => {}
                Err(_) => return None,
            };
            let mut rdr = Cursor::new(bytes);

            rdr.read_u64::<BigEndian>().ok()
        })
    }
}

impl From<DaikinResponse> for DaikinInfo {
    fn from(res: DaikinResponse) -> Self {
        DaikinInfo {
            name: get_prop!(res."/dsiot/edge.adp_d".name .to_string()),
            mac: get_prop!(res."/dsiot/edge.adp_i".mac .to_string()),
            version: get_prop!(res."/dsiot/edge.adp_i".ver .to_string()),
            edid: get_prop!(res."/dsiot/edge.adp_i".edid .to_string()),
        }
    }
}

impl std::fmt::Debug for DaikinInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DaikinInfo")
            .field("name", &self.name())
            .field("mac", &self.mac())
            .field("version", &self.version())
            .field("edid", &self.edid())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let res: DaikinResponse =
            serde_json::from_str(include_str!("./fixtures/info.json")).expect("Invalid JSON file.");
        let info: DaikinInfo = res.into();

        assert_eq!(info.name(), Some("display_name".into()));
        assert_eq!(info.mac(), Some("00005E005342".into()));
        assert_eq!(info.version(), Some("2.7.0".into()));
        assert_eq!(info.edid(), Some(19088743));
    }

    #[test]
    fn debug_display() {
        let res: DaikinResponse =
            serde_json::from_str(include_str!("./fixtures/info.json")).expect("Invalid JSON file.");
        let info: DaikinInfo = res.into();

        assert_eq!(
            format!("{:?}", info),
            r#"DaikinInfo { name: Some("display_name"), mac: Some("00005E005342"), version: Some("2.7.0"), edid: Some(19088743) }"#
        );
    }
}
