#!/usr/bin/env bash
echo "echo Guten tag from swap/examples/socket/b.sh"
sleep 2
echo 'bash swap/examples/socket/c.sh' | nc -w 1 -U "demo.sock"
echo bye from b
# wait for b to swapped out
exec bash swap/examples/socket/start.sh
