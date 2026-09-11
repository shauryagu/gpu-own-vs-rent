#!/bin/sh
set -eu

# Collect writes here when the caller passes --data-dir /data.
mkdir -p "${CHI_DATA_DIR:-/data}" 2>/dev/null || true

if [ "$#" -eq 0 ]; then
  set -- --help
fi

exec chi "$@"
