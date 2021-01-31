mod device;
pub use device::*;

mod raw_socket;
pub use raw_socket::*;

#[link(name = "setup_c")]
extern "C" {
    fn _setup_raw_sock(
        interface_name: *const libc::c_char,
        raw_sock: *mut RawSocket,
    ) -> libc::c_int;
}
