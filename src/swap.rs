use crate::baby_cmd::BabyCommand;
use crate::error::SwapError;
use anyhow::Result;

use std::time::Duration;
use tokio::{
    process::{Child, Command},
    select,
};

pub struct Swap {
    pub pid: u32,
}

fn run_cmd(cmd: &BabyCommand) -> Result<Child> {
    tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| SwapError::FailedChildBoot(e.to_string()).into())
}

async fn get_pid_and_child(cmd: &BabyCommand) -> Result<(u32, Child)> {
    let mut child = run_cmd(cmd)?;
    Ok(async move {
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
            "failed to get pid for process attempting to launch".to_owned(),
        ))
    }
    .await?)
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
    pub async fn swap(self, cmd: &BabyCommand) -> Result<SwapReady> {
        let (pid, child) = get_pid_and_child(cmd).await?;
        Ok(SwapReady {
            child,
            swap: Swap { pid },
        })
    }
}

pub struct SwapBuilder {
    pub cmd: BabyCommand,
}

impl SwapBuilder {
    pub fn new(cmd: &BabyCommand) -> SwapBuilder {
        SwapBuilder {
            cmd: cmd.to_owned(),
        }
    }

    pub async fn start(self) -> Result<SwapReady> {
        let (pid, child) = get_pid_and_child(&self.cmd).await?;
        Ok(SwapReady {
            child,
            swap: Swap { pid },
        })
    }
}

pub struct SwapReady {
    pub child: Child,
    pub swap: Swap,
}
