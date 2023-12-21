use std::process::Child;
use std::sync::mpsc;
use std::{
    process::Command,
    thread::{self, JoinHandle},
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
    active: Child,
    history: Vec<SwapVersion>,
    count: usize,
}

pub trait Runnable {
    fn run(self) -> Result<Child, String>;
}

struct ExecShell(String, Vec<String>);

impl ExecShell {
    pub fn new(cmd: &str, args: Vec<&str>) -> Self {
        Self(cmd.into(), args.iter().map(|&x| x.into()).collect())
    }
}

impl Runnable for ExecShell {
    fn run(self) -> Result<Child, String> {
        Command::new(self.0)
            .args(self.1)
            .spawn()
            .map_err(|x| x.to_string())
    }
}

impl Swap {
    pub fn start_version<R: 'static + Runnable + Send>(
        runnable: R,
        version: SwapVersion,
    ) -> Result<Self, String> {
        Ok(Self {
            active: runnable.run()?,
            history: vec![version],
            count: 1,
        })
    }
    pub fn start<R: 'static + Runnable + Send>(runnable: R) -> Result<Self, String> {
        Self::start_version(runnable, 1.into())
    }

    pub fn swap_version<R: 'static + Runnable + Send>(
        &mut self,
        runnable: R,
        version: SwapVersion,
    ) -> Result<&Self, String> {
        self.count += 1;
        self.history.push(version);
        self.active.kill().map_err(|e| e.to_string())?;
        self.active = runnable.run()?;
        Ok(self)
    }

    pub fn swap<R: 'static + Runnable + Send>(&mut self, runnable: R) -> Result<&Self, String> {
        self.swap_version(runnable, { self.count + 1 }.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, vec};

    use super::*;

    #[test]
    fn it_starts_a_process() {
        let swap = Swap::start(ExecShell::new("echo", vec!["foobar"]));
        let exit = swap
            .expect("starting proc failed")
            .active
            .wait()
            .expect("active proc failed");
        assert_eq!(exit.success(), true);
    }

    #[test]
    fn it_swaps_a_process() {
        let mut swap = Swap::start(ExecShell::new("sleep", vec!["10000"])).expect("started");
        thread::sleep(Duration::from_millis(100));
        assert_eq!(swap.count, 1);
        swap.swap(ExecShell::new("echo", vec!["swapped!"]));
        assert_eq!(swap.count, 2);
        swap.active.wait().expect("echo swapped ok");
        ()
    }
}
