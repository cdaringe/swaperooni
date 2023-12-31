#!/usr/bin/env bash
set -eo pipefail

SOCKET_PATH=demo.sock cargo run ipc -- bash examples/socket_bash/a.sh
