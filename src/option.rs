use yaml_rust::YamlLoader;

use crate::{internet, link};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeachPSOption {
    pub dev_addr: link::MacAddress,
    pub ip_addr: internet::ip::IPv4Addr,
    pub network_mask: internet::ip::IPv4Addr,
    pub debug: bool,
}

impl Default for PeachPSOption {
    fn default() -> Self {
        Self {
            dev_addr: Default::default(),
            ip_addr: Default::default(),
            network_mask: Default::default(),
            debug: false,
        }
    }
}

impl PeachPSOption {
    pub fn from_yaml(yaml_path: &str) -> PeachPSOption {
        let y = std::fs::read_to_string(yaml_path).unwrap();
        let yaml = YamlLoader::load_from_str(&y).unwrap();
        let yaml = &yaml[0];

        PeachPSOption {
            dev_addr: link::MacAddress::from(yaml["device_addr"].as_str().unwrap()),
            ip_addr: internet::ip::IPv4Addr::from(yaml["ip_addr"].as_str().unwrap()),
            network_mask: internet::ip::IPv4Addr::from(yaml["network_mask"].as_str().unwrap()),
            debug: yaml["debug"].as_bool().unwrap(),
        }
    }
}
