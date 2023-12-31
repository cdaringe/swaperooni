#!/usr/bin/env bash
echo "echo Bienvenidos de swap/examples/socket/c.sh"
echo 'echo finito!' | nc -w 1 -U "demo.sock"
