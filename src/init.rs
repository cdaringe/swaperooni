use crate::{
    baby_cmd::BabyCommand,
    cli::{IpcCmd, PollCmd, SwapCmd},
    error::SwapError,
    ipc::read_forever,
    poll::poll_modified,
};
use anyhow::Result;
use std::{
    future::Future,
    pin::Pin,
    sync::mpsc::{Receiver, Sender},
};
use tokio::net::UnixListener;

pub type BabyTx = Sender<BabyCommand>;
pub type BabyRx = Receiver<BabyCommand>;
pub type BabyCommandChannel = (BabyTx, BabyRx);

pub struct Init {
    swap_cmd: SwapCmd,
    pub cmd: BabyCommand,
    pub channel: BabyCommandChannel,
}

impl Init {
    pub fn to_tup(self) -> (impl Future<Output = Result<()>>, BabyCommand, BabyRx) {
        let Init {
            channel,
            cmd,
            swap_cmd: command,
        } = self;
        let (tx, rx) = channel;
        let listener = match command {
            SwapCmd::Poll(poll) => Box::pin(listen_poll(poll, cmd.clone(), tx))
                as Pin<Box<dyn Future<Output = Result<()>>>>,
            SwapCmd::Ipc(ipc_cmd) => Box::pin(listen_ipc(ipc_cmd, tx)),
        };
        (listener, cmd, rx)
    }
}

// on modified, re-emit cmd
pub async fn listen_poll(poll: PollCmd, cmd: BabyCommand, tx: BabyTx) -> Result<()> {
    let next_cmd = cmd.clone();
    poll_modified(poll, || {
        tx.send(next_cmd.clone())
            .map_err(|_| SwapError::ListenerChannelDown.into())
    })
    .await
}

// accept new cmd over ipc
pub async fn listen_ipc(ipc_cmd: IpcCmd, tx: BabyTx) -> Result<()> {
    let path = ipc_cmd.socket_path;
    let _ = std::fs::remove_file(&path);
    let rx = UnixListener::bind(path)?;
    loop {
        let (mut stream, _addr) = rx.accept().await?;
        read_forever(&mut stream, &tx).await?;
    }
}

impl From<Vec<String>> for BabyCommand {
    fn from(value: Vec<String>) -> Self {
        if value.len() < 2 {
            BabyCommand {
                bin: value.join(" "),
                args: vec![],
            }
        } else {
            let (a, b) = value.split_at(1);
            BabyCommand {
                bin: a[0].clone(),
                args: b.into(),
            }
        }
    }
}

impl From<SwapCmd> for Init {
    fn from(swap_cmd: SwapCmd) -> Self {
        let channel: BabyCommandChannel = std::sync::mpsc::channel();
        let cmd = match swap_cmd {
            SwapCmd::Poll(ref poll) => BabyCommand::from(poll.exe.clone()),
            SwapCmd::Ipc(ref ipc) => BabyCommand::from(ipc.cmd.clone()),
        };
        Init {
            channel,
            cmd,
            swap_cmd,
        }
    }
}
