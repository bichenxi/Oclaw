#!/usr/bin/env bash
# scripts/download-fnm.sh
# 下载各平台 fnm 二进制，放到 src-tauri/binaries/fnm-<target-triple>
# Tauri externalBin 约定：文件名 = "<sidecar-name>-<target-triple>"
#
# 用法：
#   bash scripts/download-fnm.sh           # 仅下载当前平台
#   bash scripts/download-fnm.sh --all     # 下载所有平台（CI 用）

set -euo pipefail

FNM_VERSION="${FNM_VERSION:-1.38.1}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEST_DIR="${SCRIPT_DIR}/../src-tauri/binaries"
mkdir -p "$DEST_DIR"

BASE_URL="https://github.com/Schniz/fnm/releases/download/v${FNM_VERSION}"

# ─── 查询 triple 对应的 fnm release 资源名 ────────────────────────────────
get_asset_name() {
  local triple="$1"
  case "$triple" in
    aarch64-apple-darwin)       echo "fnm-macos.zip" ;;
    x86_64-apple-darwin)        echo "fnm-macos.zip" ;;
    x86_64-pc-windows-msvc)     echo "fnm-windows.zip" ;;
    x86_64-unknown-linux-gnu)   echo "fnm-linux.zip" ;;
    aarch64-unknown-linux-gnu)  echo "fnm-arm64.zip" ;;
    *)
      echo "" ;;
  esac
}

# ─── 查询 triple 对应的解压后可执行文件名 ──────────────────────────────────
get_bin_name() {
  local triple="$1"
  case "$triple" in
    *windows*) echo "fnm.exe" ;;
    *)         echo "fnm" ;;
  esac
}

download_triple() {
  local triple="$1"
  local asset
  asset="$(get_asset_name "$triple")"
  if [[ -z "$asset" ]]; then
    echo "[$triple] 不支持的 triple，跳过" >&2
    return 0
  fi

  local bin_name
  bin_name="$(get_bin_name "$triple")"

  local dest_name="fnm-${triple}"
  if [[ "$triple" == *"windows"* ]]; then
    dest_name="${dest_name}.exe"
  fi

  local dest_path="${DEST_DIR}/${dest_name}"

  if [[ -f "$dest_path" ]]; then
    echo "[$triple] 已存在，跳过：$dest_path"
    return 0
  fi

  echo "[$triple] 下载 ${asset} ..."
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  local tmp_zip="${tmp_dir}/fnm_download.zip"

  curl -fsSL --retry 3 "${BASE_URL}/${asset}" -o "$tmp_zip"
  unzip -q "$tmp_zip" -d "$tmp_dir"

  # 在解压目录中查找可执行文件
  local extracted
  extracted="$(find "$tmp_dir" -name "$bin_name" -not -path "*/.__MACOSX/*" -type f | head -1)"

  if [[ -z "$extracted" || ! -f "$extracted" ]]; then
    echo "[$triple] 错误：解压后未找到 ${bin_name}" >&2
    rm -rf "$tmp_dir"
    return 1
  fi

  cp "$extracted" "$dest_path"
  chmod +x "$dest_path"
  rm -rf "$tmp_dir"
  echo "[$triple] 完成：$dest_path"
}

# ─── 检测当前平台 triple ────────────────────────────────────────────────────
detect_host_triple() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "${os}-${arch}" in
    Darwin-arm64)   echo "aarch64-apple-darwin" ;;
    Darwin-x86_64)  echo "x86_64-apple-darwin" ;;
    Linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
    Linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*)  echo "x86_64-pc-windows-msvc" ;;
    *)              echo "" ;;
  esac
}

# ─── 主逻辑 ────────────────────────────────────────────────────────────────
ALL_TRIPLES=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-pc-windows-msvc
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
)

if [[ "${1:-}" == "--all" ]]; then
  for triple in "${ALL_TRIPLES[@]}"; do
    download_triple "$triple"
  done
else
  # 优先使用 RUST_TARGET 环境变量（CI matrix 传入），否则检测本机
  host_triple="${RUST_TARGET:-$(detect_host_triple)}"
  if [[ -z "$host_triple" ]]; then
    echo "无法检测当前平台，请手动设置 RUST_TARGET 环境变量" >&2
    exit 1
  fi
  download_triple "$host_triple"
fi

echo "fnm sidecar 下载完成。"
