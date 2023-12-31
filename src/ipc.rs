use crate::{baby_cmd::BabyCommand, error::SwapError, init::BabyTx};
use anyhow::Result;
use tokio::{io::AsyncReadExt, net::UnixStream};

pub async fn read_forever(stream: &mut UnixStream, tx: &BabyTx) -> Result<()> {
    let mut buff = vec![0; 4095];
    loop {
        let read = stream.read(&mut buff).await?;
        match read {
            // zero indicates "ill send no more" very weakly
            0 => {
                return Ok(());
            }
            _ => {
                let utf8_str = String::from_utf8(buff.clone()).map_err(|e| {
                    SwapError::ListenerSocketInvalidCmd {
                        message: e.to_string(),
                    }
                })?;
                let cmd = utf8_str.split('\n').find(|_| true).unwrap();
                let (bin, args) = match cmd.split_once(' ') {
                    Some((bin, args_str)) => (bin, args_str.split_whitespace().collect()),
                    None => (cmd, vec![]),
                };

                // clear buff
                let _ = std::mem::replace(&mut buff, vec![0; 4095]);

                let bc = BabyCommand {
                    bin: bin.to_owned(),
                    args: args.iter().map(|&x| x.to_owned()).collect(),
                };
                tx.send(bc).map_err(|_| SwapError::ListenerChannelDown)?
            }
        }
    }
}
