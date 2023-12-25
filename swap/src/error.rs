use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwapError {
    #[error("failed to boot child process")]
    FailedChildBoot(String),

    #[error("child process reported no PID")]
    FailedChildBootNoPid,

    #[error("failed to kill pid {pid:?}")]
    FailedChildKill { pid: u32, message: String },

    #[error("waiting for process failed {message:?}")]
    ProcWaitFail { message: String },

    #[error("swap event listener halted")]
    ListenerHalted,

    #[error("signal proxy failed: {message}")]
    SignalProxyFailed { message: String },

    #[error("signal proxy halted")]
    SignalProxyHalted,
}
