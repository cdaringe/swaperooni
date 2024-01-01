#!/usr/bin/env bash
set -eo pipefail

echo "[a] Greetings from examples/ipc_bash_hopscotch/a.sh"
sleep 1

# send the next command to swaperooni
echo "bash examples/ipc_bash_hopscotch/b.sh" | nc -w 1 -U "$SOCKET_PATH"

# wait for a to swapped out
sleep 60
