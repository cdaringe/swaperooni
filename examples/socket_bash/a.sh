#!/usr/bin/env bash
echo "[a] Greetings from examples/socket_bash/a.sh"
sleep 1

# send the next command to swaperooni
echo "bash examples/socket_bash/b.sh" | nc -w 1 -U "$SOCKET_PATH"

# wait for a to swapped out
exec bash examples/socket_bash/sleep.sh
