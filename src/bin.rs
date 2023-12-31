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

    fn test_cmd(args: Vec<&str>, exit_code: i32, lines: Vec<&str>) {
        Command::new("cargo")
            .args(["build", "--release"])
            .spawn()
            .unwrap();

        let mut child = Command::new("./target/release/swaperooni")
            .args(args)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .env("SOCKET_PATH", "demo.sock")
            .spawn()
            .unwrap();
        let exit_status = child.wait().unwrap();
        let err_buf = BufReader::new(child.stderr.unwrap());
        let err_lines: Vec<String> = err_buf.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        let empty_lines: Vec<String> = vec![];
        let out_buf = BufReader::new(child.stdout.unwrap());
        let out_lines = out_buf.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        print!("stderr:\n{err_lines:?}\n\nstdout:\n{out_lines:?}");
        assert_eq!(err_lines, empty_lines);
        assert_eq!(exit_status.code().unwrap(), exit_code);
        assert_eq!(out_lines, lines);
    }

    #[test]
    fn it_runs_poll_mode() {
        test_cmd(
            "poll --poll-interval-ms=1000 -- examples/poll_countdown/main.sh 2"
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
            "ipc -- bash examples/ipc_bash_hopscotch/a.sh"
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
            "ipc -- node examples/ipc_node_counter/index.mjs 8"
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
