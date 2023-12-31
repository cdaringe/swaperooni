# swaperooni

Hot-reload a child process without tearing down the parent PID. Proxies signals
into the child process.

- if the child exits, `swaperooni` exits with the same code.
- if a swap request is issued, the child is torn down, and new child brought up.

Supports two hot-reload request options:

- `ipc` - send a newline delimited command string from the child to the swaperooni socket. See [examples/socket_bash/main.sh](examples/socket_bash/main.sh).
- `poll` - monitor the `mtime` of a file. on change, re-execute it.

**swaperooni is not a supervisor.**

## Why

Because you can. Generally, avoid using this, and use a supervisor/orchestrator.
You may have cases where swapping child PIDs is more desirable (e.g. oddball docker/k8s environments).

## How

See the examples.
