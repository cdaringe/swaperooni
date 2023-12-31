#!/usr/bin/env bash
set -eo pipefail

SOCKET_PATH=demo.sock cargo run ipc -- bash examples/ipc_bash_hopscotch/a.sh
