#!/usr/bin/env python3
import argparse
import json
from pathlib import Path
from statistics import mean, pstdev
from typing import List
import subprocess

NUMERIC_KEYS = [
    "keygen_vk_ms",
    "keygen_pk_ms",
    "prove_ms",
    "verify_ms",
    "peak_rss_mb",
    "proof_bytes",
]


def run_once(arm: str, scale: int, k_run: int, out_path: Path, require_cert: bool) -> dict:
    cmd = [
        "python",
        "scripts/run_ab_bench.py",
        "--arm",
        arm,
        "--mode",
        "full-local",
        "--scale",
        str(scale),
        "--k-run",
        str(k_run),
        "--lint-output",
        "--out",
        str(out_path),
    ]
    if require_cert and arm == "B_note":
        cmd.append("--require-cert")
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(
            "\n".join(
                [
                    f"command failed: {' '.join(cmd)}",
                    "stdout tail:",
                    "\n".join((proc.stdout or "").strip().splitlines()[-10:]),
                    "stderr tail:",
                    "\n".join((proc.stderr or "").strip().splitlines()[-10:]),
                ]
            )
        )
    return json.loads(out_path.read_text(encoding="utf-8"))


def summarize(samples: List[dict]) -> dict:
    out = {}
    for key in NUMERIC_KEYS:
        values = [float(s[key]) for s in samples]
        out[key] = {
            "mean": mean(values),
            "std": pstdev(values) if len(values) > 1 else 0.0,
            "min": min(values),
            "max": max(values),
        }
    out["k_run"] = int(samples[0]["k_run"])
    out["physical_rows"] = int(samples[0]["physical_rows"])
    return out


def format_pm(mu: float, sigma: float, digits: int = 2) -> str:
    return f"{mu:.{digits}f} +/- {sigma:.{digits}f}"


def build_md(rows: List[dict], out_path: Path, repeats: int, k_run: int, order_policy: str) -> None:
    lines = [
        "# Local Repeat Bench (full-local, fixed-k)",
        "",
        f"- repeats per point: `{repeats}`",
        f"- k_run fixed: `{k_run}`",
        f"- order policy: `{order_policy}`",
        "",
        "| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |",
        "|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for row in rows:
        a = row["a_secure"]
        b = row["b_note"]
        ratio = row["ratio"]
        a_keygen_mu = a["keygen_vk_ms"]["mean"] + a["keygen_pk_ms"]["mean"]
        b_keygen_mu = b["keygen_vk_ms"]["mean"] + b["keygen_pk_ms"]["mean"]
        lines.append(
            "| {s} | {a_p} | {b_p} | {r_p:.3f} | {a_v} | {b_v} | {r_v:.3f} | {a_k:.2f} | {b_k:.2f} | {a_pb:.0f} | {b_pb:.0f} |".format(
                s=row["scale"],
                a_p=format_pm(a["prove_ms"]["mean"], a["prove_ms"]["std"]),
                b_p=format_pm(b["prove_ms"]["mean"], b["prove_ms"]["std"]),
                r_p=ratio["prove_b_over_a_mean"],
                a_v=format_pm(a["verify_ms"]["mean"], a["verify_ms"]["std"]),
                b_v=format_pm(b["verify_ms"]["mean"], b["verify_ms"]["std"]),
                r_v=ratio["verify_b_over_a_mean"],
                a_k=a_keygen_mu,
                b_k=b_keygen_mu,
                a_pb=a["proof_bytes"]["mean"],
                b_pb=b["proof_bytes"]["mean"],
            )
        )

    lines.extend(
        [
            "",
            "Notes:",
            "- Ratios are computed from mean metrics (B_mean / A_mean).",
            "- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.",
            "- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.",
            "",
        ]
    )
    out_path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Repeat full-local A/B benchmark at fixed-k and summarize mean/std."
    )
    parser.add_argument("--scales", default="16,24,32")
    parser.add_argument("--k-run", type=int, default=13)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--out-dir", default="benches/repeat_local")
    parser.add_argument("--out-md", default="docs/repeat_local_report.md")
    parser.add_argument("--order-policy", choices=["fixed-ab", "alternate"], default="alternate")
    args = parser.parse_args()

    if args.repeats < 1:
        raise ValueError("--repeats must be >= 1")
    scales = [int(x.strip()) for x in args.scales.split(",") if x.strip()]
    if not scales:
        raise ValueError("empty scales")
    if any(s <= 0 for s in scales):
        raise ValueError("scales must be positive")

    out_dir = Path(args.out_dir)
    runs_dir = out_dir / "runs"
    runs_dir.mkdir(parents=True, exist_ok=True)

    rows = []
    for scale in scales:
        a_samples = []
        b_samples = []
        for rep in range(1, args.repeats + 1):
            a_out = runs_dir / f"a_secure_s{scale}_r{rep}.json"
            b_out = runs_dir / f"b_note_s{scale}_r{rep}.json"
            arm_order = ["A_secure", "B_note"]
            if args.order_policy == "alternate" and rep % 2 == 0:
                arm_order = ["B_note", "A_secure"]
            for arm in arm_order:
                if arm == "A_secure":
                    a_samples.append(run_once("A_secure", scale, args.k_run, a_out, require_cert=False))
                else:
                    b_samples.append(run_once("B_note", scale, args.k_run, b_out, require_cert=True))

        a_sum = summarize(a_samples)
        b_sum = summarize(b_samples)
        rows.append(
            {
                "scale": scale,
                "a_secure": a_sum,
                "b_note": b_sum,
                "ratio": {
                    "prove_b_over_a_mean": b_sum["prove_ms"]["mean"] / a_sum["prove_ms"]["mean"],
                    "verify_b_over_a_mean": b_sum["verify_ms"]["mean"] / a_sum["verify_ms"]["mean"],
                    "rss_b_over_a_mean": b_sum["peak_rss_mb"]["mean"] / a_sum["peak_rss_mb"]["mean"],
                },
            }
        )

    summary = {
        "mode": "full-local",
        "k_run": args.k_run,
        "repeats": args.repeats,
        "order_policy": args.order_policy,
        "scales": scales,
        "rows": rows,
    }
    summary_path = out_dir / "summary.json"
    summary_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")

    out_md = Path(args.out_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    build_md(rows, out_md, repeats=args.repeats, k_run=args.k_run, order_policy=args.order_policy)

    print(f"wrote repeat summary: {summary_path}")
    print(f"wrote repeat report: {out_md}")


if __name__ == "__main__":
    main()
