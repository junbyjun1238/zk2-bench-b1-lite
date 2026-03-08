#!/usr/bin/env python3
import argparse
import json
import subprocess
from pathlib import Path
from typing import Dict, List


def run_bench(
    arm: str,
    mode: str,
    scale: int,
    out_file: Path,
    require_cert: bool,
    lint: bool,
    k_run: int | None,
    input_profile: str,
) -> None:
    cmd = [
        "python",
        "scripts/run_ab_bench.py",
        "--arm",
        arm,
        "--mode",
        mode,
        "--scale",
        str(scale),
        "--input-profile",
        input_profile,
        "--out",
        str(out_file),
    ]
    if k_run is not None:
        cmd.extend(["--k-run", str(k_run)])
    if lint:
        cmd.append("--lint-output")
    if require_cert and arm == "B_note":
        cmd.append("--require-cert")

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        msg = "\n".join(
            [
                f"command failed: {' '.join(cmd)}",
                "stdout tail:",
                "\n".join((proc.stdout or "").strip().splitlines()[-10:]),
                "stderr tail:",
                "\n".join((proc.stderr or "").strip().splitlines()[-10:]),
            ]
        )
        raise RuntimeError(msg)


def load_json(path: Path) -> Dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ratio(num: float, den: float) -> float:
    if den == 0.0:
        return 0.0
    return num / den


def to_md_table(rows: List[Dict], out_md: Path, order_policy: str, input_profile: str) -> None:
    lines = [
        "# Local A/B Sweep (full-local)",
        "",
        f"- order policy: `{order_policy}`",
        f"- input profile: `{input_profile}`",
        "",
        "| scale | A_prove_ms | B_prove_ms | B/A prove | A_verify_ms | B_verify_ms | B/A verify | A_peak_rss_mb | B_peak_rss_mb | B/A rss |",
        "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for row in rows:
        lines.append(
            "| {scale} | {a_p:.3f} | {b_p:.3f} | {r_p:.3f} | {a_v:.3f} | {b_v:.3f} | {r_v:.3f} | {a_r:.3f} | {b_r:.3f} | {r_r:.3f} |".format(
                scale=row["scale"],
                a_p=row["a_secure"]["prove_ms"],
                b_p=row["b_note"]["prove_ms"],
                r_p=row["ratios"]["prove_b_over_a"],
                a_v=row["a_secure"]["verify_ms"],
                b_v=row["b_note"]["verify_ms"],
                r_v=row["ratios"]["verify_b_over_a"],
                a_r=row["a_secure"]["peak_rss_mb"],
                b_r=row["b_note"]["peak_rss_mb"],
                r_r=row["ratios"]["rss_b_over_a"],
            )
        )

    lines.extend(
        [
            "",
            "Notes:",
            "- Both arms use the same `run_ab_bench.py` contract and schema.",
            "- Both arms enforce the same row-family semantics.",
            "- `A_secure` uses explicit bit decomposition; `B_note` uses lookup-assisted binding.",
            "- This sweep is local-only (`full-local`) and intended for operational-fit baseline.",
            "- With `alternate` ordering, even-indexed scales run B->A while odd-indexed scales run A->B.",
            "",
        ]
    )
    out_md.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Run local full-local sweep for A_secure vs B_note.")
    parser.add_argument("--scales", default="2,4,8,16", help="Comma-separated scale list")
    parser.add_argument("--mode", default="full-local", choices=["full-local", "fast-structural"])
    parser.add_argument("--out-dir", default="benches/local_sweep")
    parser.add_argument("--out-md", default="docs/local_ab_table.md")
    parser.add_argument("--lint-output", action="store_true")
    parser.add_argument("--require-cert", action="store_true")
    parser.add_argument("--k-run", type=int, default=None, help="Fixed k for full-local runs")
    parser.add_argument("--order-policy", choices=["fixed-ab", "alternate"], default="alternate")
    parser.add_argument("--input-profile", choices=["standard", "boundary", "adversarial"], default="standard")
    args = parser.parse_args()

    scales = [int(x.strip()) for x in args.scales.split(",") if x.strip()]
    if not scales:
        raise ValueError("empty scales")
    if any(s <= 0 for s in scales):
        raise ValueError("scales must be positive integers")

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    summary_rows: List[Dict] = []
    for idx, scale in enumerate(scales):
        a_out = out_dir / f"a_secure_{args.mode}_{scale}.json"
        b_out = out_dir / f"b_note_{args.mode}_{scale}.json"
        arm_order = ["A_secure", "B_note"]
        if args.order_policy == "alternate" and idx % 2 == 1:
            arm_order = ["B_note", "A_secure"]

        for arm in arm_order:
            if arm == "A_secure":
                run_bench(
                    arm="A_secure",
                    mode=args.mode,
                    scale=scale,
                    out_file=a_out,
                    require_cert=args.require_cert,
                    lint=args.lint_output,
                    k_run=args.k_run,
                    input_profile=args.input_profile,
                )
            else:
                run_bench(
                    arm="B_note",
                    mode=args.mode,
                    scale=scale,
                    out_file=b_out,
                    require_cert=args.require_cert,
                    lint=args.lint_output,
                    k_run=args.k_run,
                    input_profile=args.input_profile,
                )

        a = load_json(a_out)
        b = load_json(b_out)
        summary_rows.append(
            {
                "scale": scale,
                "a_secure": {
                    "k_run": a["k_run"],
                    "prove_ms": float(a["prove_ms"]),
                    "verify_ms": float(a["verify_ms"]),
                    "peak_rss_mb": float(a["peak_rss_mb"]),
                    "physical_rows": int(a["physical_rows"]),
                },
                "b_note": {
                    "k_run": b["k_run"],
                    "prove_ms": float(b["prove_ms"]),
                    "verify_ms": float(b["verify_ms"]),
                    "peak_rss_mb": float(b["peak_rss_mb"]),
                    "physical_rows": int(b["physical_rows"]),
                },
                "ratios": {
                    "prove_b_over_a": ratio(float(b["prove_ms"]), float(a["prove_ms"])),
                    "verify_b_over_a": ratio(float(b["verify_ms"]), float(a["verify_ms"])),
                    "rss_b_over_a": ratio(float(b["peak_rss_mb"]), float(a["peak_rss_mb"])),
                    "rows_b_over_a": ratio(float(b["physical_rows"]), float(a["physical_rows"])),
                },
            }
        )

    summary = {
        "mode": args.mode,
        "order_policy": args.order_policy,
        "input_profile": args.input_profile,
        "scales": scales,
        "rows": summary_rows,
    }
    summary_path = out_dir / "summary.json"
    summary_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")

    out_md = Path(args.out_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    to_md_table(summary_rows, out_md, order_policy=args.order_policy, input_profile=args.input_profile)

    print(f"wrote local sweep summary: {summary_path}")
    print(f"wrote local sweep table: {out_md}")


if __name__ == "__main__":
    main()
