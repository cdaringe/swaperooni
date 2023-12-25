use crate::{
    cli::{Cli, Commands},
    error::SwapError,
    init::{listen, Init},
    swap::{run, SwapBuilder},
};
use anyhow::Result;
use clap::Parser;
use core::panic;
use tokio::select;

pub async fn run_cli() -> Result<i32> {
    let args = Cli::parse();

    // @todo consider an abstraction, trait InitState for Command
    let (t, cmd, (tx, rx)) = match args.command {
        Commands::Poll(poll) => Init::from(poll),
        Commands::Ipc(_) => todo!(),
    }
    .to_tup();

    let listener = listen(t, &cmd, tx);

    let sr = SwapBuilder::new(&cmd).unwrap().start().await?;
    let running = run(sr, rx);

    select! {
      _ = listener => Err(SwapError::ListenerHalted.into()),
      proc_halted_result = running => proc_halted_result,
    }
}
