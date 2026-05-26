#!/usr/bin/env bash
set -euo pipefail

assume_yes=0
repo_dir="${LIQUID_REPO_DIR:-$HOME/liquid}"
control_dir="${LIQUID_CONTROL_DIR:-$HOME/liquid-control}"
bin_src="$repo_dir/image/files/usr/local/bin"
control_src="$repo_dir/image/files/home/artist/liquid-control"

usage() {
  cat <<'EOF'
Usage: scripts/sync-pi-runtime.sh [--yes]

Syncs the repo-owned Raspberry Pi runtime commands into an already-flashed Pi.

It installs the /usr/local/bin Liquid commands with sudo, then links
~/liquid-control command wrappers back to the repo checkout so future git pulls
can update them. It preserves ~/liquid-control/settings.env.

Options:
  --yes, -y   Run without the confirmation prompt.
  -h, --help  Show this help.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --yes|-y)
      assume_yes=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ ! -d "$repo_dir/.git" ]; then
  echo "Liquid repo checkout is missing: $repo_dir" >&2
  exit 1
fi

if [ ! -d "$bin_src" ] || [ ! -d "$control_src" ]; then
  echo "Liquid runtime assets are missing under: $repo_dir/image/files" >&2
  exit 1
fi

if [ "$assume_yes" -eq 0 ] && [ ! -t 0 ]; then
  echo "Refusing to run interactively without a terminal. Pass --yes after reviewing the script." >&2
  exit 1
fi

if [ "$(id -u)" -eq 0 ]; then
  root_cmd=()
elif command -v sudo >/dev/null 2>&1; then
  root_cmd=(sudo)
else
  echo "Run as root or install sudo so /usr/local/bin commands can be updated." >&2
  exit 1
fi

run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

confirm() {
  if [ "$assume_yes" -eq 1 ]; then
    return 0
  fi

  cat <<EOF
This will:
- install repo versions of liquid-* runtime commands into /usr/local/bin
- link ~/liquid-control command scripts and README.md back to the repo checkout
- preserve ~/liquid-control/settings.env
- move any existing non-symlink control script to *.local-before-repo-link
- sync ~/.liquid-shell.zsh so zconf can load shell modules from the repo

Repo:    $repo_dir
Control: $control_dir
EOF

  printf 'Proceed? [y/N] '
  read -r answer
  case "$answer" in
    y|Y|yes|YES)
      ;;
    *)
      echo "Aborted."
      exit 0
      ;;
  esac
}

link_control_file() {
  name="$1"
  src="$control_src/$name"
  dest="$control_dir/$name"
  backup="$dest.local-before-repo-link"

  if [ ! -e "$src" ]; then
    echo "Missing control source: $src" >&2
    exit 1
  fi

  if [ -e "$dest" ] && [ ! -L "$dest" ]; then
    if [ -e "$backup" ]; then
      echo "Refusing to replace $dest because backup already exists: $backup" >&2
      exit 1
    fi
    run mv "$dest" "$backup"
  fi

  run ln -sfn "$src" "$dest"
}

confirm

for name in \
  liquid-bluetooth-keyboard \
  liquid-restart \
  liquid-run-terminal \
  liquid-start \
  liquid-update
do
  run "${root_cmd[@]}" install -m 0755 "$bin_src/$name" "/usr/local/bin/$name"
done

run mkdir -p "$control_dir"

if [ ! -f "$control_dir/settings.env" ]; then
  run install -m 0644 "$control_src/settings.env" "$control_dir/settings.env"
fi

for name in \
  README.md \
  attach \
  bluetooth \
  config \
  doctor \
  restart \
  start \
  stop \
  update
do
  link_control_file "$name"
done

if [ -x "$repo_dir/scripts/sync-pi-shell-loader.sh" ]; then
  run "$repo_dir/scripts/sync-pi-shell-loader.sh"
fi

cat <<'EOF'

Runtime sync complete.

Reload the shell config with:

  zconf

Apply renderer settings with:

  ~/liquid-control/restart
EOF
