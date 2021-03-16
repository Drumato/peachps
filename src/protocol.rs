pub mod ether;
pub mod ip;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Protocol {
    Ethernet,
    IP,
}
