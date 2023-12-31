# swaperooni

Hot-reload a child process without tearing down the parent PID. Proxies signals
into the child process.

- if the child exits, `swaperooni` exits with the same code.
- if a swap request is issued, the child is torn down, and new child brought up.

Supports two hot-reload request options:

- `ipc` - send a newline delimited command string from the child to the swaperooni socket. See [examples/socket_bash/main.sh](examples/socket_bash/main.sh).
- `poll` - monitor the `mtime` of a file. on change, re-execute it.

**swaperooni is not a supervisor.** `swaperooni` is similar to [tini](https://github.com/krallin/tini).

## Why

Because you can. Generally, avoid using this, and use a supervisor/orchestrator.
You may have cases where swapping child PIDs is more desirable (e.g. oddball docker/k8s environments).

## How

See the examples.

- `SOCKET_PATH=demo.sock cargo run ipc -- bash examples/socket_bash/a.sh`
  - program `a` runs momentarily, then requests to be swapped for program `b`. `b` requests program `c`, and `c` gracefully exits.
- `SOCKET_PATH=demo.sock cargo run ipc -- node examples/socket_node/index.mjs 0`
  - run a node.js process. the process asks swaperooni to run itself again, but with different params (a counter)
- `SOCKET_PATH=demo.sock cargo run -- poll --poll-interval-ms=1000 examples/swap_on_file_change/worker-app.sh`
  - poll for entrypoint change and re-run it.

## Performance

Well, it is <1MB and does nothing _most_ of the time.
