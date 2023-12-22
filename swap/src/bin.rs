use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use signal_hook::{
    consts::{
        SIGABRT,
        SIGALRM,
        SIGHUP,
        SIGINT,
        SIGPIPE,
        SIGQUIT,
        SIGTERM,
        // forbidden!
        // SIGFPE,
        // SIGILL,
        // SIGKILL,
        // SIGSEGV,
    },
    iterator::Signals,
};
use swaperooni::signal;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> () {
    let swap = Arc::new(Mutex::new(
        swaperooni::SwapBuilder::start(
            tokio::process::Command::new("sleep")
                .arg("1000")
                .kill_on_drop(true),
        )
        .expect("failed to boot first process"),
    ));

    let sig_handler_swap = swap.clone();
    let _sigs_handle = tokio::spawn(async move {
        let mut signals =
            Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
                .expect("failed to obtain signals");
        for sig in signals.forever() {
            if let Some(active_pid) = sig_handler_swap.lock().await.active.id() {
                signal(active_pid, sig).unwrap();
            }
        }
    });

    loop {
        let swap_b = swap.clone();

        let proc_handle = tokio::spawn(async move {
            // holds the lock across the await. make sure everything you need
            // concurrently is available to other concurrent workers
            match swap_b.lock().await.wait().await {
                Ok(status) => exit(match status.code() {
                    Some(code) => code,
                    None => {
                        eprintln!("unable to determine exit status of child: {status}");
                        1
                    }
                }),
                Err(e) => panic!("failed to get status of process: {}", e),
            }
        });

        proc_handle.await.unwrap();
    }
}
