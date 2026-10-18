#!/usr/bin/env python3
"""Check final Debian archives without installing or executing their payloads."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--check-tool", action="store_true")
    args = parser.parse_args()
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("LINTIAN_")
    }
    report = {
        "schema": "bds.debian-lintian.v1",
        "mode": "tool-preflight" if args.check_tool else "archive-check",
        "checked_at": datetime.now(timezone.utc).isoformat(),
        "policy": "debian; errors block; warnings recorded; no user configuration",
        "installation_performed": False,
        "release_acceptance": False,
        "tool_version": None,
        "packages": [],
        "errors": [],
    }
    tool = shutil.which("lintian", path=env.get("PATH"))
    try:
        if not tool:
            raise ValueError("Lintian is required; it is not available on PATH")
        version = subprocess.run(
            [tool, "--version"],
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=True,
        )
        report["tool_version"] = version.stdout.strip()
        if not report["tool_version"]:
            raise ValueError("Lintian did not report its version")
        if not args.check_tool:
            if not args.paths:
                raise ValueError("Specify at least one final .deb archive or directory")
            packages = set()
            for path in args.paths:
                if path.is_symlink() or not path.exists():
                    raise ValueError(f"Missing or symbolic package input: {path}")
                found = sorted(path.rglob("*.deb")) if path.is_dir() else [path]
                if not found:
                    raise ValueError(f"No Debian archives found: {path}")
                for package in found:
                    if (
                        package.is_symlink()
                        or not package.is_file()
                        or package.suffix != ".deb"
                    ):
                        raise ValueError(f"Expected a regular .deb archive: {package}")
                    packages.add(package.resolve())
            for package in sorted(packages):
                before = digest(package)
                result = subprocess.run(
                    [
                        tool,
                        "--no-user-dirs",
                        "--no-cfg",
                        "--profile",
                        "debian",
                        "--tag-display-limit",
                        "0",
                        "--fail-on",
                        "error",
                        "--color",
                        "never",
                        str(package),
                    ],
                    env=env,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    errors="replace",
                    timeout=600,
                )
                errors = len(re.findall(r"^E:", result.stdout, re.MULTILINE))
                warnings = len(re.findall(r"^W:", result.stdout, re.MULTILINE))
                unchanged = digest(package) == before
                passed = result.returncode == 0 and errors == 0 and unchanged
                report["packages"].append(
                    {
                        "path": str(package),
                        "sha256": before,
                        "bytes": package.stat().st_size,
                        "exit_code": result.returncode,
                        "errors": errors,
                        "warnings": warnings,
                        "unchanged_during_check": unchanged,
                        "passed": passed,
                        "output": result.stdout,
                    }
                )
                print(
                    f"{package}: {'PASS' if passed else 'FAIL'} ({errors} errors, {warnings} warnings)"
                )
                print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report["errors"].append(str(error))
        print(f"Package validation failed: {error}", file=sys.stderr)
    report["passed"] = not report["errors"] and all(
        p["passed"] for p in report["packages"]
    )
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        # The caller chooses a fresh evidence path. Never replace a prior report.
        with args.report.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
