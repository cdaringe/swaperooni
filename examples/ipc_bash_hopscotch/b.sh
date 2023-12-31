#!/usr/bin/env bash
set -eo pipefail

echo "[b] Guten tag from examples/ipc_bash_hopscotch/b.sh"
sleep 1

# send the next command to swaperooni
echo 'bash examples/ipc_bash_hopscotch/c.sh' | nc -w 1 -U "$SOCKET_PATH"

# wait for b to swapped out
exec bash examples/ipc_bash_hopscotch/sleep.sh
