#!/usr/bin/env bash
# Build `tis-list-enabled` and place it at
#   target/sidecar/tis-list-enabled-$TARGET_TRIPLE
# for Tauri `bundle.externalBin`.
#
# `binaries/` 配下は Tauri のソース監視に入り再ビルドループになるため使わない。
# tauri_build は externalBin の存在を compile 前に要求するので、未作成時のみ
# プレースホルダを書き、実体は内容が変わったときだけ上書きする。
#
# 呼び出し元: macOS `dev-runner.sh`（dev / build 共通）。build.rs がプレースホルダを補う。
# `--target <triple>` はクロスコンパイル用（CI の Intel Mac など）。
# 省略時はホスト triple。出力は常に `target/sidecar/tis-list-enabled-$TRIPLE`。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${ROOT}"

PROFILE="debug"
CARGO_PROFILE_ARGS=()
TARGET_TRIPLE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      PROFILE="release"
      CARGO_PROFILE_ARGS=(--release)
      shift
      ;;
    --target)
      if [[ $# -lt 2 || -z "${2}" ]]; then
        echo "usage: $0 [--release] [--target <triple>]" >&2
        exit 2
      fi
      TARGET_TRIPLE="$2"
      shift 2
      ;;
    *)
      echo "usage: $0 [--release] [--target <triple>]" >&2
      exit 2
      ;;
  esac
done

# `rustc -Vv | awk … exit` は pipefail 下で SIGPIPE(141) になるので使わない。
# Tauri docs も `rustc --print host-tuple`（Rust 1.84+）を推奨。
if [[ -n "${TARGET_TRIPLE}" ]]; then
  TRIPLE="${TARGET_TRIPLE}"
else
  TRIPLE="$(rustc --print host-tuple 2>/dev/null || true)"
  if [[ -z "${TRIPLE}" ]]; then
    TRIPLE="$(rustc -Vv | sed -n 's/^host: //p')"
  fi
fi
if [[ -z "${TRIPLE}" ]]; then
  echo "error: failed to detect rustc host triple" >&2
  exit 1
fi

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
SIDECAR_DIR="${TARGET_DIR}/sidecar"
mkdir -p "${SIDECAR_DIR}"
DEST="${SIDECAR_DIR}/tis-list-enabled-${TRIPLE}"

# Satisfy tauri_build's externalBin existence check during compile.
if [[ ! -e "${DEST}" ]]; then
  printf '%s\n' '#!/bin/sh' 'echo "tis-list-enabled placeholder" >&2' 'exit 1' >"${DEST}"
  chmod +x "${DEST}"
fi

CARGO_TARGET_ARGS=()
if [[ -n "${TARGET_TRIPLE}" ]]; then
  CARGO_TARGET_ARGS=(--target "${TRIPLE}")
fi
cargo build --bin tis-list-enabled \
  "${CARGO_PROFILE_ARGS[@]+"${CARGO_PROFILE_ARGS[@]}"}" \
  "${CARGO_TARGET_ARGS[@]+"${CARGO_TARGET_ARGS[@]}"}"

# `--target` 付きの成果物は `target/<triple>/<profile>/` に出る。
if [[ -n "${TARGET_TRIPLE}" ]]; then
  SRC="${TARGET_DIR}/${TRIPLE}/${PROFILE}/tis-list-enabled"
else
  SRC="${TARGET_DIR}/${PROFILE}/tis-list-enabled"
fi
if [[ ! -x "${SRC}" ]]; then
  echo "error: expected helper not found: ${SRC}" >&2
  exit 1
fi

# 同一内容なら触らない（mtime 更新 → Tauri 再ビルドループ防止）
if cmp -s "${SRC}" "${DEST}"; then
  echo "tis-list-enabled sidecar: ${DEST} (unchanged)"
  exit 0
fi

cp -f "${SRC}" "${DEST}"
chmod +x "${DEST}"
echo "tis-list-enabled sidecar: ${DEST}"
