use swaperooni::run::run_cli;
// use signal_hook::{
//     consts::{
//         SIGABRT,
//         SIGALRM,
//         SIGHUP,
//         SIGINT,
//         SIGPIPE,
//         SIGQUIT,
//         SIGTERM,
//         // forbidden!
//         // SIGFPE,
//         // SIGILL,
//         // SIGKILL,
//         // SIGSEGV,
//     },
//     iterator::Signals,
// };
use std::process::exit;

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
