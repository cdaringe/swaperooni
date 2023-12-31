use crate::{
    cli::Cli,
    error::SwapError,
    init::{BabyRx, Init},
    signals::{make_signals, proxy_common_signals},
    swap::{teardown, SwapBuilder, SwapReady},
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
    let started = async { swap_listener.await };
    select! {
      _ = started => Err(SwapError::ListenerHalted.into()),
      it = run(SwapBuilder::new(&cmd).start().await?, rx_swap_request) => it,
    }
}

pub async fn run(sr: SwapReady, rx_swap_request: BabyRx) -> Result<i32> {
    let child_arc = Arc::new(Mutex::new(sr.child));
    let mut swap = sr.swap;
    let rx_swap_request_arc = Arc::new(Mutex::new(rx_swap_request));
    let is_swapping = Arc::new(Mutex::new(false));
    let mut signal_thread = None;
    loop {
        {
            *is_swapping.clone().lock().await = false;
        };
        let pid = child_arc
            .lock()
            .await
            .id()
            .ok_or(SwapError::FailedChildBootNoPid)?;

        let signals = make_signals();
        let signals_handle = signals.handle();
        signal_thread = Some(signal_thread.unwrap_or_else(|| {
            tokio::spawn(async move { proxy_common_signals(signals, pid).await })
        }));

        let child_arc_swap = child_arc.clone();
        let child_arc_halted = child_arc.clone();

        let rx_swap_request_arcx = rx_swap_request_arc.clone();
        let halted_fut = async move { child_arc_halted.lock().await.wait().await };

        let is_swapping_arc_halted = is_swapping.clone();
        let is_swapping_arc_swap_request = is_swapping.clone();
        select! {
          halted = halted_fut => {
            match (*is_swapping_arc_halted.lock().await, halted) {
              (true, _) => (),
              // Getting the exit code apparently isn't so straightforward according to rust.
              // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
              (_, Ok(status)) => return Ok(status.code().map_or_else(|| 1, |code| code)),
              (_, Err(e)) => return Err({ SwapError::ProcWaitFail { message: e.to_string() } }.into())
            }
          }

          // handle swap request
          next_cmd_opt = tokio::spawn(async move { rx_swap_request_arcx.lock().await.recv() }) => {
            // update swapping state s.t. other tasks are cognizant
            {
              *is_swapping_arc_swap_request.lock().await = true;
            };

            // tear down the signals thread
            signals_handle.close();
            let _ = signal_thread.take(); // empty it out baby

            // tear down the last proc
            let _ = teardown(pid).await;
            let _ = child_arc_swap.lock().await.wait().await;

            // setup the swapped cmd
            let next_cmd = next_cmd_opt?.map_err(|_| SwapError::ListenerHalted)?;
            let next_sr = swap.swap(&next_cmd).await?;
            *child_arc.lock().await = next_sr.child;
            swap = next_sr.swap;

            // ...and lett'er loop to kickstart the next next
          }
        }
    }
}
