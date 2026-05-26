#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/forget-pi-host-key.sh [host ...]

Removes remembered SSH host keys for a frequently reflashed Raspberry Pi.

Default hosts:
  dogpi.local
  dogpi

Examples:
  scripts/forget-pi-host-key.sh
  scripts/forget-pi-host-key.sh dogpi.local 192.168.1.42
EOF
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

if ! command -v ssh-keygen >/dev/null 2>&1; then
  echo "Missing required command: ssh-keygen" >&2
  exit 1
fi

hosts=("$@")
if [ "${#hosts[@]}" -eq 0 ]; then
  hosts=(dogpi.local dogpi)
fi

for host in "${hosts[@]}"; do
  echo "Forgetting SSH host key: $host"
  ssh-keygen -R "$host" >/dev/null 2>&1 || true
done

cat <<'EOF'
Done.

Retry SSH with:
  ssh artist@dogpi.local

If you previously connected by IP address, rerun this script with that IP too.
EOF
