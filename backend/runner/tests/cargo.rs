use common::cargo::cargo_init;
use common::print_output;
use rsground_runner::Runner;

mod common;

#[test]
fn cargo_build() {
    let runner = Runner::new().unwrap();
    cargo_init(&runner);

    let output = runner.run("/bin/cargo", ["build", "--release"]).unwrap();

    print_output(&output);

    assert_eq!(output.status.success(), true);
}

#[test]
fn cargo_run() {
    let runner = Runner::new().unwrap();
    cargo_init(&runner);

    let output = runner.run("/bin/cargo", ["build", "--release"]).unwrap();

    print_output(&output);
    assert_eq!(output.status.success(), true);

    let output = runner
        .patch_binary("/home/target/release/rsground-main")
        .unwrap();

    print_output(&output);
    assert_eq!(output.status.success(), true);

    let executer_runner = Runner::new().unwrap();
    executer_runner.copy_file_from_runner(&runner, "main", "target/release/rsground-main");

    let output = executer_runner.run("/home/main", [] as [&str; 0]).unwrap();

    print_output(&output);
    assert_eq!(output.status.success(), true);
    assert_eq!(output.stdout, "Hello World".as_bytes().to_vec());
}
