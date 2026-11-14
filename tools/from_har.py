"""Extract api-v2 routes from a HAR file.

The tool writes cleaned response bodies to examples/ and prints YAML blocks.
"""

import argparse
import base64
import json
import re
import sys
import urllib.parse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = "api-v2.soundcloud.com"
SENSITIVE = {
    "authorization",
    "client_id",
    "cookie",
    "csrf",
    "email",
    "expires",
    "key_pair_id",
    "licenseauthtoken",
    "oauth_token",
    "policy",
    "secret_token",
    "sess",
    "signature",
    "track_authorization",
}
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")


def normal_key(key):
    return key.lower().replace("-", "_")


def sensitive(key):
    name = normal_key(key)
    return name in SENSITIVE or "token" in name or name.startswith("csrf") or name.startswith("sess")


def clean_url(value):
    if not value.startswith("http"):
        return value
    parts = urllib.parse.urlsplit(value)
    kept = [
        (key, item) for key, item in urllib.parse.parse_qsl(parts.query, keep_blank_values=True)
        if not sensitive(key)
    ]
    return urllib.parse.urlunsplit(
        (parts.scheme, parts.netloc, parts.path, urllib.parse.urlencode(kept), "")
    )


def scrub(value):
    if isinstance(value, dict):
        return {
            key: scrub(item) for key, item in value.items()
            if not sensitive(key)
        }
    if isinstance(value, list):
        return [scrub(item) for item in value]
    if isinstance(value, str):
        return clean_url(EMAIL.sub("{email}", value))
    return value


def normalize_path(path):
    segments = [urllib.parse.unquote(item) for item in path.split("/") if item]
    if not segments:
        return "/"
    if segments[0] == "media":
        return "/media/{media_path}"
    if segments[0] in {"tracks", "users", "playlists"} and len(segments) > 1 and segments[1].isdigit():
        placeholder = {
            "tracks": "{track_id}",
            "users": "{user_id}",
            "playlists": "{playlist_id}",
        }[segments[0]]
        tail = segments[2:]
        if segments[0] == "users" and len(tail) > 1 and tail[0] == "track_likes" and tail[1].isdigit():
            tail[1] = "{track_id}"
        suffix = "/" + "/".join(tail) if tail else ""
        return "/" + segments[0] + "/" + placeholder + suffix
    return "/" + "/".join(segments)


def route_area(path):
    if path.startswith("/media"):
        return "streams", "docs/streams.md"
    if path.startswith("/users") and "track_likes" in path:
        return "likes", "docs/likes.md"
    if path.startswith("/users"):
        return "users", "docs/users.md"
    if path.startswith("/playlists"):
        return "playlists", "docs/playlists.md"
    if path.startswith("/search"):
        return "search", "docs/search.md"
    if path.startswith("/charts"):
        return "charts", "docs/charts.md"
    if path == "/me":
        return "auth", "docs/auth.md"
    return "tracks", "docs/tracks.md"


def slug(method, path):
    value = path.strip("/").replace("{", "").replace("}", "")
    value = re.sub(r"[^A-Za-z0-9]+", "-", value).strip("-").lower()
    return method.lower() + "-" + (value or "root")


def response_body(entry):
    content = entry.get("response", {}).get("content", {})
    text = content.get("text", "")
    if content.get("encoding") == "base64":
        text = base64.b64decode(text).decode("utf-8", "replace")
    if not text:
        return None
    try:
        return json.loads(text)
    except ValueError:
        return None


def block(record):
    area, doc = route_area(record["path"])
    auth = "user-token" if record["auth"] else "client_id"
    kind = "read" if record["method"] == "GET" else "write"
    lines = [
        f"- id: {slug(record['method'], record['path'])}",
        f"  area: {area}",
        f"  method: {record['method']}",
        f"  host: {HOST}",
        f"  path: {record['path']}",
        f"  auth: {auth}",
        f"  class: {kind}",
        f"  status: {record['status']}",
        f"  doc: {doc}",
        f"  summary: Captured {record['method']} {record['path']}.",
    ]
    if record["query"]:
        lines.append("  query:")
        lines.extend("    - " + key for key in record["query"])
    lines.append(f"  notes: Captured {record['count']} time(s) in the HAR.")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("har", type=Path)
    parser.add_argument("--out-dir", type=Path, default=ROOT / "examples")
    args = parser.parse_args()

    try:
        har = json.loads(args.har.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        sys.exit(f"cannot read HAR: {type(exc).__name__}")

    args.out_dir.mkdir(parents=True, exist_ok=True)
    records = {}
    for entry in har.get("log", {}).get("entries", []):
        request = entry.get("request", {})
        url = request.get("url", "")
        parts = urllib.parse.urlsplit(url)
        if parts.netloc.lower() != HOST:
            continue
        method = str(request.get("method", "GET")).upper()
        path = normalize_path(parts.path)
        key = (method, path)
        query = []
        for name, _ in urllib.parse.parse_qsl(parts.query, keep_blank_values=True):
            if not sensitive(name) and name not in query:
                query.append(name)
        headers = request.get("headers", [])
        auth = any(
            str(header.get("name", "")).lower() == "authorization"
            and header.get("value")
            for header in headers
        )
        if key in records:
            records[key]["count"] += 1
            continue
        body = response_body(entry)
        if body is None:
            continue
        records[key] = {
            "method": method,
            "path": path,
            "query": query,
            "auth": auth,
            "status": entry.get("response", {}).get("status"),
            "count": 1,
            "body": scrub(body),
        }

    if not records:
        print("no api-v2 JSON entries")
        return 0

    for record in records.values():
        name = slug(record["method"], record["path"]) + ".json"
        target = args.out_dir / name
        target.write_text(
            json.dumps(record["body"], indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        print(block(record))
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
