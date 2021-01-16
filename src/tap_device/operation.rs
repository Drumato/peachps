use crate::tap_device::*;
use std::ffi::CString;

#[link(name = "setup_tap_device")]
extern "C" {
    // void setup(char *device_path, char *interface_name, struct TapDevice *tap_device);
    fn setup(
        device_path: *const libc::c_char,
        interface_name: *const libc::c_char,
        tap_device: *mut RawTapDevice,
    ) -> libc::c_int;
}

#[allow(dead_code)]
/// TAPデバイスを設定する
/// TAPデバイスのパスと，パケットを受信するインタフェースの名前を使用する
pub fn setup_tap_device(
    device_path: String,
    interface_name: String,
) -> Result<TapDevice, TapDeviceError> {
    unsafe {
        let mut tap_device: RawTapDevice = std::mem::MaybeUninit::uninit().assume_init();
        let device_path = match CString::new(device_path) {
            Ok(s) => s,
            Err(_e) => {
                return Err(TapDeviceError::MalformedDeviceName);
            }
        };
        let interface_name = match CString::new(interface_name) {
            Ok(s) => s,
            Err(_e) => {
                return Err(TapDeviceError::MalformedInterfaceName);
            }
        };
        let ret_v = setup(
            device_path.as_ptr(),
            interface_name.as_ptr(),
            &mut tap_device,
        );
        if ret_v == -1 {
            return Err(TapDeviceError::FailedToSetupTapDevice);
        }

        Ok(TapDevice::from_raw(tap_device.fd, tap_device.mac_addr))
    }
}

#[cfg(test)]
mod operation_tests {
    use super::*;

    #[test]
    #[ignore]
    fn setup_tap_device_test() {
        // sudo が必要な操作の為ignoreしておく
        let result = setup_tap_device("/dev/net/tun".to_string(), "tap0".to_string());
        assert!(result.is_ok());
    }
}
