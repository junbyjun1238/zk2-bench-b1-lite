#!/usr/bin/env python3
import argparse
import datetime
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


INSTABILITY_PATTERNS = (
    r"\bout of memory\b",
    r"\boom\b",
    r"\bswap\b",
    r"\bkilled\b",
    r"\bsegmentation fault\b",
    r"\bpanic\b",
)


@dataclass
class AttemptResult:
    arm: str
    scale: int
    success: bool
    reason: str
    out_file: Optional[str]
    prove_ms: Optional[float]
    peak_rss_mb: Optional[float]
    status: Optional[str]
    raw_stdout_tail: str
    raw_stderr_tail: str


def detect_instability(stdout: str, stderr: str) -> bool:
    haystack = f"{stdout}\n{stderr}".lower()
    return any(re.search(pattern, haystack) for pattern in INSTABILITY_PATTERNS)


def run_one(
    arm: str,
    scale: int,
    mode: str,
    out_dir: Path,
    require_cert: bool,
    lint_output: bool,
    rss_limit_mb: float,
    prove_limit_min: float,
) -> AttemptResult:
    out_file = out_dir / f"{arm}_{mode}_scale_{scale}.json"
    cmd = [
        "python",
        "scripts/run_ab_bench.py",
        "--arm",
        arm,
        "--mode",
        mode,
        "--scale",
        str(scale),
        "--out",
        str(out_file),
    ]
    if require_cert and arm == "B_note":
        cmd.append("--require-cert")
    if lint_output:
        cmd.append("--lint-output")

    proc = subprocess.run(cmd, capture_output=True, text=True)
    stdout = proc.stdout or ""
    stderr = proc.stderr or ""
    stdout_tail = "\n".join(stdout.strip().splitlines()[-6:])
    stderr_tail = "\n".join(stderr.strip().splitlines()[-6:])

    if proc.returncode != 0:
        reason = "process-failed"
        if detect_instability(stdout, stderr):
            reason = "instability-signal"
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason=reason,
            out_file=str(out_file) if out_file.exists() else None,
            prove_ms=None,
            peak_rss_mb=None,
            status=None,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )

    if not out_file.exists():
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason="missing-output-json",
            out_file=None,
            prove_ms=None,
            peak_rss_mb=None,
            status=None,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )

    payload = json.loads(out_file.read_text(encoding="utf-8"))
    prove_ms = float(payload.get("prove_ms", 0.0))
    peak_rss_mb = float(payload.get("peak_rss_mb", 0.0))
    status = str(payload.get("status", "unknown"))
    prove_limit_ms = prove_limit_min * 60_000.0

    if peak_rss_mb > rss_limit_mb:
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason=f"rss-limit-exceeded({peak_rss_mb:.2f}>{rss_limit_mb:.2f})",
            out_file=str(out_file),
            prove_ms=prove_ms,
            peak_rss_mb=peak_rss_mb,
            status=status,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )
    if prove_ms > prove_limit_ms:
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason=f"prove-time-limit-exceeded({prove_ms:.2f}>{prove_limit_ms:.2f})",
            out_file=str(out_file),
            prove_ms=prove_ms,
            peak_rss_mb=peak_rss_mb,
            status=status,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )
    if detect_instability(stdout, stderr):
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason="instability-signal",
            out_file=str(out_file),
            prove_ms=prove_ms,
            peak_rss_mb=peak_rss_mb,
            status=status,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )
    if "ok" not in status:
        return AttemptResult(
            arm=arm,
            scale=scale,
            success=False,
            reason=f"non-ok-status({status})",
            out_file=str(out_file),
            prove_ms=prove_ms,
            peak_rss_mb=peak_rss_mb,
            status=status,
            raw_stdout_tail=stdout_tail,
            raw_stderr_tail=stderr_tail,
        )

    return AttemptResult(
        arm=arm,
        scale=scale,
        success=True,
        reason="success",
        out_file=str(out_file),
        prove_ms=prove_ms,
        peak_rss_mb=peak_rss_mb,
        status=status,
        raw_stdout_tail=stdout_tail,
        raw_stderr_tail=stderr_tail,
    )


def doubling_search(
    arm: str,
    start_scale: int,
    max_scale: int,
    mode: str,
    out_dir: Path,
    require_cert: bool,
    lint_output: bool,
    rss_limit_mb: float,
    prove_limit_min: float,
) -> Tuple[List[AttemptResult], Optional[int], Optional[int]]:
    attempts: List[AttemptResult] = []
    scale = start_scale
    last_success: Optional[int] = None
    first_failure: Optional[int] = None

    while True:
        result = run_one(
            arm=arm,
            scale=scale,
            mode=mode,
            out_dir=out_dir,
            require_cert=require_cert,
            lint_output=lint_output,
            rss_limit_mb=rss_limit_mb,
            prove_limit_min=prove_limit_min,
        )
        attempts.append(result)
        if result.success:
            last_success = scale
            next_scale = scale * 2
            if next_scale > max_scale:
                break
            scale = next_scale
            continue

        first_failure = scale
        break

    return attempts, last_success, first_failure


def binary_refine(
    arm: str,
    left_success: int,
    right_failure: int,
    mode: str,
    out_dir: Path,
    require_cert: bool,
    lint_output: bool,
    rss_limit_mb: float,
    prove_limit_min: float,
) -> Tuple[List[AttemptResult], int, int]:
    attempts: List[AttemptResult] = []
    l = left_success
    r = right_failure

    while l + 1 < r:
        mid = (l + r) // 2
        result = run_one(
            arm=arm,
            scale=mid,
            mode=mode,
            out_dir=out_dir,
            require_cert=require_cert,
            lint_output=lint_output,
            rss_limit_mb=rss_limit_mb,
            prove_limit_min=prove_limit_min,
        )
        attempts.append(result)
        if result.success:
            l = mid
        else:
            r = mid
    return attempts, l, r


def to_summary_entry(
    arm: str,
    start_scale: int,
    max_scale: int,
    mode: str,
    out_dir: Path,
    require_cert: bool,
    lint_output: bool,
    rss_limit_mb: float,
    prove_limit_min: float,
) -> Dict:
    doubling_attempts, last_success, first_failure = doubling_search(
        arm=arm,
        start_scale=start_scale,
        max_scale=max_scale,
        mode=mode,
        out_dir=out_dir,
        require_cert=require_cert,
        lint_output=lint_output,
        rss_limit_mb=rss_limit_mb,
        prove_limit_min=prove_limit_min,
    )
    all_attempts = list(doubling_attempts)
    binary_attempts: List[AttemptResult] = []

    feasible_max = last_success
    failure_scale = first_failure
    stop_reason = "reached-max-scale" if first_failure is None else "found-failure"

    if last_success is not None and first_failure is not None and first_failure - last_success > 1:
        binary_attempts, feasible_max, failure_scale = binary_refine(
            arm=arm,
            left_success=last_success,
            right_failure=first_failure,
            mode=mode,
            out_dir=out_dir,
            require_cert=require_cert,
            lint_output=lint_output,
            rss_limit_mb=rss_limit_mb,
            prove_limit_min=prove_limit_min,
        )
        all_attempts.extend(binary_attempts)

    if first_failure is not None:
        stop_reason = next(
            (x.reason for x in all_attempts if x.scale == failure_scale and not x.success),
            "found-failure",
        )

    entry = {
        "arm": arm,
        "mode": mode,
        "start_scale": start_scale,
        "max_scale": max_scale,
        "rss_limit_mb": rss_limit_mb,
        "prove_limit_min": prove_limit_min,
        "feasible_max_scale": feasible_max,
        "first_failure_scale": failure_scale,
        "stop_reason": stop_reason,
        "attempt_count": len(all_attempts),
        "attempts": [
            {
                "scale": a.scale,
                "success": a.success,
                "reason": a.reason,
                "out_file": a.out_file,
                "prove_ms": a.prove_ms,
                "peak_rss_mb": a.peak_rss_mb,
                "status": a.status,
                "stdout_tail": a.raw_stdout_tail,
                "stderr_tail": a.raw_stderr_tail,
            }
            for a in all_attempts
        ],
    }
    return entry


def write_markdown_report(summary: Dict, out_md: Path) -> None:
    lines = [
        "# Local Feasible Scale Search Report",
        "",
        f"- mode: `{summary['mode']}`",
        f"- generated_at_utc: `{summary['generated_at_utc']}`",
        f"- rss_limit_mb: `{summary['rss_limit_mb']}`",
        f"- prove_limit_min: `{summary['prove_limit_min']}`",
        "",
    ]

    for item in summary["results"]:
        lines.extend(
            [
                f"## Arm `{item['arm']}`",
                f"- feasible_max_scale: `{item['feasible_max_scale']}`",
                f"- first_failure_scale: `{item['first_failure_scale']}`",
                f"- stop_reason: `{item['stop_reason']}`",
                f"- attempts: `{item['attempt_count']}`",
                "",
                "| scale | success | reason | prove_ms | peak_rss_mb | out_file |",
                "|---:|:---:|---|---:|---:|---|",
            ]
        )
        for attempt in item["attempts"]:
            lines.append(
                "| {scale} | {success} | {reason} | {prove_ms} | {peak_rss_mb} | `{out_file}` |".format(
                    scale=attempt["scale"],
                    success="Y" if attempt["success"] else "N",
                    reason=attempt["reason"],
                    prove_ms=(
                        f"{attempt['prove_ms']:.2f}" if attempt["prove_ms"] is not None else "n/a"
                    ),
                    peak_rss_mb=(
                        f"{attempt['peak_rss_mb']:.2f}"
                        if attempt["peak_rss_mb"] is not None
                        else "n/a"
                    ),
                    out_file=attempt["out_file"] or "n/a",
                )
            )
        lines.append("")

    out_md.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Automated local feasible-scale search (doubling + binary refinement)."
    )
    parser.add_argument(
        "--arms",
        default="A_secure,B_note",
        help="Comma-separated arms to search (default: A_secure,B_note)",
    )
    parser.add_argument(
        "--mode",
        default="full-local",
        choices=["fast-structural", "full-local"],
    )
    parser.add_argument("--start-scale", type=int, default=2)
    parser.add_argument("--max-scale", type=int, default=256)
    parser.add_argument("--rss-limit-mb", type=float, default=13312.0)
    parser.add_argument("--prove-limit-min", type=float, default=45.0)
    parser.add_argument("--require-cert", action="store_true")
    parser.add_argument("--lint-output", action="store_true")
    parser.add_argument(
        "--out-json",
        default="benches/scale_search/summary.json",
    )
    parser.add_argument(
        "--out-md",
        default="docs/scale_search_report.md",
    )
    args = parser.parse_args()

    arms = [x.strip() for x in args.arms.split(",") if x.strip()]
    for arm in arms:
        if arm not in {"U", "A_secure", "B_note"}:
            raise ValueError(f"unsupported arm: {arm}")
    if args.start_scale < 1:
        raise ValueError("--start-scale must be >= 1")
    if args.max_scale < args.start_scale:
        raise ValueError("--max-scale must be >= --start-scale")

    out_json = Path(args.out_json)
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_dir = out_json.parent
    results: List[Dict] = []

    for arm in arms:
        entry = to_summary_entry(
            arm=arm,
            start_scale=args.start_scale,
            max_scale=args.max_scale,
            mode=args.mode,
            out_dir=out_dir,
            require_cert=args.require_cert,
            lint_output=args.lint_output,
            rss_limit_mb=args.rss_limit_mb,
            prove_limit_min=args.prove_limit_min,
        )
        results.append(entry)

    generated_at = datetime.datetime.utcnow().isoformat() + "Z"
    summary = {
        "mode": args.mode,
        "start_scale": args.start_scale,
        "max_scale": args.max_scale,
        "rss_limit_mb": args.rss_limit_mb,
        "prove_limit_min": args.prove_limit_min,
        "generated_at_utc": generated_at,
        "results": results,
    }

    out_json.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    out_md = Path(args.out_md)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    write_markdown_report(summary, out_md)

    print(f"wrote summary json: {out_json}")
    print(f"wrote report markdown: {out_md}")
    for entry in results:
        print(
            f"[{entry['arm']}] feasible_max_scale={entry['feasible_max_scale']}, "
            f"first_failure_scale={entry['first_failure_scale']}, stop={entry['stop_reason']}"
        )


if __name__ == "__main__":
    main()
