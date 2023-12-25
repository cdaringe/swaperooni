use crate::error::SwapError;
use crate::signals::proxy_common_signals;
use crate::{baby_cmd::BabyCommand, init::BabyRx};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::select;
use tokio::sync::Mutex;

pub enum SwapVersion {
    // default, naive counter
    Counter(usize),
    // user specified version
    UserVersion(usize),
    // user JSONpayload
    JsonVersion(String),
}

impl From<usize> for SwapVersion {
    fn from(value: usize) -> Self {
        SwapVersion::Counter(value)
    }
}

pub struct Swap {
    // if it doesn't have a pid, we're not swapping.
    pub pid: u32,
    pub current_cmd: BabyCommand,
    // history: Vec<SwapVersion>,
    count: usize,
}

fn run_cmd(cmd: &BabyCommand) -> Result<Child> {
    let child = cmd
        .cmd()
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| SwapError::FailedChildBoot(e.to_string()))?;
    Ok(child)
}

async fn get_pid_and_child(cmd: &BabyCommand) -> Result<(u32, Child)> {
    let mut child = run_cmd(cmd)?;
    let pc = tokio::spawn(async move {
        let mut tries = 10;
        while tries > 0 {
            let _ = child
                .try_wait()
                .map_err(|e| SwapError::FailedChildBoot(e.to_string()))?;
            match child.id() {
                Some(pid) => {
                    return Ok((pid, child));
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    ()
                }
            }
            tries -= 1;
        }
        Err(SwapError::FailedChildBoot(
            "could not get child pid".to_owned(),
        ))
    })
    .await??;

    Ok(pc)
}

pub async fn signal(pid: u32, signal: i32) -> Result<()> {
    Command::new("kill")
        .args([format!("-{}", signal), pid.to_string()])
        .spawn()
        .map_err(|e| SwapError::FailedChildKill {
            pid,
            message: e.to_string(),
        })?
        .wait()
        .await
        .map(|_| Ok(()))
        .map_err(|e| SwapError::FailedChildKill {
            pid,
            message: e.to_string(),
        })?
}

impl Swap {
    pub async fn swap_version(
        self,
        cmd: &BabyCommand,
        // version: SwapVersion,
    ) -> Result<SwapReady> {
        signal(self.pid, 9).await?;
        let (pid, child) = get_pid_and_child(cmd).await?;
        let next_count = self.count.clone() + 1;
        Ok(SwapReady {
            child,
            swap: Swap {
                pid,
                current_cmd: cmd.clone(),
                count: next_count,
            },
        })
    }

    pub async fn swap(self, cmd: &BabyCommand) -> Result<SwapReady> {
        self.swap_version(cmd).await //, { self.count + 1 }.into()).await
    }
}

pub struct SwapBuilder {
    pub cmd: BabyCommand,
}

impl SwapBuilder {
    pub fn new_version(cmd: &BabyCommand, _version: SwapVersion) -> Result<SwapBuilder, String> {
        Ok(SwapBuilder {
            cmd: cmd.to_owned(),
        })
    }
    pub fn new(cmd: &BabyCommand) -> Result<SwapBuilder, String> {
        Self::new_version(cmd, 1.into())
    }

    pub async fn start(self: Self) -> Result<SwapReady> {
        let (pid, child) = get_pid_and_child(&self.cmd).await?;
        Ok(SwapReady {
            child,
            swap: Swap {
                pid,
                current_cmd: self.cmd,
                count: 1,
            },
        })
    }
}

pub struct SwapReady {
    pub child: Child,
    pub swap: Swap,
}

pub async fn run(sr_0: SwapReady, rx: BabyRx) -> Result<i32> {
    let sr: SwapReady = sr_0;
    let child_arc = Arc::new(Mutex::new(sr.child));
    let mut swap = sr.swap;
    let rxx = Arc::new(Mutex::new(rx));
    loop {
        let pid = {
            child_arc
                .lock()
                .await
                .id()
                .ok_or_else(|| SwapError::FailedChildBootNoPid)?
        };
        let signal_proxy_f = tokio::spawn(async move { proxy_common_signals(pid).await });
        let run_child = child_arc.clone();

        let rxxx = rxx.clone();
        select! {
          halted = tokio::spawn(async move { run_child.lock().await.wait().await }) => {
            match halted.unwrap() {
              // Getting the exit code apparently isn't so staightforward according to rust.
              // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
              Ok(status) => return Ok(status.code().map_or_else(|| 1, |code| code)),
              Err(e) => return Err({ SwapError::ProcWaitFail { message: e.to_string() } }.into())
            }
          }
          _swap_error = signal_proxy_f => {
            // eager abort on signal error. should have been cancelled out of existence.
            // return Err(swap_error.into())
            panic!("arsat")
          }
          next_cmd_opt = tokio::spawn(async move { rxxx.lock().await.recv() }) => {
            let next_cmd = match next_cmd_opt.unwrap() {
              Ok(sr) => sr,
              _ => return Err(SwapError::ListenerHalted.into())
            };
            let next_sr = swap.swap(&next_cmd).await?;
            *child_arc.lock().await = next_sr.child;
            swap = next_sr.swap;
          }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     // use std::{thread, time::Duration, vec};
//     use tokio::time;

//     use super::*;

//     #[tokio::test]
//     async fn it_starts_a_process() {
//         let mut swap =
//             SwapBuilder::start(Command::new("echo").arg("foobar")).expect("starting proc failed");
//         let exit = swap.active.wait().await.expect("active proc failed");
//         assert_eq!(exit.success(), true);
//     }

//     #[tokio::test]
//     async fn it_swaps_a_process() {
//         let mut swap = SwapBuilder::start(Command::new("sleep").arg("10000")).expect("");
//         tokio::time::sleep(Duration::from_millis(100)).await;
//         assert_eq!(swap.count, 1);
//         swap.swap(Command::new("echo").args(vec!["swapped!"]))
//             .await
//             .unwrap();
//         assert_eq!(swap.count, 2);
//         swap.active.wait().await.unwrap();
//         ()
//     }
// }
