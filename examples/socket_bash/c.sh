#!/usr/bin/env bash
echo "[c] Bienvenidos de examples/socket_bash/c.sh"
echo 'echo finished.' | nc -w 1 -U "$SOCKET_PATH"
