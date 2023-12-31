#!/usr/bin/env bash
echo "[b] Guten tag from examples/socket_bash/b.sh"
sleep 1

# send the next command to swaperooni
echo 'bash examples/socket_bash/c.sh' | nc -w 1 -U "$SOCKET_PATH"

# wait for b to swapped out
exec bash examples/socket_bash/sleep.sh
