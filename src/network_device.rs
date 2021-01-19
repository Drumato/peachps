mod tap_device;
pub use tap_device::*;

mod device;
pub use device::*;

mod raw_socket;
pub use raw_socket::*;

#[link(name = "setup_c")]
extern "C" {
    // void _setup_tap_dev(char *device_path, struct TapDevice *tap_device);
    fn _setup_tap_dev(
        device_path: *const libc::c_char,
        tap_device: *mut RawTapDevice,
    ) -> libc::c_int;
    fn _setup_raw_sock(
        interface_name: *const libc::c_char,
        raw_sock: *mut RawSocket,
    ) -> libc::c_int;
}
