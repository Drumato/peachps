pub type RawMacAddress = [u8; 6];

#[derive(Debug)]
pub struct MacAddress {
    pub addr: RawMacAddress,
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr_str = self
            .addr
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(":");
        write!(f, "{}", addr_str)
    }
}

#[cfg(test)]
mod mac_address_tests {
    use super::*;

    #[test]
    fn display_address_test() {
        let addr = MacAddress {
            addr: [12, 34, 56, 78, 90, 12],
        };

        assert_eq!("0c:22:38:4e:5a:0c", addr.to_string());
    }
}
