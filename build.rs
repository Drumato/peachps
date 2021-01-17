extern crate cc;

fn main() {
    cc::Build::new()
        .files(&["src/c/setup_tap_device.c", "src/c/setup_raw_socket.c"])
        .compile("libsetup_c.a");
}
