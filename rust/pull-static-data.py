#!/usr/bin/env python3
"""Fetch the latest pocamind/data `all.json` bundle into `assets/`.
"""

import json
import os
import sys
import urllib.request

URL = "https://github.com/pocamind/data/releases/latest/download/all.json"


def main() -> int:
    dest = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "all.json")
    os.makedirs(os.path.dirname(dest), exist_ok=True)

    req = urllib.request.Request(URL, headers={"User-Agent": "deepwoken-pull-static-data"})
    try:
        with urllib.request.urlopen(req) as resp:
            data = resp.read()
    except Exception as e:  # noqa: BLE001 - report any download failure and stop
        print(f"failed to download {URL}: {e}", file=sys.stderr)
        return 1

    try:
        json.loads(data)
    except ValueError as e:
        print(f"downloaded file is not valid JSON, leaving assets/all.json untouched: {e}", file=sys.stderr)
        return 1

    with open(dest, "wb") as f:
        f.write(data)
    print(f"wrote {dest} ({len(data)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
