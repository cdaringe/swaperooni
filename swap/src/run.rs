use crate::{
    cli::Cli,
    error::SwapError,
    init::{BabyRx, Init},
    signals::proxy_common_signals,
    swap::{SwapBuilder, SwapReady},
};
use anyhow::Result;
use clap::Parser;
use core::panic;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;

pub async fn run_cli() -> Result<i32> {
    let args = Cli::parse();
    let (listener, cmd, rx_swap_request) = Init::from(args.command).to_tup();
    let proc_fut = run(SwapBuilder::new(&cmd).start().await?, rx_swap_request);
    select! {
      _ = listener => Err(SwapError::ListenerHalted.into()),
      proc_result = proc_fut => proc_result,
    }
}

pub async fn run(sr: SwapReady, rx_swap_request: BabyRx) -> Result<i32> {
    let child_arc = Arc::new(Mutex::new(sr.child));
    let mut swap = sr.swap;
    let rx_swap_request_arc = Arc::new(Mutex::new(rx_swap_request));
    loop {
        let pid = child_arc
            .lock()
            .await
            .id()
            .ok_or(SwapError::FailedChildBootNoPid)?;

        let signal_proxy_f = tokio::spawn(async move { proxy_common_signals(pid).await });
        let run_child = child_arc.clone();

        let rx_swap_request_arcx = rx_swap_request_arc.clone();
        select! {
          halted = tokio::spawn(async move { run_child.lock().await.wait().await }) => {
            match halted? {
              // Getting the exit code apparently isn't so staightforward according to rust.
              // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
              Ok(status) => return Ok(status.code().map_or_else(|| 1, |code| code)),
              Err(e) => return Err({ SwapError::ProcWaitFail { message: e.to_string() } }.into())
            }
          }
          swap_error = signal_proxy_f => {
            // eager abort on signal error. should have been cancelled out of existence.
            return match swap_error {
              Ok(_) => Err(SwapError::SignalProxyHalted.into()),
              Err(e) => Err(SwapError::SignalProxyFailed { message: e.to_string()}.into())
            }
          }
          next_cmd_opt = tokio::spawn(async move { rx_swap_request_arcx.lock().await.recv() }) => {
            let next_cmd = next_cmd_opt?.map_err(|_| SwapError::ListenerHalted)?;
            let next_sr = swap.swap(&next_cmd).await?;
            *child_arc.lock().await = next_sr.child;
            swap = next_sr.swap;
          }
        }
    }
}
