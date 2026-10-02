#!/usr/bin/env python3
"""Regenerates the images in docs/img/ and the browser demo's sample pair.

Run from the repository root:

    python3 docs/img/generate.py

Needs cargo and Pillow (with WebP support). The scenes and heatmaps come from
`examples/showcase.rs`; this script converts them to WebP and draws the
benchmark chart from the recorded results in `parity/results/`.
"""

import re
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
IMG = ROOT / "docs" / "img"
WEB = ROOT / "examples" / "web"
RESULTS = ROOT / "parity" / "results"


def run_showcase():
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(
            ["cargo", "run", "--release", "--features", "image",
             "--example", "showcase", "--", tmp, str(WEB)],
            cwd=ROOT, check=True,
        )
        for name in ["hero", "hdr-exposures"]:
            image = Image.open(Path(tmp) / f"{name}.png").convert("RGB")
            image.save(IMG / f"{name}.webp", "WEBP", quality=90, method=6)


def table(path):
    """Returns {case: [seconds, ...]} from the first Markdown table in path."""
    rows = {}
    for line in path.read_text().splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 3 or not re.fullmatch(r"[\d.]+", cells[1]):
            continue
        rows[cells[0]] = [float(c) for c in cells[1:3]]
    return rows


def label(case):
    # "HDR ACES, 3 exposures 1920×1080" -> "HDR 1920×1080"
    kind = case.split()[0]
    size = case.split()[-1]
    return f"{kind} {size}"


def chart():
    single = table(RESULTS / "benchmarks-single.md")
    parallel = table(RESULTS / "benchmarks-parallel.md")
    series = [
        ("C++ -O2, 1 thread", "bar-cpp"),
        ("Rust, 1 thread", "bar-rs1"),
        ("Rust, 8 threads", "bar-rs8"),
    ]
    cases = list(single)
    left, right, bar, gap, group_gap = 120, 90, 14, 3, 18
    plot_w = 440
    top = 70
    group_h = 3 * bar + 2 * gap
    height = top + len(cases) * (group_h + group_gap) + 40
    width = left + plot_w + right
    out = []
    out.append(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" font-family="-apple-system,BlinkMacSystemFont,'
        f'Segoe UI,Helvetica,Arial,sans-serif" font-size="12">'
    )
    out.append(
        "<style>"
        "text{fill:#1f2328}.muted{fill:#59636e}.axis{stroke:#d1d9e0}"
        ".bar-cpp{fill:#8c959f}.bar-rs1{fill:#e8743b}.bar-rs8{fill:#8250df}"
        "@media (prefers-color-scheme:dark){text{fill:#e6edf3}.muted{fill:#9198a1}"
        ".axis{stroke:#3d444d}.bar-cpp{fill:#768390}.bar-rs1{fill:#f0883e}"
        ".bar-rs8{fill:#a371f7}}"
        "</style>"
    )
    out.append(
        f'<text x="0" y="16" font-size="14" font-weight="600">'
        f"Time relative to C++ -O2 on one thread (shorter is faster)</text>"
    )
    out.append(
        '<text class="muted" x="0" y="34">Seconds per comparison, median of 3 runs, '
        "AMD Ryzen 9 7945HX. HDR: ACES, 3 exposures.</text>"
    )
    x = left
    for i, (name, cls) in enumerate(series):
        out.append(f'<rect class="{cls}" x="{x}" y="46" width="10" height="10"/>')
        out.append(f'<text x="{x + 14}" y="55">{name}</text>')
        x += 14 + 7 * len(name) + 18
    for g, case in enumerate(cases):
        cpp, rs1 = single[case]
        rs8 = parallel[case][1]
        y0 = top + g * (group_h + group_gap)
        out.append(
            f'<text x="{left - 8}" y="{y0 + group_h / 2 + 4}" text-anchor="end">'
            f"{label(case)}</text>"
        )
        for i, ((_, cls), seconds) in enumerate(zip(series, [cpp, rs1, rs8])):
            w = plot_w * seconds / cpp
            y = y0 + i * (bar + gap)
            out.append(
                f'<rect class="{cls}" x="{left}" y="{y}" width="{w:.1f}" height="{bar}"/>'
            )
            out.append(
                f'<text x="{left + w + 6:.1f}" y="{y + bar - 3}">{seconds:.3f} s</text>'
            )
    axis_y = top + len(cases) * (group_h + group_gap) - group_gap + 6
    out.append(
        f'<line class="axis" x1="{left}" y1="{top - 4}" x2="{left}" y2="{axis_y}"/>'
    )
    out.append(
        f'<text class="muted" x="0" y="{height - 8}">8-thread C++ was not measured. '
        "Data: parity/results/benchmarks-*.md</text>"
    )
    out.append("</svg>")
    (IMG / "benchmarks.svg").write_text("\n".join(out) + "\n")


if __name__ == "__main__":
    run_showcase()
    chart()
