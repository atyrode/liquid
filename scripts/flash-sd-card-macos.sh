#!/usr/bin/env bash
set -euo pipefail

image=""
disk=""
assume_yes=0
dry_run=0
list_only=0

usage() {
  cat <<'EOF'
Usage: scripts/flash-sd-card-macos.sh --disk /dev/diskN [options]

Flashes a downloaded Liquid Raspberry Pi image to an SD card on macOS.
This destroys all data on the selected disk.

Options:
  --disk DISK       Whole SD card disk, for example /dev/disk7
  --image PATH      Image to flash. Default: the only liquid-rpi3-lite image in dist
  --list            Show external physical disks and exit
  --yes             Skip the interactive disk-id confirmation
  --dry-run         Print the commands without running them
  -h, --help        Show this help

Examples:
  scripts/download-image.sh
  scripts/flash-sd-card-macos.sh --list
  scripts/flash-sd-card-macos.sh --disk /dev/disk7
  scripts/flash-sd-card-macos.sh --disk /dev/disk7 --image dist/liquid-rpi3-lite-2026-05-26-56f427e.img.zst
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --disk)
      disk="${2:?missing value for --disk}"
      shift 2
      ;;
    --image)
      image="${2:?missing value for --image}"
      shift 2
      ;;
    --list)
      list_only=1
      shift
      ;;
    --yes)
      assume_yes=1
      shift
      ;;
    --dry-run)
      dry_run=1
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

need_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  if [ "$dry_run" -eq 0 ]; then
    "$@"
  fi
}

find_default_image() {
  local found=""
  local count=0
  local candidate=""

  for candidate in dist/liquid-rpi3-lite-*.img.zst dist/liquid-rpi3-lite-*.img.xz dist/liquid-rpi3-lite-*.img; do
    if [ ! -e "$candidate" ]; then
      continue
    fi
    found="$candidate"
    count=$((count + 1))
  done

  if [ "$count" -eq 0 ]; then
    echo "No image found in dist. Run scripts/download-image.sh first or pass --image." >&2
    exit 1
  fi

  if [ "$count" -gt 1 ]; then
    echo "More than one image found in dist. Pass --image explicitly." >&2
    exit 1
  fi

  printf '%s\n' "$found"
}

normalize_disk() {
  local input="$1"

  case "$input" in
    /dev/disk*)
      printf '%s\n' "$input"
      ;;
    /dev/rdisk*)
      printf '/dev/%s\n' "${input#/dev/r}"
      ;;
    *)
      echo "Disk must look like /dev/diskN, for example /dev/disk7." >&2
      exit 1
      ;;
  esac
}

print_external_disks() {
  diskutil list external physical
}

need_command diskutil

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script is for macOS only." >&2
  exit 1
fi

if [ "$list_only" -eq 1 ]; then
  print_external_disks
  exit 0
fi

if [ -z "$disk" ]; then
  echo "Missing --disk." >&2
  echo >&2
  print_external_disks >&2
  echo >&2
  usage >&2
  exit 2
fi

if [ -z "$image" ]; then
  image="$(find_default_image)"
fi

if [ ! -f "$image" ]; then
  echo "Image not found: $image" >&2
  exit 1
fi

case "$image" in
  *.img.zst)
    need_command zstd
    ;;
  *.img.xz)
    need_command xz
    ;;
  *.img)
    ;;
  *)
    echo "Unsupported image type: $image" >&2
    echo "Expected .img, .img.zst, or .img.xz." >&2
    exit 1
    ;;
esac

whole_disk="$(normalize_disk "$disk")"
if [[ ! "$whole_disk" =~ ^/dev/disk[0-9]+$ ]]; then
  echo "Refusing to flash a partition or unusual disk path: $whole_disk" >&2
  echo "Use the whole disk path, for example /dev/disk7, not /dev/disk7s1." >&2
  exit 1
fi

raw_disk="/dev/r${whole_disk#/dev/}"
disk_id="${whole_disk#/dev/}"

disk_info="$(diskutil info "$whole_disk")"
if ! printf '%s\n' "$disk_info" | grep -Eq '^[[:space:]]*Whole:[[:space:]]*Yes$'; then
  echo "Refusing to flash because diskutil does not report this as a whole disk: $whole_disk" >&2
  exit 1
fi

if printf '%s\n' "$disk_info" | grep -Eq '^[[:space:]]*Internal:[[:space:]]*Yes$'; then
  echo "Refusing to flash an internal disk: $whole_disk" >&2
  exit 1
fi

echo "Target disk:"
diskutil list "$whole_disk"
echo
echo "Image: $image"
echo "Write device: $raw_disk"
echo
echo "This will destroy all data on $whole_disk."

if [ "$assume_yes" -eq 0 ]; then
  printf 'Type %s to continue: ' "$disk_id"
  read -r reply
  if [ "$reply" != "$disk_id" ]; then
    echo "Aborted."
    exit 1
  fi
fi

run diskutil unmountDisk "$whole_disk"

case "$image" in
  *.img.zst)
    echo "+ zstd -dc $(printf '%q' "$image") | sudo dd of=$(printf '%q' "$raw_disk") bs=4m"
    if [ "$dry_run" -eq 0 ]; then
      zstd -dc "$image" | sudo dd of="$raw_disk" bs=4m
    fi
    ;;
  *.img.xz)
    echo "+ xz -dc $(printf '%q' "$image") | sudo dd of=$(printf '%q' "$raw_disk") bs=4m"
    if [ "$dry_run" -eq 0 ]; then
      xz -dc "$image" | sudo dd of="$raw_disk" bs=4m
    fi
    ;;
  *.img)
    run sudo dd "if=$image" "of=$raw_disk" bs=4m
    ;;
esac

run sync
run diskutil eject "$whole_disk"

echo "Done. Insert the SD card into the Raspberry Pi and boot it."
