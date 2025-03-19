use hakoniwa::Output;

pub fn print_output(output: &Output) {
    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));
}
