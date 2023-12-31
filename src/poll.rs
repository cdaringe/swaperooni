use std::time::Duration;

use anyhow::Result;

use crate::cli::PollCmd;

pub async fn poll_modified(poll: PollCmd, on_change: impl Fn() -> Result<()>) -> Result<()> {
    let mut t_prev = crate::fs::get_modified(&poll.exe.clone()).await?;
    loop {
        tokio::time::sleep(Duration::from_millis(poll.poll_interval_ms)).await;
        let t_next = crate::fs::get_modified(&poll.exe.clone()).await?;
        if t_next > t_prev {
            on_change()?;
        }
        t_prev = t_next;
    }
}
