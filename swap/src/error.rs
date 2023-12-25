use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwapError {
    #[error("failed to boot child process")]
    FailedChildBoot(String),

    #[error("failed to kill pid {pid:?}")]
    FailedChildKill { pid: u32, message: String },

    #[error("swap event listener halted")]
    ListenerHalted,
}
