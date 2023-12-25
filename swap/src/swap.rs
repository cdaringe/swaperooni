use crate::error::SwapError;
use crate::{baby_cmd::BabyCommand, init::BabyRx};
use anyhow::Result;
use std::time::Duration;
use tokio::process::{Child, Command};

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

pub async fn run(_sr: SwapReady, _rx: BabyRx) -> Result<i32> {
    todo!()
    // bind sigs
    // wait for finish
    // listen for swap events
    // on event, swap!
    // let cmd_arc: Arc<Mutex<Option<BabyCommand>>> = Arc::new(Mutex::new(Some(cmd_0.clone())));

    // let pid_arc: Arc<std::sync::Mutex<Option<u32>>> = Arc::new(std::sync::Mutex::new(None));

    // let poll_cmd_arc = cmd_arc.clone();
    // let poll_acid_arc = pid_arc.clone();
    // let _poller = match args.command {
    //     cli::Commands::Poll(poll) => {
    //         tokio::spawn(async move { poll_swap(poll, poll_cmd_arc, &cmd_0, poll_acid_arc).await })
    //     }
    //     cli::Commands::Ipc(_) => todo!(),
    // };

    // // signal proxy
    // let signal_acid_arc = pid_arc.clone();
    // let _ = tokio::spawn(async move {
    //     let mut signals =
    //         Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGABRT, SIGPIPE, SIGALRM, SIGTERM])
    //             .expect("failed to obtain signals");
    //     for sig in signals.forever() {
    //         let id: u32 = {
    //             let mut id: u32 = 0;
    //             while id == 0 {
    //                 let maybe_id = { signal_acid_arc.lock().unwrap().to_owned() };
    //                 match maybe_id {
    //                     Some(aid) => {
    //                         id = aid;
    //                         break;
    //                     }
    //                     None => tokio::time::sleep(Duration::from_millis(10)).await,
    //                 }
    //             }
    //             id
    //         };
    //         signal(id, sig).unwrap()
    //     }
    // });

    // loop {
    //     // update active child id s.t. signals can be passed into it
    //     let before_id: i32 = {
    //         let x: Option<u32> = pid_arc.lock().unwrap().clone();
    //         match x {
    //             Some(x) => i32::try_from(x).unwrap(),
    //             None => -1,
    //         }
    //     };

    //     // wait for child to halt
    //     let proc_pid_arc = pid_arc.clone();
    //     let proc_cmd_arc = cmd_arc.clone();
    //     let proc_result = tokio::spawn(async move {
    //         let cmd = {
    //             let cmd: Option<BabyCommand> = *proc_cmd_arc.lock().expect("failed to lock");
    //             cmd.take()
    //         };
    //         match cmd {
    //             Some(cmd) => {
    //                 let mut running = cmd.cmd().kill_on_drop(true).spawn().unwrap();
    //                 // update
    //                 {
    //                     let active_id = running.id();
    //                     if active_id.is_none() {
    //                         panic!("expected a PID, got none");
    //                     }
    //                     *proc_pid_arc.lock().unwrap() = active_id;
    //                     println!("updating pid to  {}", active_id.unwrap());
    //                 };
    //                 // holds the lock across the await. make sure everything you need
    //                 // concurrently is available to other concurrent workers
    //                 running.wait().await.map(|status| {
    //                     println!("@PROC_HALTED WITH {status}");
    //                     // Getting the exit code apparently isn't so staightforward according to rust.
    //                     // https://doc.rust-lang.org/std/process/struct.ExitStatus.html#method.code
    //                     status.code().map_or_else(|| 1, |code| Some(code))
    //                 })
    //             }
    //             None => {
    //                 return Ok(None);
    //             }
    //         }
    //     })
    //     .await;

    //     // check if we've swapped
    //     let after_id: i32 = {
    //         let x: Option<u32> = pid_arc.lock().unwrap().clone();
    //         match x {
    //             Some(x) => i32::try_from(x).unwrap(),
    //             None => -2,
    //         }
    //     };
    //     dbg!("ok, so were in the loop", before_id, after_id);
    //     match (after_id.eq(&before_id), proc_result) {
    //         (true, Ok(Ok(exit_code))) => exit(exit_code),
    //         (true, Ok(Err(e))) => {
    //             eprintln!("swap process halted, but child proc failed: {e}");
    //             return Err(e.to_string());
    //         }
    //         (true, Err(msg)) => {
    //             eprintln!("swap process failed");
    //             return Err(msg.to_string());
    //         }
    //         // continue!
    //         (false, _) => (),
    //     }
    // }
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
