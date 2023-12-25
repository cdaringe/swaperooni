use std::process::exit;
use swaperooni::run::run_cli;

#[tokio::main]
async fn main() -> () {
    run_cli()
        .await
        .map(|code| exit(code))
        .map_err(|err| {
            eprintln!("{err}");
            exit(1)
        })
        .unwrap()
}
