extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/c/setup_tap_device.c")
        .compile("libsetup_tap_device.a");
}
