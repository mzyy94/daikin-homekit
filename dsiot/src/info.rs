use crate::response::DaikinResponse;
use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Deserializer, de};
use std::io::Cursor;

#[derive(Deserialize, Debug, Clone)]
pub struct DaikinInfo {
    pub name: String,
    pub mac: String,
    #[serde(rename = "ver", deserialize_with = "parse_version")]
    pub version: String,
    #[serde(deserialize_with = "parse_edid")]
    pub edid: u64,
}

fn parse_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(s.replace('_', "."))
}

fn parse_edid<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let s = String::deserialize(deserializer)?;
    str2edid(&s).ok_or_else(|| de::Error::custom("Invalid EDID format"))
}

fn str2edid(edid: &str) -> Option<u64> {
    let mut bytes = vec![0u8; 8];
    match hex::decode_to_slice(edid, &mut bytes as &mut [u8]) {
        Ok(_) => {}
        Err(_) => return None,
    };
    let mut rdr = Cursor::new(bytes);

    rdr.read_u64::<BigEndian>().ok()
}

impl From<DaikinResponse> for DaikinInfo {
    fn from(res: DaikinResponse) -> Self {
        DaikinInfo {
            name: get_prop!(res."/dsiot/edge.adp_d".name .to_string()).unwrap_or_default(),
            mac: get_prop!(res."/dsiot/edge.adp_i".mac .to_string()).unwrap_or_default(),
            version: get_prop!(res."/dsiot/edge.adp_i".ver .to_string())
                .unwrap_or_default()
                .replace('_', "."),
            edid: str2edid(
                &get_prop!(res."/dsiot/edge.adp_i".edid .to_string()).unwrap_or_default(),
            )
            .unwrap_or(0),
        }
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

        assert_eq!(info.name, "display_name");
        assert_eq!(info.mac, "00005E005342");
        assert_eq!(info.version, "2.7.0");
        assert_eq!(info.edid, 19088743);
    }

    #[test]
    fn serde() {
        let text = "ret=OK,type=GPF,cdev=RA,protocol=DGC,reg=jp,ver=2_7_0,rev=aabbcc00,comm_err=0,lpw_flag=0,adp_kind=4,mac=00005E005342,ssid=DaikinAP12345,adp_mode=ap_run,method=polling,name=%64%69%73%70%6c%61%79%5f%6e%61%6d%65,icon=23,edid=0000000001234567,sw_id=1900294D,api_ver=2_2";
        let info: DaikinInfo = serde_qs::from_str(&text.replace(',', "&")).unwrap();

        assert_eq!(info.name, "display_name");
        assert_eq!(info.mac, "00005E005342");
        assert_eq!(info.version, "2.7.0");
        assert_eq!(info.edid, 19088743);
    }
}
