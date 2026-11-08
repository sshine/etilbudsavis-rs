#!/usr/bin/env bash
# A read-only tour of the live service. Creates one anonymous account in a
# throwaway token file so the account-backed reads have something to query.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

session="$(mktemp)"
trap 'rm -f "$session"' EXIT

A=(cargo run --quiet -p etilbudsavis-cli -- --session-file "$session")

run() {
  local title="$1"
  shift
  echo "=== $title ==="
  "$@" 2>&1 || echo "(failed)"
  echo
}

run "Auth status (before)" "${A[@]}" auth status
run "Create anonymous account" "${A[@]}" auth anonymous
run "Auth status (after)" "${A[@]}" auth status
run "Catalogue groups" "${A[@]}" catalogue list
run "Businesses" "${A[@]}" business list --limit 5
run "One business" "${A[@]}" business show 365discount
run "Offers" "${A[@]}" offer list --limit 8
run "Stores" "${A[@]}" store list --limit 5
run "Top searches" "${A[@]}" search top --limit 10
run "Suggestions" "${A[@]}" search suggestions mælk
run "SEO landing page" "${A[@]}" search seo mælk
run "Videos" "${A[@]}" video list --limit 3
run "Favourites" "${A[@]}" favorites list
run "Shopping lists" "${A[@]}" lists list
run "Fallback location" "${A[@]}" location show
run "Privacy policy heading" "${A[@]}" privacy | head -n 5
