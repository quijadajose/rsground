#[test]
fn rust() {
    let runner = rsground_runner::Runner::new().expect("The runners was not created");

    runner
        .create_file(
            "main.rs",
            r#"
            fn main() {
                println!("Hello World");
            }
            "#,
        )
        .unwrap();

    let output = runner
        // .run("/bin/ls", ["-l", "/"])
        // .run("/bin/whoami", [] as [&str; 0])
        .run("/bin/rustc", [ "-C", "linker=/bin/ld", "-C", "link-args=-L/lib", "-C", "link-args=-L/lib/gcc/x86_64-unknown-linux-gnu/14.2.1", "main.rs"])
        // .run(
        //     "/bin/rustc",
        //     ["-C", "linker=/bin/cc", "--print", "link-args", "main.rs"],
        // )
        .expect("Cannot run code");

    eprintln!("-- STDOUT\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("-- STDERR\n{}", String::from_utf8_lossy(&output.stderr));

    panic!("Forced stop")

    // assert_eq!(output.status.success(), true);
    // assert_eq!(output.stdout, "Hello World".as_bytes().to_vec());
}
