use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    IP,
    ARP,
    IPv6,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::IP => "IP",
                Type::ARP => "ARP",
                Type::IPv6 => "IPv6",
            }
        )
    }
}

impl From<u16> for Type {
    fn from(v: u16) -> Self {
        match v {
            0x0800 => Type::IP,
            0x0806 => Type::ARP,
            0x86dd => Type::IPv6,
            _ => unimplemented!(),
        }
    }
}
