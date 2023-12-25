use std::process::exit;
use swaperooni::run::run_cli;

#[tokio::main]
async fn main() -> () {
    exit(
        run_cli()
            .await
            .map(|code| exit(code))
            .unwrap_or_else(|err| {
                eprintln!("{err}");
                1
            }),
    )
}
