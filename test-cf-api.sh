#!/usr/bin/env bash
set -euo pipefail

# Load secrets from worker/.env
ENV_FILE="$(dirname "$0")/worker/.env"
if [[ ! -f "$ENV_FILE" ]]; then
  echo "ERROR: $ENV_FILE not found" >&2
  exit 1
fi
# shellcheck disable=SC1090
set -o allexport; source "$ENV_FILE"; set +o allexport

: "${CF_API_TOKEN:?CF_API_TOKEN not set in .env}"
: "${CF_ZONE_ID:?CF_ZONE_ID not set in .env}"

BASE="https://api.cloudflare.com/client/v4"

echo "=== 1. GET /zones/{zone_id} (requires Zone:Read) ==="
curl -s \
  -H "Authorization: Bearer $CF_API_TOKEN" \
  "$BASE/zones/$CF_ZONE_ID" \
  | python3 -m json.tool

echo ""
echo "=== 2. GET /zones/{zone_id}/email/routing/rules (requires Email Routing:Read) ==="
curl -s \
  -H "Authorization: Bearer $CF_API_TOKEN" \
  "$BASE/zones/$CF_ZONE_ID/email/routing/rules" \
  | python3 -m json.tool
