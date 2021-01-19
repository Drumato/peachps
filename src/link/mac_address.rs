pub type RawMacAddress = [u8; 6];
pub const BLOADCAST_MAC_ADDRESS: MacAddress = MacAddress([0xff; 6]);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MacAddress(pub RawMacAddress);

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr_str = self
            .0
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(":");

        write!(f, "{}", addr_str)
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn display_address_test() {
        let addr = MacAddress([12, 34, 56, 78, 90, 12]);

        assert_eq!("0c:22:38:4e:5a:0c", addr.to_string());
    }
}
