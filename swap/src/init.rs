use crate::{baby_cmd::BabyCommand, cli::PollCmd, poll::poll_modified};
use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

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
    poll_modified(poll, || match tokio::runtime::Handle::try_current() {
        Ok(rt) => rt
            .block_on(async { tx.send(next_cmd.clone()).await })
            .map_err(|_| panic!("local channel unavailable")),
        Err(_) => panic!("impossible case: missing runtime"),
    })
    .await
}

impl From<PollCmd> for Init<PollCmd> {
    fn from(t: PollCmd) -> Self {
        let cmd = BabyCommand {
            bin: t.exe.to_owned(),
            args: vec![],
        };
        let channel: BabyCommandChannel = tokio::sync::mpsc::channel(1);
        Init { t, cmd, channel }
    }
}
