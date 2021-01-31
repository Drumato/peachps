use crate::network_device::*;

use std::ffi::CString;

#[allow(dead_code)]
pub fn setup_raw_socket(interface_name: String) -> Result<Socket, NetworkDeviceError> {
    unsafe {
        let mut raw_sock: RawSocket = std::mem::MaybeUninit::uninit().assume_init();

        let interface_name = match CString::new(interface_name) {
            Ok(s) => s,
            Err(_e) => {
                return Err(NetworkDeviceError::MalformedInterfaceName);
            }
        };

        let ret_v = _setup_raw_sock(interface_name.as_ptr(), &mut raw_sock);
        if ret_v == -1 {
            return Err(NetworkDeviceError::FailedToSetupNetworkDevice);
        }

        Ok(Socket::from_raw(raw_sock.fd, raw_sock.mac_addr))
    }
}

#[cfg(test)]
mod operation_tests {
    use super::*;

    #[test]
    #[ignore]
    fn setup_raw_socket_test() {
        // sudo が必要な操作の為ignoreしておく
        let result = setup_raw_socket("eth0".to_string());
        assert!(result.is_ok());
    }
}
