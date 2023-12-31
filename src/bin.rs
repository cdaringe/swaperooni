use std::process::exit;
use swaperooni::run::run_cli;

#[tokio::main]
async fn main() {
    exit(
        run_cli()
            .await
            .map(|code| exit(code))
            .unwrap_or_else(|err| {
                eprintln!("{err}");
                1
            }),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };

    fn test_cmd(cmd: &str, args: Vec<&str>, exit_code: i32, lines: Vec<&str>) {
        let mut child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .env("SOCKET_PATH", "demo.sock")
            .spawn()
            .unwrap();
        let exit_status = child.wait().unwrap();
        let buf = BufReader::new(child.stdout.unwrap());
        let actual_lines = buf.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        assert_eq!(actual_lines, lines);
        assert_eq!(exit_status.code().unwrap(), exit_code);
    }

    #[test]
    fn it_runs_poll_mode() {
        test_cmd(
            "cargo",
            "run poll --poll-interval-ms=1000 -- examples/poll_countdown/main.sh 2"
                .split(' ')
                .collect(),
            0,
            vec![
                "Change this text! 2",
                "Change this text! 1",
                "no change detected, exiting",
            ],
        )
    }

    #[test]
    fn it_runs_ipc_hopscotch() {
        test_cmd(
            "cargo",
            "run ipc -- bash examples/ipc_bash_hopscotch/a.sh"
                .split(' ')
                .collect(),
            0,
            vec![
                "[a] Greetings from examples/ipc_bash_hopscotch/a.sh",
                "[b] Guten tag from examples/ipc_bash_hopscotch/b.sh",
                "[c] Bienvenidos de examples/ipc_bash_hopscotch/c.sh",
                "finished.",
            ],
        )
    }

    #[test]
    fn it_runs_ipc_counter() {
        test_cmd(
            "cargo",
            "run ipc -- node examples/ipc_node_counter/index.mjs 8"
                .split(' ')
                .collect(),
            2,
            vec![
                "[child] started with id: 8",
                "[child] sending cmd: node examples/ipc_node_counter/index.mjs 9",
                "",
                "[child] started with id: 9",
                "[child] sending cmd: node examples/ipc_node_counter/index.mjs 10",
                "",
                "10 reached, exiting with code 2",
            ],
        )
    }
}
