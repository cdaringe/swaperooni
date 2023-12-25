use crate::{
    baby_cmd::BabyCommand,
    cli::{PollCmd, SwapCmd},
    error::SwapError,
    poll::poll_modified,
};
use anyhow::Result;
use std::{
    future::Future,
    sync::mpsc::{Receiver, Sender},
};

pub type BabyTx = Sender<BabyCommand>;
pub type BabyRx = Receiver<BabyCommand>;
pub type BabyCommandChannel = (BabyTx, BabyRx);

pub struct Init {
    swap_cmd: SwapCmd,
    pub cmd: BabyCommand,
    pub channel: BabyCommandChannel,
}

impl Init {
    pub fn to_tup(self: Self) -> (impl Future<Output = Result<()>>, BabyCommand, BabyRx) {
        let Init {
            channel,
            cmd,
            swap_cmd: command,
        } = self;
        let (tx, rx) = channel;
        (
            match command {
                SwapCmd::Poll(poll) => listen(poll, cmd.clone(), tx),
                SwapCmd::Ipc(_) => todo!(),
            },
            cmd,
            rx,
        )
    }
}

pub async fn listen(poll: PollCmd, cmd: BabyCommand, tx: BabyTx) -> Result<()> {
    let next_cmd = cmd.clone();
    poll_modified(poll, || {
        tx.send(next_cmd.clone())
            .map_err(|_| SwapError::ListenerChannelDown.into())
    })
    .await
}

impl From<SwapCmd> for Init {
    fn from(swap_cmd: SwapCmd) -> Self {
        let channel: BabyCommandChannel = std::sync::mpsc::channel();
        let cmd = match swap_cmd {
            SwapCmd::Poll(ref poll) => BabyCommand {
                bin: poll.exe.clone(),
                args: vec![],
            },
            SwapCmd::Ipc(_) => todo!(),
        };
        Init {
            swap_cmd,
            cmd,
            channel,
        }
    }
}
