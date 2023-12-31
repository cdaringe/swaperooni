use crate::baby_cmd::BabyCommand;
use crate::error::SwapError;
use anyhow::Result;

use std::time::Duration;
use tokio::{
    process::{Child, Command},
    select,
};

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
    tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| SwapError::FailedChildBoot(e.to_string()).into())
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
                None => tokio::time::sleep(Duration::from_millis(50)).await,
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

pub async fn teardown(pid: u32) -> Result<()> {
    let sigterm_fut = signal(pid, 15);
    let sigkill_timeout_fut = async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let _ = signal(pid, 9).await;
    };
    select! {
      _ = sigterm_fut => (),
      _ = sigkill_timeout_fut => (),
    };
    Ok(())
}

impl Swap {
    pub async fn swap_version(
        self,
        cmd: &BabyCommand,
        // version: SwapVersion,
    ) -> Result<SwapReady> {
        // let _ = signal(self.pid, 9).await;
        let (pid, child) = get_pid_and_child(cmd).await?;
        let next_count = self.count + 1;
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
    pub fn new_version(cmd: &BabyCommand, _version: SwapVersion) -> SwapBuilder {
        SwapBuilder {
            cmd: cmd.to_owned(),
        }
    }
    pub fn new(cmd: &BabyCommand) -> SwapBuilder {
        Self::new_version(cmd, 1.into())
    }

    pub async fn start(self) -> Result<SwapReady> {
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
