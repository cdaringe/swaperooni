use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;

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
    active: Child,
    history: Vec<SwapVersion>,
    count: usize,
}

fn run_cmd(cmd: &mut Command) -> Result<Child, String> {
    cmd.spawn().map_err(|e| e.to_string())
}

impl Swap {
    pub fn swap_version(
        &mut self,
        cmd: &mut Command,
        version: SwapVersion,
    ) -> Result<&Self, String> {
        self.count += 1;
        self.history.push(version);
        self.active.kill().map_err(|e| e.to_string())?;
        // wait the killed process to ensure we reap the zombie.
        // not waiting == zombie.
        self.active.wait().map_err(|e| e.to_string())?;
        self.active = run_cmd(cmd)?;
        Ok(self)
    }

    pub fn swap(&mut self, cmd: &mut Command) -> Result<&Self, String> {
        self.swap_version(cmd, { self.count + 1 }.into())
    }

    pub fn signal(&mut self, signal: i32) -> Result<(), String> {
        dbg!("doing work i guess", signal);
        Command::new("kill")
            .args([format!("-{}", signal), self.active.id().to_string()])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("failed to send signal to active proc: {}", e))
    }

    pub fn wait(&mut self) -> Result<ExitStatus, String> {
        self.active.wait().map_err(|e| e.to_string())
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, String> {
        self.active.try_wait().map_err(|e| e.to_string())
    }
}

pub struct SwapBuilder();

impl SwapBuilder {
    pub fn start_version(cmd: &mut Command, version: SwapVersion) -> Result<Swap, String> {
        Ok(Swap {
            active: run_cmd(cmd)?,
            count: 1,
            history: vec![version],
        })
    }
    pub fn start(cmd: &mut Command) -> Result<Swap, String> {
        Self::start_version(cmd, 1.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration, vec};

    use super::*;

    #[test]
    fn it_starts_a_process() {
        let mut swap =
            SwapBuilder::start(Command::new("echo").arg("foobar")).expect("starting proc failed");
        let exit = swap.active.wait().expect("active proc failed");
        assert_eq!(exit.success(), true);
    }

    #[test]
    fn it_swaps_a_process() {
        let mut swap = SwapBuilder::start(Command::new("sleep").arg("10000")).expect("");
        thread::sleep(Duration::from_millis(100));
        assert_eq!(swap.count, 1);
        swap.swap(Command::new("echo").args(vec!["swapped!"]))
            .expect("swap failed");
        assert_eq!(swap.count, 2);
        swap.active.wait().expect("echo swapped ok");
        ()
    }
}
