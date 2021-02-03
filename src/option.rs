use std::collections::HashSet;

use yaml_rust::YamlLoader;

use crate::{internet, link, transport};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeachPSOption {
    pub dev_addr: link::MacAddress,
    pub ip_addr: internet::ip::IPv4Addr,
    pub network_mask: internet::ip::IPv4Addr,
    pub debug: bool,
    pub internet_filter: HashSet<internet::InternetProtocol>,
    pub transport_filter: HashSet<transport::TransportProtocol>,
}

impl Default for PeachPSOption {
    fn default() -> Self {
        Self {
            dev_addr: Default::default(),
            ip_addr: Default::default(),
            network_mask: Default::default(),
            debug: false,
            internet_filter: Default::default(),
            transport_filter: Default::default(),
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
            internet_filter: {
                let mut s: HashSet<internet::InternetProtocol> = Default::default();
                let ips = yaml["internet"].clone().into_vec().unwrap();
                for ip in ips.iter() {
                    s.insert(internet::InternetProtocol::from(ip.as_str().unwrap()));
                }
                s
            },
            transport_filter: {
                let mut s: HashSet<transport::TransportProtocol> = Default::default();
                let tps = yaml["transport"].clone().into_vec().unwrap();
                for tp in tps.iter() {
                    s.insert(transport::TransportProtocol::from(tp.as_str().unwrap()));
                }
                s
            },
        }
    }
}
