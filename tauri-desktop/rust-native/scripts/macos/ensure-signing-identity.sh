#!/usr/bin/env bash
# Verify a stable macOS code-signing identity exists. Never creates or regenerates
# a certificate — regenerating would invalidate every TCC grant tied to the old leaf.
set -euo pipefail

DEFAULT_IDENTITY="ReadyType Code Signing"
IDENTITY="${APPLE_SIGNING_IDENTITY:-$DEFAULT_IDENTITY}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DOCS="$(cd "${SCRIPT_DIR}/../../../../docs" && pwd)/macos-code-signing.md"

# Do not use `security find-identity -v`: self-signed roots are often hidden as
# CSSMERR_TP_NOT_TRUSTED even though codesign can still use them.
if ! security find-identity -p codesigning 2>/dev/null | grep -F "\"${IDENTITY}\"" >/dev/null; then
  cat >&2 <<EOF
error: code-signing identity not found in keychain: ${IDENTITY}

Create it once (never regenerate) — see:
  ${REPO_DOCS}

Quick check (no -v):
  security find-identity -p codesigning | grep '${IDENTITY}'
EOF
  exit 1
fi

printf '%s\n' "${IDENTITY}"
