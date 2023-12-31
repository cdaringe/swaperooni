#!/usr/bin/env bash
echo "[c] Bienvenidos de examples/ipc_bash_hopscotch/c.sh"
echo 'echo finished.' | nc -w 1 -U "$SOCKET_PATH"
