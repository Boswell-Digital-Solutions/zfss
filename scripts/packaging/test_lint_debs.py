"""Exercise the command boundary; fake Lintian results are not archive proof."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent


class LintianBoundary(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tools = self.root / "tools"
        self.tools.mkdir()
        self.package = self.root / "example package.deb"
        self.package.write_bytes(b"disposable command fixture; not a real package")
        self.report = self.root / "report.json"
        self.env = dict(os.environ, PATH=str(self.tools), FIXTURE_MODE="pass")
        self.tool = self.tools / "lintian"
        self.tool.write_text(
            "#!/bin/sh\n"
            'if [ "$1" = --version ]; then echo "Lintian fixture"; exit 0; fi\n'
            '[ -z "${LINTIAN_SUPPRESS_TAGS:-}" ] || exit 9\n'
            'case "$FIXTURE_MODE" in\n'
            'error) echo "E: fixture: rejected"; exit 2;;\n'
            'crash) echo "tool failed" >&2; exit 1;;\n'
            "silent) exit 7;;\n"
            'warning) echo "W: fixture: review";;\n'
            'mixed) for arg do case "$arg" in *bad.deb) echo "E: fixture: rejected"; exit 2;; esac; done;;\n'
            "esac\n"
        )
        self.tool.chmod(0o755)

    def run_gate(self, *paths):
        return subprocess.run(
            [
                sys.executable,
                str(ROOT / "lint_debs.py"),
                *map(str, paths),
                "--report",
                str(self.report),
            ],
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )

    def result(self):
        return json.loads(self.report.read_text())

    def test_success_records_hash_without_changing_archive(self):
        before = self.package.read_bytes()
        result = self.run_gate(self.package)
        self.assertEqual(result.returncode, 0, result.stdout)
        report = self.result()
        self.assertTrue(report["passed"])
        self.assertEqual(report["tool_version"], "Lintian fixture")
        self.assertEqual(report["packages"][0]["bytes"], len(before))
        self.assertEqual(self.package.read_bytes(), before)
        self.assertFalse(report["release_acceptance"])

    def test_missing_tool_blocks(self):
        self.tool.unlink()
        self.assertNotEqual(self.run_gate(self.package).returncode, 0)
        self.assertIn("required", self.result()["errors"][0])

    def test_errors_and_tool_failures_block(self):
        for mode in ["error", "crash", "silent"]:
            with self.subTest(mode=mode):
                self.env["FIXTURE_MODE"] = mode
                self.assertNotEqual(self.run_gate(self.package).returncode, 0)
                self.assertFalse(self.result()["passed"])
                self.report.unlink()

    def test_warnings_are_recorded(self):
        self.env["FIXTURE_MODE"] = "warning"
        self.assertEqual(self.run_gate(self.package).returncode, 0)
        self.assertEqual(self.result()["packages"][0]["warnings"], 1)

    def test_empty_directory_and_missing_archive_block(self):
        for path in [self.tools, self.root / "missing.deb"]:
            self.assertNotEqual(self.run_gate(path).returncode, 0)
            self.assertFalse(self.result()["passed"])
            self.report.unlink()

    def test_all_archives_are_checked(self):
        (self.root / "bad.deb").write_bytes(b"second fixture")
        self.env["FIXTURE_MODE"] = "mixed"
        self.assertNotEqual(self.run_gate(self.root).returncode, 0)
        self.assertEqual(len(self.result()["packages"]), 2)

    def test_user_suppression_environment_is_removed(self):
        self.env["LINTIAN_SUPPRESS_TAGS"] = "all"
        self.assertEqual(self.run_gate(self.package).returncode, 0)

    def test_existing_evidence_is_not_overwritten(self):
        self.report.write_text("retained evidence\n")
        self.assertNotEqual(self.run_gate(self.package).returncode, 0)
        self.assertEqual(self.report.read_text(), "retained evidence\n")

    def test_build_wrapper_propagates_build_and_lint_failures(self):
        repo = self.root / "repository"
        scripts = repo / "scripts/packaging"
        scripts.mkdir(parents=True)
        for name in ["lint_debs.py", "tauri-checked.sh"]:
            shutil.copy2(ROOT / name, scripts / name)
        (repo / "build-deb.sh").write_text("#!/bin/sh\nexit 0\n")
        (self.tools / "bash").symlink_to("/bin/bash")
        bundle = repo / "src-tauri/target/release/bundle/deb"
        bundle.mkdir(parents=True)
        shutil.copy2(self.package, bundle / "app.deb")
        # Match the repo-specific CLI in the wrapper without invoking a build.
        wrapper = (scripts / "tauri-checked.sh").read_text()
        cli = next(
            line.split()[0] for line in wrapper.splitlines() if 'tauri "$@"' in line
        )
        (self.tools / cli).write_text(
            "#!/bin/sh\n"
            '[ "${FIXTURE_BUILD_STALE:-false}" = true ] || touch "$FIXTURE_PACKAGE"\n'
            'exit "${FIXTURE_BUILD_EXIT:-0}"\n'
        )
        (self.tools / cli).chmod(0o755)
        for name in ["uname", "mktemp", "dirname", "touch", "rm"]:
            (self.tools / name).symlink_to(shutil.which(name))
        (self.tools / "python3").symlink_to(sys.executable)
        self.env["FIXTURE_PACKAGE"] = str(bundle / "app.deb")
        self.env.pop("CARGO_TARGET_DIR", None)
        self.env.pop("CARGO_BUILD_TARGET", None)
        for build_exit, mode, expected in [
            (0, "pass", 0),
            (4, "pass", 4),
            (0, "error", 1),
        ]:
            with self.subTest(build_exit=build_exit, mode=mode):
                self.env.update(FIXTURE_BUILD_EXIT=str(build_exit), FIXTURE_MODE=mode)
                result = subprocess.run(
                    [
                        "/bin/bash",
                        str(scripts / "tauri-checked.sh"),
                        "build",
                        "--bundles",
                        "deb",
                    ],
                    env=self.env,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                )
                self.assertEqual(result.returncode, expected, result.stdout)
        self.env.update(
            FIXTURE_BUILD_EXIT="0", FIXTURE_MODE="pass", FIXTURE_BUILD_STALE="true"
        )
        result = subprocess.run(
            [
                "/bin/bash",
                str(scripts / "tauri-checked.sh"),
                "build",
                "--bundles",
                "deb",
            ],
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not refreshed", result.stdout)
        # Debug cross-target output must be checked in its actual directory.
        custom = repo / "custom-target"
        debug_bundle = custom / "x86_64-unknown-linux-gnu/debug/bundle/deb"
        debug_bundle.mkdir(parents=True)
        shutil.copy2(self.package, debug_bundle / "debug.deb")
        self.env.update(
            CARGO_TARGET_DIR=str(custom),
            FIXTURE_BUILD_STALE="false",
            FIXTURE_PACKAGE=str(debug_bundle / "debug.deb"),
        )
        result = subprocess.run(
            [
                "/bin/bash",
                str(scripts / "tauri-checked.sh"),
                "build",
                "--debug",
                "--target=x86_64-unknown-linux-gnu",
                "--bundles=deb",
            ],
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.assertEqual(result.returncode, 0, result.stdout)
        # AppImage-only output has no Debian claim, so Lintian is not required.
        self.tool.unlink()
        result = subprocess.run(
            [
                "/bin/bash",
                str(scripts / "tauri-checked.sh"),
                "build",
                "--bundles",
                "appimage",
            ],
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.assertEqual(result.returncode, 0, result.stdout)


if __name__ == "__main__":
    unittest.main()
