use std::sync::{Arc, Mutex};

use anyhow::Result;
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

use crate::{error::SwapError, swap::signal};

pub async fn proxy_common_signals(pid: u32) -> SwapError {
    let mut signals = Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
        .expect("failed to obtain signals");
    for sig in signals.forever() {
        match signal(pid, sig).await {
            Ok(_) => (),
            Err(e) => {
                return SwapError::SignalProxyFailed {
                    message: e.to_string(),
                };
            }
        };
    }
    SwapError::SignalProxyHalted
}
