"""Probe the read routes in endpoints.yaml.

The client id comes from SC_CLIENT_ID or from the web player. It is never written.
"""

import argparse
import json
import os
import re
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

from client_id import discover

try:
    import yaml
except ImportError:
    sys.exit("pyyaml is required: python -m pip install pyyaml")

ROOT = Path(__file__).resolve().parent.parent
TRACK_ID = "584692665"
TRACK_IDS = "584692665,2324995076"
USER_ID = "288939218"
PLAYLIST_ID = "2233110584"
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Accept": "application/json, text/plain, */*",
    "Accept-Language": "en-US,en;q=0.9",
    "Origin": "https://soundcloud.com",
    "Referer": "https://soundcloud.com/",
}
QUERY_VALUES = {
    "q": "ambient",
    "limit": "5",
    "offset": "0",
    "threaded": "1",
    "kind": "trending",
    "genre": "soundcloud:genres:all-music",
    "linked_partitioning": "1",
}


def values():
    return {
        "track_id": TRACK_ID,
        "track_ids": TRACK_IDS,
        "user_id": USER_ID,
        "playlist_id": PLAYLIST_ID,
        "media_path": "",
        "resolve_url": "",
    }


def fill_path(path, current):
    for key, value in current.items():
        path = path.replace("{" + key + "}", str(value))
    return path


def query_value(key, current):
    if key == "ids":
        return current["track_ids"]
    if key == "url":
        return current["resolve_url"] or None
    return QUERY_VALUES.get(key)


def build_url(entry, current, client_id):
    path = fill_path(entry["path"], current)
    pairs = []
    for key in entry.get("query", []):
        if key == "client_id":
            continue
        value = query_value(key, current)
        if value is not None and value != "":
            pairs.append((key, value))
    pairs.append(("client_id", client_id))
    return "https://" + entry["host"] + path + "?" + urllib.parse.urlencode(pairs)


def send(entry, current, client_id, timeout, context):
    url = build_url(entry, current, client_id)
    headers = dict(HEADERS)
    data = None
    if entry["method"] == "POST":
        headers["Content-Type"] = "application/json"
        data = b"{}"
    request = urllib.request.Request(
        url,
        data=data,
        headers=headers,
        method=entry["method"],
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout, context=context) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read()
    except Exception as exc:
        return None, type(exc).__name__


def remember_track(entry, status, body, current):
    if entry["id"] != "track" or status != 200:
        return
    try:
        track = json.loads(body)
        current["resolve_url"] = track.get("permalink_url") or ""
        transcodings = (track.get("media") or {}).get("transcodings") or []
        if transcodings:
            path = urllib.parse.urlsplit(transcodings[0].get("url", "")).path
            current["media_path"] = path.removeprefix("/media/")
    except (ValueError, TypeError, AttributeError):
        return


def rewrite_statuses(path, updates):
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    entry_pattern = re.compile(r"^(\s*-\s+id:\s*)([A-Za-z0-9_-]+)(\s*)(\r?\n)?$")
    status_pattern = re.compile(r"^(\s*status:\s*)(\d+)(\s*)(\r?\n)?$")
    current = None
    found = set()
    for index, line in enumerate(lines):
        entry_match = entry_pattern.match(line)
        if entry_match:
            current = entry_match.group(2)
            continue
        if current not in updates:
            continue
        status_match = status_pattern.match(line)
        if not status_match:
            continue
        lines[index] = (
            status_match.group(1)
            + str(updates[current])
            + status_match.group(3)
            + (status_match.group(4) or "")
        )
        found.add(current)
    missing = set(updates) - found
    if missing:
        sys.exit("could not find status line for: " + ", ".join(sorted(missing)))
    new_text = "".join(lines)
    if new_text != text:
        with path.open("w", encoding="utf-8", newline="") as handle:
            handle.write(new_text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update", action="store_true", help="rewrite changed status values")
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()

    client_id = os.environ.get("SC_CLIENT_ID", "")
    if not client_id:
        try:
            client_id = discover(args.timeout)
        except (OSError, RuntimeError) as error:
            sys.exit(f"could not resolve the client id: {error}")
        print("resolved a client id from the web player", file=sys.stderr)

    manifest = ROOT / "endpoints.yaml"
    entries = [
        entry for entry in yaml.safe_load(manifest.read_text(encoding="utf-8"))
        if entry.get("class") == "read"
    ]
    if not entries:
        sys.exit("no read routes to probe")

    current = values()
    context = ssl.create_default_context()
    width = max(len(entry["id"]) for entry in entries)
    print(f"{'id':<{width}}  {'want':>4}  {'got':>4}")
    failures = 0
    errors = 0
    updates = {}

    for entry in entries:
        status, body = send(entry, current, client_id, args.timeout, context)
        expected = entry.get("status")
        if status is None:
            failures += 1
            errors += 1
            print(f"{entry['id']:<{width}}  {str(expected):>4}  {'ERR':>4}  {body}")
        else:
            remember_track(entry, status, body, current)
            if str(status) != str(expected):
                failures += 1
                if args.update:
                    updates[entry["id"]] = status
            print(f"{entry['id']:<{width}}  {str(expected):>4}  {status:>4}")
        time.sleep(1.1)

    if args.update and updates:
        rewrite_statuses(manifest, updates)
        print(f"updated {len(updates)} status values", file=sys.stderr)
    if args.update:
        return 1 if errors else 0
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
