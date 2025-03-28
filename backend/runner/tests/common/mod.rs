#![allow(
    dead_code,
    reason = "Common is used in many test, there are dead_code for some of them"
)]

pub mod cargo;

use hakoniwa::Output;

pub const HELLO_WORLD_RS: &str = r#"
fn main() {
    print!("Hello World");
}
"#;

pub fn print_output(output: &Output) {
    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));
}
