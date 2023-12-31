use std::process::exit;
use swaperooni::run::run_cli;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        exit(
            run_cli()
                .await
                .map(|code| exit(code))
                .unwrap_or_else(|err| {
                    eprintln!("{err}");
                    1
                }),
        )
    })
}
