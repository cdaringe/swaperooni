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
    iterator::{Signals, SignalsInfo},
};

use crate::{error::SwapError, swap::signal};

pub fn make_signals() -> SignalsInfo {
    Signals::new([SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
        .expect("failed to obtain signals")
}

pub async fn proxy_common_signals(mut signals: SignalsInfo, pid: u32) -> SwapError {
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
