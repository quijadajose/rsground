#[tokio::main]
async fn main() {
    let runner = rsground_runner::Runner::new().await.unwrap();

    runner
        .create_file(
            "main.rs",
            r#"
            fn main() {
                println!("Hello World");
            }
            "#,
        )
        .await
        .unwrap();

    runner
        .create_file("main.c", r#"int main() { return 0; }"#)
        .await
        .unwrap();

    let mut cmd = runner.spawn("/bin/bash", ["-i"]).unwrap();

    cmd.wait().unwrap();
}
