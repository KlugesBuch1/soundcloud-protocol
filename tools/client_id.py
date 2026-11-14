"""Resolve the current client id from the SoundCloud web player."""

import re
import urllib.request

PLAYER_URL = "https://soundcloud.com/"
ASSET_PATTERN = re.compile(r'https://a-v2\.sndcdn\.com/assets/[^"]+\.js')
CLIENT_ID_PATTERN = re.compile(r'client_id:"([A-Za-z0-9]{32})"')
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
}


def fetch(url, timeout):
    request = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return response.read().decode("utf-8", "replace")


def discover(timeout=30.0):
    html = fetch(PLAYER_URL, timeout)
    urls = list(dict.fromkeys(ASSET_PATTERN.findall(html)))
    if not urls:
        raise RuntimeError("the player page did not reference any assets")
    failed = 0
    for url in urls:
        try:
            script = fetch(url, timeout)
        except OSError:
            failed += 1
            continue
        match = CLIENT_ID_PATTERN.search(script)
        if match:
            return match.group(1)
    raise RuntimeError(f"no client id in {len(urls)} player assets, {failed} failed to load")
