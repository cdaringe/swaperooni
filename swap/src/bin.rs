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
use swaperooni::{signal, SwapBuilder};
use tokio::{process::Command, sync::Mutex};

#[tokio::main]
async fn main() -> () {
    let swap = Arc::new(Mutex::new(
        SwapBuilder::start(
            Command::new("bash")
                .arg("swap/examples/swap_on_file_change/worker-app.sh")
                .kill_on_drop(true),
        )
        .expect("failed to boot first process"),
    ));

    let active_child_id_arc: Arc<std::sync::Mutex<Option<u32>>> =
        Arc::new(std::sync::Mutex::new(None));

    let signal_acid_arc = active_child_id_arc.clone();
    let _ = tokio::spawn(async move {
        let mut signals =
            Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
                .expect("failed to obtain signals");
        for sig in signals.forever() {
            let id: u32 = {
                let mut id: u32 = 0;
                while id == 0 {
                    let maybe_id = { signal_acid_arc.lock().unwrap().to_owned() };
                    match maybe_id {
                        Some(aid) => {
                            id = aid;
                            break;
                        }
                        None => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
                id
            };
            signal(id, sig).unwrap()
        }
    });

    loop {
        let swap_arc = swap.clone();

        // update active child id s.t. signals can be passed into it
        {
            let next_id = swap_arc.lock().await.active.id();
            *active_child_id_arc.lock().unwrap() = next_id;
        };

        let proc_handle = tokio::spawn(async move {
            // holds the lock across the await. make sure everything you need
            // concurrently is available to other concurrent workers
            swap_arc.lock().await.wait().await.map(|status| {
                // Getting the exit code apparently isn't so staightforward according to rust.
                // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
                status.code().map_or_else(|| 1, |code| code)
            })
        });

        match proc_handle.await.expect("failed to complete child process") {
            Ok(exit_code) => exit(exit_code),
            Err(msg) => {
                eprintln!("{msg}");
                exit(1)
            }
        }
    }
}
