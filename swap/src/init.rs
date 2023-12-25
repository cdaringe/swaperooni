use crate::{baby_cmd::BabyCommand, cli::PollCmd, error::SwapError, poll::poll_modified};
use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};

pub type BabyTx = Sender<BabyCommand>;
pub type BabyRx = Receiver<BabyCommand>;
pub type BabyCommandChannel = (BabyTx, BabyRx);

pub struct Init<T> {
    t: T,
    pub cmd: BabyCommand,
    pub channel: BabyCommandChannel,
}

impl<T> Init<T> {
    pub fn to_tup(self: Self) -> (T, BabyCommand, BabyCommandChannel) {
        (self.t, self.cmd, self.channel)
    }
}

pub async fn listen(poll: PollCmd, cmd: &BabyCommand, tx: BabyTx) -> Result<()> {
    let next_cmd = cmd.clone();
    poll_modified(poll, || {
        tx.send(next_cmd.clone())
            .map_err(|_| SwapError::ListenerChannelDown.into())
    })
    .await
}

impl From<PollCmd> for Init<PollCmd> {
    fn from(t: PollCmd) -> Self {
        let cmd = BabyCommand {
            bin: t.exe.to_owned(),
            args: vec![],
        };
        let channel: BabyCommandChannel = std::sync::mpsc::channel();
        Init { t, cmd, channel }
    }
}
