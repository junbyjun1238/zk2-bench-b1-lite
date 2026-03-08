#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt


def load_summary(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def plot_times(summary: dict, out_path: Path) -> None:
    scales = [row["scale"] for row in summary["rows"]]
    a_prove = [row["a_secure"]["prove_ms"] for row in summary["rows"]]
    b_prove = [row["b_note"]["prove_ms"] for row in summary["rows"]]
    a_verify = [row["a_secure"]["verify_ms"] for row in summary["rows"]]
    b_verify = [row["b_note"]["verify_ms"] for row in summary["rows"]]

    fig, axes = plt.subplots(1, 2, figsize=(12, 4.5), dpi=160)

    axes[0].plot(scales, a_prove, marker="o", label="A_secure")
    axes[0].plot(scales, b_prove, marker="o", label="B_note")
    axes[0].set_title("Prove Time vs Scale")
    axes[0].set_xlabel("scale")
    axes[0].set_ylabel("prove_ms")
    axes[0].grid(alpha=0.3)
    axes[0].legend()

    axes[1].plot(scales, a_verify, marker="o", label="A_secure")
    axes[1].plot(scales, b_verify, marker="o", label="B_note")
    axes[1].set_title("Verify Time vs Scale")
    axes[1].set_xlabel("scale")
    axes[1].set_ylabel("verify_ms")
    axes[1].grid(alpha=0.3)
    axes[1].legend()

    fig.tight_layout()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def plot_ratios(summary: dict, out_path: Path) -> None:
    scales = [row["scale"] for row in summary["rows"]]
    prove_ratio = [row["ratios"]["prove_b_over_a"] for row in summary["rows"]]
    verify_ratio = [row["ratios"]["verify_b_over_a"] for row in summary["rows"]]
    rss_ratio = [row["ratios"]["rss_b_over_a"] for row in summary["rows"]]

    fig, ax = plt.subplots(figsize=(8.5, 4.5), dpi=160)
    ax.plot(scales, prove_ratio, marker="o", label="B/A prove")
    ax.plot(scales, verify_ratio, marker="o", label="B/A verify")
    ax.plot(scales, rss_ratio, marker="o", label="B/A rss")
    ax.axhline(1.0, color="black", linewidth=1.0, linestyle="--", alpha=0.8)
    ax.set_title("B/A Ratios vs Scale")
    ax.set_xlabel("scale")
    ax.set_ylabel("ratio")
    ax.grid(alpha=0.3)
    ax.legend()

    fig.tight_layout()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def plot_occupancy(summary: dict, k_run: int, out_path: Path) -> None:
    cap = float(2**k_run)
    scales = [row["scale"] for row in summary["rows"]]
    occupancy = [row["a_secure"]["physical_rows"] / cap for row in summary["rows"]]

    fig, ax = plt.subplots(figsize=(8.5, 4.5), dpi=160)
    ax.plot(scales, occupancy, marker="o", label="row occupancy")
    ax.axhline(0.25, color="#999999", linestyle="--", linewidth=1.0, label="25%")
    ax.axhline(0.60, color="#666666", linestyle="--", linewidth=1.0, label="60%")
    ax.axhline(0.90, color="#333333", linestyle="--", linewidth=1.0, label="90%")
    ax.set_title(f"Occupancy vs Scale (k={k_run}, cap=2^{k_run})")
    ax.set_xlabel("scale")
    ax.set_ylabel("occupancy")
    ax.grid(alpha=0.3)
    ax.legend()

    fig.tight_layout()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot fixed-k local full-bench graphs.")
    parser.add_argument(
        "--summary",
        default="benches/fullbench_local_fixedk/summary.json",
        help="Path to fixed-k summary.json",
    )
    parser.add_argument("--k-run", type=int, default=13)
    parser.add_argument("--out-dir", default="docs/figures")
    args = parser.parse_args()

    summary_path = Path(args.summary)
    if not summary_path.exists():
        raise FileNotFoundError(f"summary file not found: {summary_path}")

    summary = load_summary(summary_path)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    out_times = out_dir / "fullbench_fixedk_times.png"
    out_ratios = out_dir / "fullbench_fixedk_ratios.png"
    out_occ = out_dir / "fullbench_fixedk_occupancy.png"

    plot_times(summary, out_times)
    plot_ratios(summary, out_ratios)
    plot_occupancy(summary, args.k_run, out_occ)

    print(f"wrote: {out_times}")
    print(f"wrote: {out_ratios}")
    print(f"wrote: {out_occ}")


if __name__ == "__main__":
    main()
