use std::process::exit;
use std::sync::Mutex;
use std::time::Duration;
use std::{process::Command, sync::Arc, thread};

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

pub fn main() -> () {
    let swap = Arc::new(Mutex::new(
        swaperooni::SwapBuilder::start(Command::new("sleep").arg("1000"))
            .expect("failed to boot first process"),
    ));

    let swap_a = swap.clone();
    thread::spawn(move || {
        let mut signals =
            Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
                .expect("failed to obtain signals");
        for sig in signals.forever() {
            swap_a.lock().unwrap().signal(sig).unwrap();
        }
    });

    let swap_b = swap.clone();
    let proc_haltled = thread::spawn(move || loop {
        let _ = {
            match swap_b.lock().unwrap().try_wait() {
                Ok(Some(status)) => exit(match status.code() {
                    Some(code) => code,
                    None => {
                        eprintln!("unable to determine exit status of child: {status}");
                        1
                    }
                }),
                Ok(None) => (),
                Err(e) => panic!("failed to get status of process: {}", e),
            }
        };
        thread::sleep(Duration::from_secs(1));
    });

    proc_haltled.join().unwrap()
}
