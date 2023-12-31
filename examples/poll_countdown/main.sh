#!/usr/bin/env bash
set -o pipefail
i=${1:-10}
while [ $i != "0" ]; do
  echo "Change this text! $i"
  sleep 1
  # bash could not possibly be a worse scripting language.
  # `expr 1 - 1` exits with code 1
  # hence, for this tiny script, we turn of -e. f
  i=$(expr $i - 1)
done
echo "no change detected, exiting"
