#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageChops


EXPECTED_SIZE = (1280, 720)


def compare(actual: Path, golden: Path, tolerance: int) -> None:
    image = Image.open(actual).convert("RGB")
    expected = Image.open(golden).convert("RGB")
    if image.size != EXPECTED_SIZE:
        raise SystemExit(f"expected {EXPECTED_SIZE} capture, got {image.size}: {actual}")
    if expected.size != image.size:
        raise SystemExit(f"image size changed: actual={image.size} golden={expected.size}")
    difference = ImageChops.difference(image, expected)
    changed = sum(1 for pixel in difference.getdata() if max(pixel) > tolerance)
    allowed = image.width * image.height // 1000
    if changed > allowed:
        diff = actual.with_name(f"{actual.stem}-diff.png")
        difference.save(diff)
        raise SystemExit(f"{actual.name}: {changed} pixels differ; allowed {allowed}; diff={diff}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update", action="store_true")
    parser.add_argument("--binary", default="target/debug/vn")
    parser.add_argument("--tolerance", type=int, default=3)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    output = root / "target/visual-ci"
    golden = root / "crates/vn_bevy/tests/golden"
    output.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["WGPU_BACKEND"] = "vulkan"
    env["LIBGL_ALWAYS_SOFTWARE"] = "1"
    env["XDG_DATA_HOME"] = tempfile.mkdtemp(prefix="vinyl-visual-data-")

    subprocess.run(
        [str(root / args.binary), "run", str(root / "fixtures/mvp"), "--visual-test-output", str(output)],
        cwd=root,
        env=env,
        timeout=45,
        check=True,
    )

    save_files = list(Path(env["XDG_DATA_HOME"]).rglob("slot-01.json"))
    if len(save_files) != 1:
        raise SystemExit(f"expected one manual save, found {save_files}")
    save = json.loads(save_files[0].read_text())
    if not save.get("screenshot_png"):
        raise SystemExit("manual save screenshot is empty")

    for name in ("menu.png", "next.png"):
        actual = output / name
        expected = golden / name
        if Image.open(actual).size != EXPECTED_SIZE:
            raise SystemExit(f"expected {EXPECTED_SIZE} capture, got {Image.open(actual).size}: {actual}")
        if args.update:
            golden.mkdir(parents=True, exist_ok=True)
            expected.write_bytes(actual.read_bytes())
        elif not expected.exists():
            raise SystemExit(f"missing golden: {expected}; run with --update")
        else:
            compare(actual, expected, args.tolerance)


if __name__ == "__main__":
    main()
