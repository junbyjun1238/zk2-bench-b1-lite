#!/usr/bin/env python3
import argparse
import json
import subprocess
from pathlib import Path
from statistics import mean
from typing import Dict, List


DEFAULT_SCALES = [2, 4, 8, 16, 24, 32, 48, 64, 80, 96, 128, 160]


def run_local_sweep(scales: List[int], k_run: int, out_dir: Path) -> Path:
    summary_path = out_dir / "summary.json"
    cmd = [
        "python",
        "scripts/local_sweep.py",
        "--scales",
        ",".join(str(s) for s in scales),
        "--mode",
        "full-local",
        "--lint-output",
        "--require-cert",
        "--k-run",
        str(k_run),
        "--out-dir",
        str(out_dir),
        "--out-md",
        str(out_dir / "table.md"),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(
            "\n".join(
                [
                    "local_sweep failed",
                    "stdout tail:",
                    "\n".join((proc.stdout or "").strip().splitlines()[-12:]),
                    "stderr tail:",
                    "\n".join((proc.stderr or "").strip().splitlines()[-12:]),
                ]
            )
        )
    if not summary_path.exists():
        raise RuntimeError(f"summary missing: {summary_path}")
    return summary_path


def bucket(occupancy: float) -> str:
    if occupancy < 0.25:
        return "low(<25%)"
    if occupancy < 0.60:
        return "mid(25-60%)"
    if occupancy < 0.90:
        return "mid-high(60-90%)"
    return "high(>=90%)"


def build_enriched(summary: Dict, k_run: int) -> Dict:
    cap = float(2**k_run)
    rows = []
    for row in summary["rows"]:
        physical_rows = float(row["a_secure"]["physical_rows"])
        occ = physical_rows / cap
        row = dict(row)
        row["occupancy"] = occ
        row["occupancy_bucket"] = bucket(occ)
        rows.append(row)

    grouped: Dict[str, List[Dict]] = {}
    for row in rows:
        grouped.setdefault(row["occupancy_bucket"], []).append(row)

    per_bucket = {}
    for key, vals in grouped.items():
        per_bucket[key] = {
            "num_points": len(vals),
            "scale_min": min(v["scale"] for v in vals),
            "scale_max": max(v["scale"] for v in vals),
            "avg_prove_b_over_a": mean(v["ratios"]["prove_b_over_a"] for v in vals),
            "avg_verify_b_over_a": mean(v["ratios"]["verify_b_over_a"] for v in vals),
            "avg_rss_b_over_a": mean(v["ratios"]["rss_b_over_a"] for v in vals),
        }

    return {
        "k_run_fixed": k_run,
        "capacity_rows": int(cap),
        "mode": summary["mode"],
        "scales": summary["scales"],
        "rows": rows,
        "bucket_summary": per_bucket,
    }


def write_markdown(enriched: Dict, out_md: Path) -> None:
    lines = [
        "# Local Full-Bench (Fixed-k) Report",
        "",
        f"- mode: `{enriched['mode']}`",
        f"- k_run_fixed: `{enriched['k_run_fixed']}`",
        f"- row_capacity(2^k): `{enriched['capacity_rows']}`",
        f"- scales: `{','.join(str(x) for x in enriched['scales'])}`",
        "",
        "## Point Table",
        "",
        "| scale | occupancy | bucket | A prove(ms) | B prove(ms) | B/A prove | A verify(ms) | B verify(ms) | B/A verify |",
        "|---:|---:|---|---:|---:|---:|---:|---:|---:|",
    ]
    for r in enriched["rows"]:
        lines.append(
            "| {scale} | {occ:.3f} | {bucket} | {a_p:.3f} | {b_p:.3f} | {rp:.3f} | {a_v:.3f} | {b_v:.3f} | {rv:.3f} |".format(
                scale=r["scale"],
                occ=r["occupancy"],
                bucket=r["occupancy_bucket"],
                a_p=r["a_secure"]["prove_ms"],
                b_p=r["b_note"]["prove_ms"],
                rp=r["ratios"]["prove_b_over_a"],
                a_v=r["a_secure"]["verify_ms"],
                b_v=r["b_note"]["verify_ms"],
                rv=r["ratios"]["verify_b_over_a"],
            )
        )

    lines.extend(["", "## Bucket Summary", ""])
    lines.append(
        "| bucket | points | scale range | avg B/A prove | avg B/A verify | avg B/A rss |"
    )
    lines.append("|---|---:|---:|---:|---:|---:|")

    for key in ["low(<25%)", "mid(25-60%)", "mid-high(60-90%)", "high(>=90%)"]:
        if key not in enriched["bucket_summary"]:
            continue
        s = enriched["bucket_summary"][key]
        lines.append(
            "| {b} | {n} | {mn}-{mx} | {rp:.3f} | {rv:.3f} | {rr:.3f} |".format(
                b=key,
                n=s["num_points"],
                mn=s["scale_min"],
                mx=s["scale_max"],
                rp=s["avg_prove_b_over_a"],
                rv=s["avg_verify_b_over_a"],
                rr=s["avg_rss_b_over_a"],
            )
        )

    lines.extend(
        [
            "",
            "Interpretation rule:",
            "- If `B/A prove` consistently decreases or stays < 1 across low+mid buckets, local-only trend is considered meaningful.",
            "- If trend is unstable across low+mid buckets, escalate to cloud/high-scale extension.",
            "",
        ]
    )
    out_md.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Fixed-k local full-bench runner with bucketed trend report.")
    parser.add_argument("--k-run", type=int, default=13)
    parser.add_argument("--scales", default=",".join(str(x) for x in DEFAULT_SCALES))
    parser.add_argument("--out-dir", default="benches/fullbench_local_fixedk")
    parser.add_argument("--out-md", default="docs/fullbench_local_fixedk.md")
    args = parser.parse_args()

    scales = [int(x.strip()) for x in args.scales.split(",") if x.strip()]
    if not scales:
        raise ValueError("empty scales")
    if any(s <= 0 for s in scales):
        raise ValueError("scales must be positive")

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    summary_path = run_local_sweep(scales, args.k_run, out_dir)
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    enriched = build_enriched(summary, args.k_run)

    enriched_path = out_dir / "enriched_summary.json"
    enriched_path.write_text(json.dumps(enriched, indent=2), encoding="utf-8")

    out_md = Path(args.out_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    write_markdown(enriched, out_md)

    print(f"wrote fixed-k summary: {summary_path}")
    print(f"wrote fixed-k enriched summary: {enriched_path}")
    print(f"wrote fixed-k report: {out_md}")


if __name__ == "__main__":
    main()
