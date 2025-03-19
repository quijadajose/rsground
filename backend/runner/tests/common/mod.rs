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
