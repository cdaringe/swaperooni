use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwapError {
    #[error("bad cli input")]
    BadCliInput,

    #[error("failed to boot child process: {0}")]
    FailedChildBoot(String),

    #[error("child process reported no PID")]
    FailedChildBootNoPid,

    #[error("failed to kill pid {pid:?}")]
    FailedChildKill { pid: u32, message: String },

    #[error("waiting for process failed {message:?}")]
    ProcWaitFail { message: String },

    #[error("swap event listener channel down")]
    ListenerChannelDown,

    #[error("swap event listener halted")]
    ListenerHalted,

    #[error("invalid cmd received over socket")]
    ListenerSocketInvalidCmd { message: String },

    #[error("signal proxy failed: {message}")]
    SignalProxyFailed { message: String },

    #[error("signal proxy halted")]
    SignalProxyHalted,
}
