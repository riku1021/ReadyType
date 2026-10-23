#!/usr/bin/env python3
"""Build Tauri's static updater manifest from staged signature files.

The desktop workflow uploads installers under stable names. This script reads
the matching `.sig` files and writes `latest.json` for
`releases/latest/download/latest.json`. Missing signatures skip that platform.
When none are present, no file is written.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ASSETS = (
    {
        "signature": "ReadyType-aarch64.app.tar.gz.sig",
        "asset": "ReadyType-aarch64.app.tar.gz",
        "platforms": ("darwin-aarch64", "darwin-aarch64-app"),
    },
    {
        "signature": "ReadyType-x64.app.tar.gz.sig",
        "asset": "ReadyType-x64.app.tar.gz",
        "platforms": ("darwin-x86_64", "darwin-x86_64-app"),
    },
    {
        "signature": "ReadyType-x64-setup.exe.sig",
        "asset": "ReadyType-x64-setup.exe",
        "platforms": ("windows-x86_64", "windows-x86_64-nsis"),
    },
    {
        "signature": "ReadyType-x64.AppImage.sig",
        "asset": "ReadyType-x64.AppImage",
        "platforms": ("linux-x86_64", "linux-x86_64-appimage"),
    },
)


def build_manifest(
    assets_dir: Path, version: str, download_base: str, pub_date: str
) -> dict[str, object] | None:
    platforms: dict[str, dict[str, str]] = {}
    base = download_base.rstrip("/")
    for entry in ASSETS:
        signature_path = assets_dir / str(entry["signature"])
        if not signature_path.is_file():
            continue
        signature = signature_path.read_text(encoding="utf-8").strip()
        if signature == "":
            raise SystemExit(f"empty updater signature: {signature_path}")
        asset = str(entry["asset"])
        payload = {"signature": signature, "url": f"{base}/{asset}"}
        for platform in entry["platforms"]:
            platforms[str(platform)] = payload
    if not platforms:
        return None
    return {
        "version": version,
        "notes": f"ReadyType {version}",
        "pub_date": pub_date,
        "platforms": platforms,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--assets-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--download-base", required=True)
    parser.add_argument("--pub-date")
    args = parser.parse_args()
    pub_date = args.pub_date or datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    manifest = build_manifest(args.assets_dir, args.version, args.download_base, pub_date)
    if manifest is None:
        print("no updater signatures; skipping latest.json", file=sys.stderr)
        return 0
    args.output.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
