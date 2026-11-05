#!/usr/bin/env python3
"""Smoke-test a frozen Packtools sidecar without importing Packtools locally."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


SOURCE = """<?xml version="1.0" encoding="utf-8"?>
<article xmlns:xlink="http://www.w3.org/1999/xlink"
         dtd-version="1.3" specific-use="sps-1.10"
         article-type="research-article" xml:lang="en">
  <front>
    <journal-meta><journal-title-group><journal-title>Test</journal-title></journal-title-group></journal-meta>
    <article-meta>
      <title-group><article-title>Packtools test</article-title></title-group>
      <author-notes><fn fn-type="coi-statement"><p>No conflicts.</p></fn></author-notes>
    </article-meta>
  </front>
  <body><p>Test.</p></body>
  <back><fn-group><fn fn-type="data-availability"><p>Data available.</p></fn></fn-group></back>
  <sub-article id="SA1" article-type="reviewer-report" xml:lang="en">
    <front-stub><title-group><article-title>Review</article-title></title-group></front-stub>
    <body><p>Review text.</p></body>
  </sub-article>
</article>
"""


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: smoke_test.py PATH_TO_SIDECAR")
    executable = Path(sys.argv[1]).resolve()
    if not executable.is_file():
        raise SystemExit(f"sidecar does not exist: {executable}")
    request = {
        "protocol": 1,
        "sps_version": "1.10",
        "locale": "pt_BR",
        "source": SOURCE,
    }
    completed = subprocess.run(
        [str(executable)],
        input=json.dumps(request),
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"sidecar exited with {completed.returncode}: {completed.stderr[:500]}"
        )
    response = json.loads(completed.stdout)
    if response.get("protocol") != 1:
        raise SystemExit("unexpected sidecar protocol")
    if response.get("packtools_version") != "4.18.3":
        raise SystemExit("sidecar did not report Packtools 4.18.3")
    if response.get("engine_errors"):
        raise SystemExit(f"Packtools rule groups failed: {response['engine_errors'][:3]}")
    messages = "\n".join(
        diagnostic.get("message", "") for diagnostic in response.get("diagnostics", [])
    )
    forbidden = (
        "Invalid value 'coi-statement'",
        "Invalid value 'data-availability'",
        "Invalid value 'reviewer-report'",
    )
    if any(message in messages for message in forbidden):
        raise SystemExit("Packtools 4.18.3 accepted-value smoke test failed")
    print(
        f"Packtools {response['packtools_version']} sidecar OK "
        f"({len(response.get('diagnostics', []))} diagnostics)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
