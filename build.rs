extern crate cc;

fn main() {
    cc::Build::new()
        .files(&["src/c/setup.c"])
        .compile("libsetup_c.a");
}
