"""Render local HTML artwork with Chrome or Chromium, preserving SVG strokes."""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def chrome_binary():
    configured = os.environ.get("CHROME")
    candidates = (configured,) if configured else (
        "google-chrome", "chromium", "chromium-browser", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    )
    for candidate in candidates:
        if browser := shutil.which(candidate):
            return browser
    sys.exit("Install Chrome or Chromium, or set CHROME to its executable path to render HTML artwork.")


def render_html(layout, output, size, *, browser=None):
    """Stage a PNG before replacing the output; keep each render's browser profile isolated."""
    browser = browser or chrome_binary()
    output = Path(output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    width, height = size
    with tempfile.TemporaryDirectory(prefix=".html-render-", dir=output.parent) as directory:
        work = Path(directory)
        capture = work / "capture.png"
        command = [
            browser, "--headless", "--no-first-run", "--no-default-browser-check", "--disable-background-networking",
            "--force-color-profile=srgb", "--force-device-scale-factor=1", "--hide-scrollbars",
            f"--window-size={width},{height}", f"--user-data-dir={work / 'chrome-profile'}",
            f"--screenshot={capture}", Path(layout).resolve().as_uri(),
        ]
        result = subprocess.run(command, capture_output=True, text=True)
        if result.returncode != 0 or not capture.is_file():
            sys.exit(f"Could not render {output.name}:\n{result.stderr or result.stdout}")
        if not output.is_file() or output.read_bytes() != capture.read_bytes():
            capture.replace(output)
