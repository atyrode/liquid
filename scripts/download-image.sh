#!/usr/bin/env bash
set -euo pipefail

repo="atyrode/liquid"
tag=""
image="pi3a-gui-image"
out_dir="dist"

usage() {
  cat <<'EOF'
Usage: scripts/download-image.sh [options]

Downloads a Raspberry Pi image from a GitHub Release, reassembles split parts
when needed, and verifies the published SHA256 checksum.

Options:
  --repo OWNER/REPO   GitHub repository. Default: atyrode/liquid
  --tag TAG           Release tag. Default: latest release
  --image NAME        Image package name. Default: pi3a-gui-image
                     Common values: pi3a-gui-image, pi3a-image
  --dir DIR           Output directory. Default: dist
  -h, --help          Show this help

Examples:
  scripts/download-image.sh
  scripts/download-image.sh --tag pi3a-gui-lite-2026-05-25
  scripts/download-image.sh --image pi3a-image --dir dist/headless
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      repo="${2:?missing value for --repo}"
      shift 2
      ;;
    --tag)
      tag="${2:?missing value for --tag}"
      shift 2
      ;;
    --image)
      image="${2:?missing value for --image}"
      shift 2
      ;;
    --dir)
      out_dir="${2:?missing value for --dir}"
      shift 2
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

need_command gh
need_command sha256sum

if [ -z "$tag" ]; then
  tag="$(gh release view --repo "$repo" --json tagName --jq .tagName)"
fi

mkdir -p "$out_dir"

echo "Repository: $repo"
echo "Release: $tag"
echo "Image: $image"
echo "Output: $out_dir"

gh release download "$tag" \
  --repo "$repo" \
  --pattern "liquid-${image}-*.img.zst*" \
  --dir "$out_dir" \
  --clobber

mapfile -t part_files < <(find "$out_dir" -maxdepth 1 -type f -name "liquid-${image}-*.img.zst.part-*" | sort)

if [ "${#part_files[@]}" -gt 0 ]; then
  base="${part_files[0]%.part-*}"
  for part in "${part_files[@]}"; do
    part_base="${part%.part-*}"
    if [ "$part_base" != "$base" ]; then
      echo "Found split parts for more than one image in $out_dir" >&2
      exit 1
    fi
  done

  echo "Reassembling: $base"
  rm -f "$base"
  cat "${part_files[@]}" > "$base"
  image_path="$base"
else
  mapfile -t image_files < <(find "$out_dir" -maxdepth 1 -type f -name "liquid-${image}-*.img.zst" | sort)
  if [ "${#image_files[@]}" -ne 1 ]; then
    echo "Expected one image file, found ${#image_files[@]} in $out_dir" >&2
    exit 1
  fi
  image_path="${image_files[0]}"
fi

checksum_path="${image_path}.sha256"
if [ ! -f "$checksum_path" ]; then
  echo "Missing checksum file: $checksum_path" >&2
  exit 1
fi

(
  cd "$(dirname "$image_path")"
  sha256sum -c "$(basename "$checksum_path")"
)

echo "Ready: $image_path"
