#!/usr/bin/env bash
set -euo pipefail

repo_dir="${LIQUID_REPO_DIR:-$HOME/liquid}"
loader="$repo_dir/image/files/home/artist/.liquid-shell.zsh"

if [ ! -f "$loader" ]; then
  echo "Missing shell loader in repo checkout: $loader" >&2
  exit 1
fi

install -m 0644 "$loader" "$HOME/.liquid-shell.zsh"

cat <<'EOF'
Synced ~/.liquid-shell.zsh.

The shell loader now prefers modules from:
  ~/liquid/image/files/home/artist/.liquid-shell.d

Reload with:
  zconf
EOF
