// swaperooni --poll-exe-mtime=foo -- bar
// swaperooni --pipe=/dev/swapper -- node server
// swaperooni --ipc-cmd=/dev/swapper --wait-for-first

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SwapCmd,
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct PollCmd {
    /// Executable to monitor and run
    pub exe: String,

    // Duration (milliseconds) between mtime poll
    #[arg(short = 'i', long, default_value_t = 4_000)]
    pub poll_interval_ms: u64,
}

#[derive(Clone, Subcommand, Debug)]
pub enum SwapCmd {
    Poll(PollCmd),
    Ipc(IpcCmd),
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct IpcCmd {
    #[arg(short = 's', long, env = "SOCKET_PATH")]
    pub socket_path: String,
    #[arg(last = true)]
    pub cmd: Vec<String>,
}
