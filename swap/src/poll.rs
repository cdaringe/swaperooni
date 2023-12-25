use std::time::Duration;

use anyhow::Result;

use crate::cli::PollCmd;

pub async fn poll_modified(poll: PollCmd, on_change: impl Fn() -> Result<()>) -> Result<()> {
    let mut t_prev = crate::fs::get_modified(&poll.exe.clone()).await?;
    loop {
        dbg!("@poll", poll.poll_interval_ms);
        tokio::time::sleep(Duration::from_millis(poll.poll_interval_ms)).await;
        let t_next = crate::fs::get_modified(&poll.exe.clone()).await?;
        dbg!(t_next, t_prev, t_next > t_prev);
        if t_next > t_prev {
            dbg!("@swapping");
            on_change()?;
            dbg!("@swapped");
        }
        t_prev = t_next;
    }
}
