use crate::network_device::tap_device::*;
use crate::network_device::NetworkDeviceError;
use std::ffi::CString;

#[link(name = "setup_c")]
extern "C" {
    // void _setup_tap_dev(char *device_path, struct TapDevice *tap_device);
    fn _setup_tap_dev(
        device_path: *const libc::c_char,
        tap_device: *mut RawTapDevice,
    ) -> libc::c_int;
}

#[allow(dead_code)]
/// TAPデバイスを設定する
/// TAPデバイスのパスと，パケットを受信するインタフェースの名前を使用する
pub fn setup_tap_device(device_path: String) -> Result<TapDevice, NetworkDeviceError> {
    unsafe {
        let mut tap_device: RawTapDevice = std::mem::MaybeUninit::uninit().assume_init();
        let device_path = match CString::new(device_path) {
            Ok(s) => s,
            Err(_e) => {
                return Err(NetworkDeviceError::MalformedDeviceName);
            }
        };

        let ret_v = _setup_tap_dev(device_path.as_ptr(), &mut tap_device);
        if ret_v == -1 {
            return Err(NetworkDeviceError::FailedToSetupNetworkDevice);
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
