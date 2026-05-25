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

Requires: curl, sha256sum

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

need_command curl
need_command sha256sum

github_api="${GITHUB_API_URL:-https://api.github.com}"
release_api="$github_api/repos/$repo/releases"

curl_github() {
  curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "$1"
}

if [ -z "$tag" ]; then
  release_json="$(curl_github "$release_api/latest")"
  tag="$(printf '%s\n' "$release_json" | sed -n 's/^[[:space:]]*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
  if [ -z "$tag" ]; then
    echo "Could not determine latest release tag for $repo" >&2
    exit 1
  fi
else
  release_json="$(curl_github "$release_api/tags/$tag")"
fi

mkdir -p "$out_dir"

echo "Repository: $repo"
echo "Release: $tag"
echo "Image: $image"
echo "Output: $out_dir"

prefix="liquid-${image}-"
mapfile -t asset_urls < <(
  printf '%s\n' "$release_json" \
    | sed -n 's/^[[:space:]]*"browser_download_url":[[:space:]]*"\([^"]*\)".*/\1/p' \
    | while IFS= read -r url; do
        asset_name="${url##*/}"
        case "$asset_name" in
          "$prefix"*.img.zst|"$prefix"*.img.zst.part-*|"$prefix"*.img.zst.sha256|"$prefix"*.img.zst.reassemble.txt)
            printf '%s\n' "$url"
            ;;
        esac
      done
)

if [ "${#asset_urls[@]}" -eq 0 ]; then
  echo "No release assets found for image '$image' in $repo release $tag" >&2
  exit 1
fi

downloaded_files=()
for url in "${asset_urls[@]}"; do
  asset_name="${url##*/}"
  output_path="$out_dir/$asset_name"
  echo "Downloading: $asset_name"
  curl -fL --retry 3 --retry-delay 2 -o "$output_path" "$url"
  downloaded_files+=("$output_path")
done

mapfile -t part_files < <(printf '%s\n' "${downloaded_files[@]}" | sed -n '/[.]img[.]zst[.]part-/p' | sort)

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
  mapfile -t image_files < <(printf '%s\n' "${downloaded_files[@]}" | sed -n '/[.]img[.]zst$/p' | sort)
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
