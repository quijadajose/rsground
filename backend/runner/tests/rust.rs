const HELLO_WORLD_RS: &str = r#"
fn main() {
    print!("Hello World");
}
"#;

#[test]
fn rust_compilation() {
    let runner = rsground_runner::Runner::new().expect("The runners was not created");

    runner.create_file("main.rs", HELLO_WORLD_RS).unwrap();

    let output = runner.run_rustc(["main.rs"]).expect("Cannot run code");

    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));

    assert_eq!(output.status.success(), true);
    assert_eq!(output.stdout, "".as_bytes().to_vec());
}

#[test]
fn rust_executable() {
    let runner = rsground_runner::Runner::new().expect("The runners was not created");

    runner.create_file("main.rs", HELLO_WORLD_RS).unwrap();

    let output = runner.run_rustc(["main.rs"]).expect("Cannot run code");

    eprintln!("-- COMPILATION --");
    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));

    assert_eq!(output.status.success(), true);

    let output = runner
        .run(
            "/bin/patchelf",
            [
                "--set-interpreter",
                "/lib/ld-linux-x86-64.so.2",
                "/home/main",
            ],
        )
        .expect("Cannot run code");

    eprintln!("-- PATCHING --");
    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));

    assert_eq!(output.status.success(), true);

    let output = runner
        .run("/home/main", [] as [&str; 0])
        .expect("Cannot run code");

    eprintln!("-- EXECUTABLE --");
    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));

    assert_eq!(output.status.success(), true);
    assert_eq!(output.stdout, "Hello World".as_bytes().to_vec());
}
