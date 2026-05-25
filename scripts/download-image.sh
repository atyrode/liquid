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

Requires: curl, plus sha256sum or shasum

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

if command -v sha256sum >/dev/null 2>&1; then
  checksum_check() {
    sha256sum -c "$1"
  }
elif command -v shasum >/dev/null 2>&1; then
  checksum_check() {
    shasum -a 256 -c "$1"
  }
else
  echo "Missing required command: sha256sum or shasum" >&2
  exit 1
fi

tmp_base="${TMPDIR:-/tmp}"
tmp_dir="$(mktemp -d "$tmp_base/liquid-download.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

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
asset_urls_file="$tmp_dir/asset-urls"
downloaded_files_file="$tmp_dir/downloaded-files"
part_files_file="$tmp_dir/part-files"
image_files_file="$tmp_dir/image-files"

printf '%s\n' "$release_json" \
  | sed -n 's/^[[:space:]]*"browser_download_url":[[:space:]]*"\([^"]*\)".*/\1/p' \
  | while IFS= read -r url; do
      asset_name="${url##*/}"
      case "$asset_name" in
        "$prefix"*.img.zst|"$prefix"*.img.zst.part-*|"$prefix"*.img.zst.sha256|"$prefix"*.img.zst.reassemble.txt)
          printf '%s\n' "$url"
          ;;
      esac
    done > "$asset_urls_file"

if [ ! -s "$asset_urls_file" ]; then
  echo "No release assets found for image '$image' in $repo release $tag" >&2
  exit 1
fi

: > "$downloaded_files_file"
while IFS= read -r url; do
  asset_name="${url##*/}"
  output_path="$out_dir/$asset_name"
  echo "Downloading: $asset_name"
  curl -fL --retry 3 --retry-delay 2 -o "$output_path" "$url"
  printf '%s\n' "$output_path" >> "$downloaded_files_file"
done < "$asset_urls_file"

sed -n '/[.]img[.]zst[.]part-/p' "$downloaded_files_file" | sort > "$part_files_file"

if [ -s "$part_files_file" ]; then
  first_part="$(sed -n '1p' "$part_files_file")"
  base="${first_part%.part-*}"
  while IFS= read -r part; do
    part_base="${part%.part-*}"
    if [ "$part_base" != "$base" ]; then
      echo "Found split parts for more than one image in $out_dir" >&2
      exit 1
    fi
  done < "$part_files_file"

  echo "Reassembling: $base"
  rm -f "$base"
  while IFS= read -r part; do
    cat "$part"
  done < "$part_files_file" > "$base"
  image_path="$base"
else
  sed -n '/[.]img[.]zst$/p' "$downloaded_files_file" | sort > "$image_files_file"
  image_count="$(wc -l < "$image_files_file" | tr -d ' ')"
  if [ "$image_count" -ne 1 ]; then
    echo "Expected one image file, found $image_count in $out_dir" >&2
    exit 1
  fi
  image_path="$(sed -n '1p' "$image_files_file")"
fi

checksum_path="${image_path}.sha256"
if [ ! -f "$checksum_path" ]; then
  echo "Missing checksum file: $checksum_path" >&2
  exit 1
fi

(
  cd "$(dirname "$image_path")"
  checksum_check "$(basename "$checksum_path")"
)

echo "Ready: $image_path"
