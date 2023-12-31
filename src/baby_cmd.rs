/// A simplified representation of std::process::Command
#[derive(Clone, Debug)]
pub struct BabyCommand {
    pub bin: String,
    pub args: Vec<String>,
}

impl From<&BabyCommand> for tokio::process::Command {
    fn from(value: &BabyCommand) -> Self {
        let mut cmd = tokio::process::Command::new(value.bin.clone());
        cmd.args(value.args.clone());
        cmd
    }
}
