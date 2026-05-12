#!/usr/bin/env python3
"""
install_pyside.py — Install the Nephis Python sidecar and its dependencies.

Usage:
  python scripts/install_pyside.py

Requires Python 3.11+ and pip.
"""
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PYSIDE_DIR = REPO_ROOT / "apps" / "pyside"


def run(*args, **kwargs):
    print(f">>> {' '.join(str(a) for a in args)}")
    subprocess.run(list(args), check=True, **kwargs)


def main():
    if sys.version_info < (3, 11):
        print("ERROR: Python 3.11+ is required.", file=sys.stderr)
        sys.exit(1)

    print("=== Installing Nephis pyside sidecar ===\n")
    print(f"  Directory: {PYSIDE_DIR}\n")

    # Install the package in editable mode so imports work from the repo root.
    run(
        sys.executable, "-m", "pip", "install", "-e", str(PYSIDE_DIR),
        "--quiet",
    )

    print("\n=== Checking optional GPU deps ===")
    # ONNX Runtime — Silero VAD needs this.
    run(sys.executable, "-m", "pip", "install", "onnxruntime", "--quiet")

    print("\n=== Installation complete ===")
    print("\nTo start the sidecar:")
    print("  python -m nephis_pyside.pipe_server")
    print("\nThe Rust app will connect automatically on first voice activation (Alt+V).")


if __name__ == "__main__":
    main()
