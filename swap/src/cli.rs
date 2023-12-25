// swaperooni --poll-exe-mtime=foo -- bar
// swaperooni --pipe=/dev/swapper -- node server
// swaperooni --ipc-cmd=/dev/swapper --wait-for-first

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct PollCmd {
    /// Name of the person to greet
    pub exe: String,

    #[arg(short = 'i', long, default_value_t = 10_000)]
    pub poll_interval_ms: u64,
}

#[derive(Clone, Subcommand, Debug)]
pub enum Commands {
    Poll(PollCmd),
    Ipc(IpcCmd),
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct IpcCmd {
    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
    #[arg(last = true)]
    pub cmd: Vec<String>,
}
