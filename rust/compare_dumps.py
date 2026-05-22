#!/usr/bin/env python3
"""Compare two EUI-NEO dump JSON files (C++ vs Rust)."""

import json
import sys
import os

# Tolerances
RECT_TOL = 1.0
COLOR_TOL = 0.02
FLOAT_TOL = 0.5


def load_dump(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def compare_arrays(a, b, tol, label):
    diffs = []
    for i, (va, vb) in enumerate(zip(a, b)):
        d = abs(va - vb)
        if d > tol:
            diffs.append((i, va, vb, d))
    return diffs


def compare_command(idx, cpp, rust):
    issues = []

    # Type
    if cpp.get("type") != rust.get("type"):
        issues.append(f"  type: {cpp.get('type')} vs {rust.get('type')}")

    # Rect
    cr = cpp.get("rect", [0, 0, 0, 0])
    rr = rust.get("rect", [0, 0, 0, 0])
    rd = compare_arrays(cr, rr, RECT_TOL, "rect")
    if rd:
        issues.append(f"  rect: C++={fmt_arr(cr)} Rust={fmt_arr(rr)}")
        for i, va, vb, d in rd:
            issues.append(f"    [{i}] diff={d:.2f} (C++={va:.2f} Rust={vb:.2f})")

    # Color
    cc = cpp.get("color", [0, 0, 0, 0])
    rc = rust.get("color", [0, 0, 0, 0])
    cd = compare_arrays(cc, rc, COLOR_TOL, "color")
    if cd:
        issues.append(f"  color: C++={fmt_arr(cc)} Rust={fmt_arr(rc)}")

    # Text
    ct = cpp.get("text", "")
    rt = rust.get("text", "")
    if ct != rt and (ct or rt):
        issues.append(f"  text: C++={repr(ct)} Rust={repr(rt)}")

    # Font size
    cf = cpp.get("font_size", 0)
    rf = rust.get("font_size", 0)
    if abs(cf - rf) > FLOAT_TOL:
        issues.append(f"  font_size: C++={cf} Rust={rf}")

    # Radius/rounding
    crr = cpp.get("radius", 0)
    rrr = rust.get("radius", 0)
    if abs(crr - rrr) > FLOAT_TOL:
        issues.append(f"  radius: C++={crr} Rust={rrr}")

    # Blur
    cb = cpp.get("blur_radius", 0)
    rb = rust.get("blur_radius", 0)
    if abs(cb - rb) > FLOAT_TOL:
        issues.append(f"  blur_radius: C++={cb} Rust={rb}")

    # Shadow
    cs = cpp.get("shadow")
    rs = rust.get("shadow")
    if cs and rs:
        sb = abs(cs.get("blur", 0) - rs.get("blur", 0))
        if sb > FLOAT_TOL:
            issues.append(f"  shadow.blur: C++={cs['blur']} Rust={rs['blur']}")
        so = compare_arrays(
            cs.get("offset", [0, 0]), rs.get("offset", [0, 0]), FLOAT_TOL, "shadow.offset"
        )
        if so:
            issues.append(f"  shadow.offset: C++={cs['offset']} Rust={rs['offset']}")
        sc = compare_arrays(
            cs.get("color", [0, 0, 0, 0]),
            rs.get("color", [0, 0, 0, 0]),
            COLOR_TOL,
            "shadow.color",
        )
        if sc:
            issues.append(f"  shadow.color: C++={cs['color']} Rust={rs['color']}")
    elif cs != rs:
        issues.append(f"  shadow: C++={'present' if cs else 'null'} Rust={'present' if rs else 'null'}")

    return issues


def fmt_arr(arr):
    return "[" + ", ".join(f"{v:.2f}" for v in arr) + "]"


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <cpp_dump.json> <rust_dump.json>")
        sys.exit(1)

    cpp_path = sys.argv[1]
    rust_path = sys.argv[2]

    if not os.path.exists(cpp_path):
        print(f"ERROR: C++ dump not found: {cpp_path}")
        sys.exit(1)
    if not os.path.exists(rust_path):
        print(f"ERROR: Rust dump not found: {rust_path}")
        sys.exit(1)

    cpp = load_dump(cpp_path)
    rust = load_dump(rust_path)

    cpp_count = cpp.get("frame_command_count", 0)
    rust_count = rust.get("frame_command_count", 0)

    print(f"C++ commands:  {cpp_count}")
    print(f"Rust commands: {rust_count}")
    print()

    cpp_cmds = cpp.get("commands", [])
    rust_cmds = rust.get("commands", [])

    max_idx = max(len(cpp_cmds), len(rust_cmds))
    total_issues = 0
    matched = 0

    for i in range(max_idx):
        if i >= len(cpp_cmds):
            print(f"[{i}] EXTRA in Rust: {rust_cmds[i].get('type', '?')}")
            total_issues += 1
            continue
        if i >= len(rust_cmds):
            print(f"[{i}] MISSING in Rust: {cpp_cmds[i].get('type', '?')} text={repr(cpp_cmds[i].get('text', ''))}")
            total_issues += 1
            continue

        issues = compare_command(i, cpp_cmds[i], rust_cmds[i])
        if issues:
            ctype = cpp_cmds[i].get("type", "?")
            ctext = cpp_cmds[i].get("text", "")
            label = f"{ctype}"
            if ctext:
                label += f' "{ctext}"'
            print(f"[{i}] DIFF ({label}):")
            for issue in issues:
                print(issue)
            total_issues += 1
        else:
            matched += 1

    print()
    print(f"Summary: {matched}/{max_idx} commands match, {total_issues} with differences")

    if cpp_count != rust_count:
        print(f"WARNING: Command count mismatch ({cpp_count} vs {rust_count})")

    sys.exit(0 if total_issues == 0 else 1)


if __name__ == "__main__":
    main()
