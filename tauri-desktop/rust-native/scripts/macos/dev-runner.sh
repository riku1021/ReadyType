#!/usr/bin/env bash
# Tauri build.runner replacement for macOS: cargo build|run with stable codesign.
# Tauri invokes: <runner> run|build [cargo-args...] [-- app-args...]
# See crates/tauri-cli/src/interface/rust/desktop.rs cargo_command().
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 run|build [cargo-args...] [-- app-args...]" >&2
  exit 2
fi

SUBCOMMAND="$1"
shift

BIN_NAME="tauri-desktop"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENSURE="${SCRIPT_DIR}/ensure-signing-identity.sh"
TAURI_CONF="${SCRIPT_DIR}/../../tauri.conf.json"
BUNDLE_ID="$(
  python3 -c 'import json,sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["identifier"])' \
    "${TAURI_CONF}"
)"

cargo_args=()
app_args=()
seeing_app=0
profile="debug"
target_triple=""
next_is_target=0
next_is_profile=0

for arg in "$@"; do
  if [[ "${seeing_app}" -eq 1 ]]; then
    app_args+=("${arg}")
    continue
  fi
  if [[ "${arg}" == "--" ]]; then
    seeing_app=1
    continue
  fi
  cargo_args+=("${arg}")

  if [[ "${next_is_target}" -eq 1 ]]; then
    target_triple="${arg}"
    next_is_target=0
    continue
  fi
  if [[ "${next_is_profile}" -eq 1 ]]; then
    profile="${arg}"
    next_is_profile=0
    continue
  fi

  case "${arg}" in
    --release)
      profile="release"
      ;;
    --target)
      next_is_target=1
      ;;
    --target=*)
      target_triple="${arg#--target=}"
      ;;
    --profile)
      next_is_profile=1
      ;;
    --profile=*)
      profile="${arg#--profile=}"
      ;;
  esac
done

if [[ "${SUBCOMMAND}" != "run" && "${SUBCOMMAND}" != "build" ]]; then
  exec cargo "${SUBCOMMAND}" "$@"
fi

# externalBin 用 sidecar を先に用意（tauri_build の存在チェック対策を含む）。
# cargo_args（Tauri 由来の --release 等）は渡さない。二重指定で cargo が失敗する。
PREPARE="${SCRIPT_DIR}/prepare-tis-list-sidecar.sh"
prepare_args=()
if [[ "${profile}" == "release" ]]; then
  prepare_args+=(--release)
fi
# Tauri の `--target` と sidecar の triple を揃える（Intel 向けクロスコンパイル）。
if [[ -n "${target_triple}" ]]; then
  prepare_args+=(--target "${target_triple}")
fi
bash "${PREPARE}" ${prepare_args[@]+"${prepare_args[@]}"}

# Always build first so we can sign before launch (cargo run would exec unsigned).
cargo build "${cargo_args[@]+"${cargo_args[@]}"}" --bin "${BIN_NAME}"

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
if [[ -n "${target_triple}" ]]; then
  BIN_PATH="${TARGET_DIR}/${target_triple}/${profile}/${BIN_NAME}"
  HELPER_PATH="${TARGET_DIR}/${target_triple}/${profile}/tis-list-enabled"
else
  BIN_PATH="${TARGET_DIR}/${profile}/${BIN_NAME}"
  HELPER_PATH="${TARGET_DIR}/${profile}/tis-list-enabled"
fi

if [[ ! -x "${BIN_PATH}" ]]; then
  echo "error: expected binary not found or not executable: ${BIN_PATH}" >&2
  exit 1
fi

if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  IDENTITY="${APPLE_SIGNING_IDENTITY}"
else
  IDENTITY="$("${ENSURE}")"
  export APPLE_SIGNING_IDENTITY="${IDENTITY}"
fi

echo "codesign: signing ${BIN_PATH} with \"${IDENTITY}\"" >&2
codesign --force --sign "${IDENTITY}" --identifier "${BUNDLE_ID}" "${BIN_PATH}"
if [[ -x "${HELPER_PATH}" ]]; then
  echo "codesign: signing ${HELPER_PATH} with \"${IDENTITY}\"" >&2
  codesign --force --sign "${IDENTITY}" --identifier "${BUNDLE_ID}.tis-list-enabled" "${HELPER_PATH}"
fi

case "${SUBCOMMAND}" in
  build)
    exit 0
    ;;
  run)
    exec "${BIN_PATH}" "${app_args[@]+"${app_args[@]}"}"
    ;;
esac
