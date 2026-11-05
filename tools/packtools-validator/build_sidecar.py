#!/usr/bin/env python3
"""Build the pinned Packtools validation adapter as a Tauri sidecar."""

from __future__ import annotations

import os
import shutil
import sys
from importlib import metadata
from pathlib import Path

_WORKSPACE = Path(__file__).resolve().parents[2]
os.environ.setdefault(
    "PYINSTALLER_CONFIG_DIR", str(_WORKSPACE / "target" / "pyinstaller-cache")
)

import PyInstaller.__main__
from packtools.version import __version__ as packtools_version


EXPECTED_VERSION = "4.18.3"


def collect_license_notices(destination: Path) -> None:
    sections = [
        "SciXML Packtools validator — third-party license notices",
        "=========================================================",
        "",
        "Generated from the exact Python environment used to freeze this sidecar.",
        "",
    ]
    seen: set[tuple[str, str]] = set()
    for distribution in sorted(
        metadata.distributions(),
        key=lambda item: (item.metadata.get("Name") or "").lower(),
    ):
        name = distribution.metadata.get("Name") or "unknown"
        version = distribution.version
        license_files: list[Path] = []
        for entry in distribution.files or []:
            lowered = entry.name.lower()
            if not any(
                marker in lowered
                for marker in ("license", "licence", "copying", "notice")
            ):
                continue
            path = Path(distribution.locate_file(entry))
            if path.is_file() and path.stat().st_size <= 1024 * 1024:
                license_files.append(path)
        if not license_files:
            declared = (
                distribution.metadata.get("License-Expression")
                or distribution.metadata.get("License")
                or "No machine-readable license field"
            )
            sections.extend((f"--- {name} {version} ---", declared.strip(), ""))
            continue
        for path in sorted(set(license_files)):
            key = (name.lower(), path.name)
            if key in seen:
                continue
            seen.add(key)
            text = path.read_text(encoding="utf-8", errors="replace").strip()
            sections.extend((f"--- {name} {version}: {path.name} ---", text, ""))

    python_license = Path(os.__file__).resolve().parent / "LICENSE.txt"
    if python_license.is_file():
        sections.extend(
            (
                f"--- Python {sys.version.split()[0]}: LICENSE.txt ---",
                python_license.read_text(encoding="utf-8", errors="replace").strip(),
                "",
            )
        )
    destination.write_text("\n".join(sections), encoding="utf-8")


def main() -> int:
    if packtools_version != EXPECTED_VERSION:
        raise RuntimeError(
            f"expected Packtools {EXPECTED_VERSION}, found {packtools_version}"
        )
    workspace = _WORKSPACE
    source = Path(__file__).with_name("scixml_packtools_validator.py")
    output = workspace / "target" / "packtools-sidecar"
    work = workspace / "target" / "packtools-build"
    shutil.rmtree(output, ignore_errors=True)
    shutil.rmtree(work, ignore_errors=True)
    output.mkdir(parents=True, exist_ok=True)
    work.mkdir(parents=True, exist_ok=True)

    arguments = [
        str(source),
        "--name=scixml-packtools-validator",
        "--onefile",
        "--clean",
        "--noconfirm",
        "--noupx",
        f"--distpath={output}",
        f"--workpath={work}",
        f"--specpath={work}",
        "--collect-data=packtools",
    ]
    PyInstaller.__main__.run(arguments)

    suffix = ".exe" if sys.platform == "win32" else ""
    executable = output / f"scixml-packtools-validator{suffix}"
    if not executable.is_file() or executable.stat().st_size == 0:
        raise RuntimeError(f"Packtools sidecar was not produced: {executable}")
    collect_license_notices(output / "THIRD-PARTY-LICENSES.txt")
    print(f"built {executable}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
