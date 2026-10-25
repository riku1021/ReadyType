#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
region="${AWS_REGION:-ap-southeast-2}"
bucket="${AWS_S3_BUCKET:-}"

if [[ "$region" != "ap-southeast-2" ]]; then
  echo "AWS_REGION must be ap-southeast-2" >&2
  exit 1
fi

if [[ ! "$bucket" =~ ^readytype-lp-prod-[0-9]{12}-ap-southeast-2-an$ ]]; then
  echo "AWS_S3_BUCKET must use the readytype-lp-prod account-regional name in ap-southeast-2" >&2
  exit 1
fi

for command_name in pnpm aws; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command not found: $command_name" >&2
    exit 1
  fi
done

cd "$repo_root/astro-lp"
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
pnpm build

aws s3 sync dist/ "s3://${bucket}/" \
  --region "$region" \
  --exclude "_astro/*" \
  --delete \
  --cache-control "no-cache, max-age=0, must-revalidate"

aws s3 sync dist/_astro/ "s3://${bucket}/_astro/" \
  --region "$region" \
  --cache-control "public, max-age=31536000, immutable"
