#!/usr/bin/env bash
echo "echo Greetings from swap/examples/socket/a.sh"
sleep 1
echo "bash swap/examples/socket/b.sh" | nc -w 1 -U "demo.sock"
echo bye from a
# wait for b to swapped out
exec bash swap/examples/socket/start.sh
