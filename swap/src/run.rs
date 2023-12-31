use crate::{
    cli::Cli,
    error::SwapError,
    init::{BabyRx, Init},
    signals::proxy_common_signals,
    swap::{signal, SwapBuilder, SwapReady},
};
use anyhow::Result;
use clap::Parser;
use core::panic;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;

pub async fn run_cli() -> Result<i32> {
    let args = Cli::parse();
    let (swap_listener, cmd, rx_swap_request) = Init::from(args.command).to_tup();
    select! {
      _ = swap_listener => Err(SwapError::ListenerHalted.into()),
      it = run(SwapBuilder::new(&cmd).start().await?, rx_swap_request) => it,
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
        let halted_fut = async move { run_child.lock().await.wait().await };
        println!("\n\nstarting PID {pid}");

        select! {
          halted = halted_fut => {
            dbg!(&halted);
            match halted {
              // Getting the exit code apparently isn't so straightforward according to rust.
              // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
              Ok(status) => return Ok(status.code().map_or_else(|| 1, |code| code)),
              Err(e) => return Err({ SwapError::ProcWaitFail { message: e.to_string() } }.into())
            }
          }

          // handle signal result (should almost never trigger)
          signal_r = signal_proxy_f => {
            dbg!(&signal_r);
            // eager abort on signal error. should have been cancelled out of existence.
            return match signal_r {
              Ok(_) => Err(SwapError::SignalProxyHalted.into()),
              Err(e) => Err(SwapError::SignalProxyFailed { message: e.to_string()}.into())
            }
          }

          // handle swap request
          next_cmd_opt = tokio::spawn(async move { rx_swap_request_arcx.lock().await.recv() }) => {
            dbg!(&next_cmd_opt);
            let next_cmd = next_cmd_opt?.map_err(|_| SwapError::ListenerHalted)?;
            let _ = signal(pid, 9).await;
            let next_sr = swap.swap(&next_cmd).await?;
            *child_arc.lock().await = next_sr.child;
            swap = next_sr.swap;
          }
        }
    }
}
