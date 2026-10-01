#!/usr/bin/env python3
"""Single-thread stage profile of ldr_flip and hdr_flip.

Copies the crate to a temporary directory, inserts timers at fixed source
anchors, and runs the benchmark inputs. The working tree is not modified.
"""
import argparse
from collections import defaultdict
import hashlib
import os
from pathlib import Path
import shutil
from statistics import median
import subprocess
import tempfile

START = "let profile_start = std::time::Instant::now();\n"


def emit(label):
    return f'eprintln!("STAGE|{label}|{{:.9}}", profile_start.elapsed().as_secs_f64());\n'


def edit(root, name, replacements):
    path = root / name
    source = path.read_text()
    for old, new in replacements:
        if source.count(old) != 1:
            raise ValueError(f"Expected one instrumentation anchor in {name}: {old!r}")
        source = source.replace(old, new, 1)
    path.write_text(source)


def instrument(root):
    evaluation = "    filters.evaluate(&mut workspace);"
    edit(root, "src/lib.rs", [
        ("let filters = filters::Filters::new(ppd)?;", START + "let filters = filters::Filters::new(ppd)?;"),
        (evaluation, emit("LDR conversion/setup") + evaluation),
    ])
    feature_comment = "// image::computeFeatureDifferenceAndFinalError\n"
    edit(root, "src/filters.rs", [
        ("// image::computeColorDifference (two separable passes)", "// image::computeColorDifference (two separable passes)\n" + START),
        ("let cmax = color::max_distance();", emit("Color horizontal") + START + "let cmax = color::max_distance();"),
        ("        });\n    }\n\n    fn feature_difference", "        });\n" + emit("Color vertical/Lab metric") + "    }\n\n    fn feature_difference"),
        (feature_comment, feature_comment + START),
        ("let norm = 1.0 / 2.0_f32.sqrt();", emit("Feature horizontal") + START + "let norm = 1.0 / 2.0_f32.sqrt();"),
        ("        });\n    }\n}\n\n// Keep", "        });\n" + emit("Feature vertical/final metric") + "    }\n}\n\n// Keep"),
        ("        workspace.reference.normalize_luminance();", START + "        workspace.reference.normalize_luminance();"),
        ("        self.feature_difference_and_final_error(", emit("Feature luminance normalization") + "        self.feature_difference_and_final_error("),
    ])
    hdr_eval = "        filters.evaluate(&mut workspace);"
    edit(root, "src/hdr.rs", [
        ("    if reference.width != test.width", "    " + START + "    if reference.width != test.width"),
        ("    for i in 0..parameters.num_exposures {", emit("HDR setup/resolve") + "    for i in 0..parameters.num_exposures {\n" + START),
        (hdr_eval, emit("HDR tone/opponent conversion") + hdr_eval + "\n" + START),
        ("    }\n    Ok(HdrResult {", emit("HDR max/exposure merge") + "    }\n    Ok(HdrResult {"),
    ])
    (root / "examples/profile.rs").write_text('''#[allow(dead_code)]
mod support;
use support::{generated, hdr_options, Case, Result};
fn main() -> Result<()> {
    for (w, h) in [(1920, 1080), (3840, 2160)] {
        for hdr in [false, true] {
            let (r, t) = generated("noise", w, h, hdr);
            let case = Case {
                name: format!("{} {w}x{h}", if hdr { "HDR" } else { "LDR" }),
                group: "Profile", w, h, r, t,
                options: hdr.then(|| hdr_options(
                    flip_rs::DEFAULT_PPD, flip_rs::Tonemapper::Aces, Some(-4.0), Some(4.0), Some(3),
                )),
                ppd: flip_rs::DEFAULT_PPD,
            };
            for iteration in 0..4 {
                eprintln!("CASE|{}|{iteration}", case.name);
                let result = case.rust()?;
                eprintln!("STAGE|API total|{:.9}", result.seconds);
                let start = std::time::Instant::now();
                std::hint::black_box(result.map.statistics());
                eprintln!("STAGE|Pooling (outside API)|{:.9}", start.elapsed().as_secs_f64());
            }
        }
    }
    Ok(())
}
''')
    with (root / "Cargo.toml").open("a") as manifest:
        manifest.write('\n[[example]]\nname = "profile"\nrequired-features = ["image"]\n')


def report(log):
    samples = defaultdict(lambda: defaultdict(list))
    sums = defaultdict(float)
    case = None
    iteration = 0

    def finish():
        if case and iteration > 0:
            for stage, seconds in sums.items():
                samples[case][stage].append(seconds)

    for line in log.splitlines():
        fields = line.split("|")
        if fields[0] == "CASE":
            finish()
            case, iteration = fields[1], int(fields[2])
            sums.clear()
        elif fields[0] == "STAGE":
            sums[fields[1]] += float(fields[2])
    finish()
    lines = ["| Case | Stage | Median seconds |", "|---|---|---:|"]
    for case, stages in samples.items():
        for stage, times in stages.items():
            if len(times) != 3:
                raise ValueError(f"Expected three timed samples for {case}/{stage}")
            lines.append(f"| {case} | {stage} | {median(times):.6f} |")
    if len(samples) != 4:
        raise ValueError("Expected all four profile cases")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    source = Path(__file__).resolve().parent.parent
    target = Path(os.environ.get("CARGO_TARGET_DIR", source / "target")).resolve()
    identities = []
    for name in ["Cargo.toml", "Cargo.lock", "src/lib.rs", "src/color.rs", "src/filters.rs", "src/hdr.rs"]:
        identities.append(f"- `{name}`: `{hashlib.sha256((source / name).read_bytes()).hexdigest()}`")
    with tempfile.TemporaryDirectory(prefix="flip-stage-profile-") as temporary:
        root = Path(temporary)
        for directory in ["src", "examples"]:
            shutil.copytree(source / directory, root / directory)
        for name in ["Cargo.toml", "Cargo.lock", "README.md", "LICENSE"]:
            shutil.copy2(source / name, root / name)
        instrument(root)
        subprocess.run([
            "cargo", "build", "--release", "--no-default-features", "--features", "image",
            "--manifest-path", str(root / "Cargo.toml"), "--target-dir", str(target),
            "--example", "profile",
        ], check=True)
        run = subprocess.run([str(target / "release/examples/profile")], check=True, capture_output=True, text=True)
    args.report.write_text(
        "# Single-thread stage profile\n\n"
        "Stage timers are inserted into a temporary copy of the sources. "
        "Release profile, one warmup followed by three samples. HDR stage times "
        "sum all three exposures per sample before taking the median. "
        "Pooling is outside the comparison API. Stage medians need not add up to "
        "the separately measured API median. Hardware is recorded in "
        "`qualification.md`.\n\n"
        + report(run.stderr)
        + "\nSource SHA-256 before instrumentation:\n\n"
        + "\n".join(identities)
        + "\n\nRaw stage timings:\n\n```text\n"
        + run.stderr + "```\n"
    )
    print(f"Wrote {args.report}")


if __name__ == "__main__":
    main()
