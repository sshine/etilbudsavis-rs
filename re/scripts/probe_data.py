#!/usr/bin/env python3
"""Probe the eTilbudsavis data endpoint and summarise response shapes.

Generated output belongs under secrets/probes/ (gitignored): responses can
contain a location, account data, or other personal values.
"""

import base64
import json
import pathlib
import sys
import urllib.request

BASE = "https://etilbudsavis.tjek.com/"
KEY = "590990e7ad62f206512ce390939ea3d3"
OUT = pathlib.Path(__file__).resolve().parents[2] / "secrets" / "probes"


def data_key(key, params):
    raw = json.dumps([key, params], separators=(",", ":")).encode()
    return base64.b64encode(raw).decode()


def call(entries, token=None):
    payload = json.dumps({"data": [data_key(k, p) for k, p in entries]}).encode()
    headers = {
        "content-type": "application/json",
        "x-api-key": KEY,
        "user-agent": "etilbudsavis-rs-probe/0.1",
    }
    if token:
        headers["x-token"] = token
    req = urllib.request.Request(BASE, data=payload, headers=headers, method="POST")
    with urllib.request.urlopen(req, timeout=60) as response:
        body = response.read().decode()
        status = response.status
    return status, body


def summarise(value, depth=0, max_depth=3):
    if depth >= max_depth:
        return type(value).__name__
    if isinstance(value, dict):
        return {k: summarise(v, depth + 1, max_depth) for k, v in value.items()}
    if isinstance(value, list):
        if not value:
            return []
        return [summarise(value[0], depth + 1, max_depth)]
    if isinstance(value, str):
        return f"str({value[:40]!r})" if len(value) < 60 else f"str[len={len(value)}]"
    return value


def run(name, entries):
    OUT.mkdir(parents=True, exist_ok=True)
    try:
        status, body = call(entries)
    except Exception as error:  # noqa: BLE001 - a probe reports, it does not handle
        print(f"\n### {name}: request failed: {error}")
        return
    path = OUT / f"{name}.ndjson"
    path.write_text(body)
    print(f"\n### {name}: HTTP {status}, {len(body)} bytes -> {path.name}")
    for line in body.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            print("  non-json line:", line[:200])
            continue
        print("  key:", record.get("key"), "status:", record.get("status"))
        if record.get("error"):
            print("  error:", json.dumps(record["error"])[:500])
        value = record.get("value")
        if value is not None:
            print("  shape:", json.dumps(summarise(value), indent=2)[:4000])


def main():
    probes = [
        ("groupedFronts", [("groupedFronts", None)]),
        ("businesses", [("businesses", {})]),
        ("business", [("business", {"slug": "Wolt-Market"})]),
        ("fronts", [("fronts", {"businessSlugs": ["Wolt-Market"]})]),
        ("offers", [("offers", {"pagination": {"limit": 3, "offset": 0}, "sources": ["publication", "business_product"], "sort": ["popularity_desc"]})]),
        ("topSearches", [("topSearches", {"pagination": {"limit": 3, "offset": 0}})]),
        ("stores", [("stores", {"pagination": {"limit": 3, "offset": 0}})]),
        ("videos", [("videos", {"limit": 3})]),
        ("privacyPolicy", [("privacyPolicy", {"locale": "da"})]),
        ("location", [("location", None)]),
        ("locationInfo", [("locationInfo", {"latitude": 55.6761, "longitude": 12.5683})]),
        ("searchBusiness", [("searchBusiness", {"searchTerm": "wolt"})]),
    ]
    if len(sys.argv) > 1:
        wanted = set(sys.argv[1:])
        probes = [probe for probe in probes if probe[0] in wanted]
    for name, entries in probes:
        run(name, entries)


if __name__ == "__main__":
    main()
