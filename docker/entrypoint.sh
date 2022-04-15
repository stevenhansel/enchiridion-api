#!/bin/bash -e

set -eu

MAIN_BINARY=$1

echo "[`date`] Running entrypoint script"
echo "[`date`] Running $MAIN_BINARY with ${@:2}"

prog=/app/$MAIN_BINARY

if ! [ -e "$prog" ]; then
  echo "$MAIN_BINARY is not available" >&2
  exit 1
fi

exec "$prog" "${@:2}"
