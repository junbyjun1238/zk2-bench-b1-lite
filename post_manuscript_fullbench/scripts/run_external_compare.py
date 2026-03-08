#!/usr/bin/env python3
import argparse
import json
import platform
import subprocess
import time
from pathlib import Path

try:
    import psutil
except Exception:
    psutil = None


def _extract_last_json_object(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            return json.loads(line)
    raise ValueError("no JSON object found in process output")


def _run_with_peak_rss(cmd: list[str], cwd: Path | None = None):
    proc = subprocess.Popen(
        cmd,
        cwd=str(cwd) if cwd is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    peak_rss_bytes = 0
    proc_ps = psutil.Process(proc.pid) if psutil is not None else None

    def sample_rss():
        nonlocal peak_rss_bytes
        if proc_ps is None:
            return
        try:
            peak_rss_bytes = max(peak_rss_bytes, proc_ps.memory_info().rss)
            for child in proc_ps.children(recursive=True):
                try:
                    peak_rss_bytes = max(peak_rss_bytes, child.memory_info().rss)
                except psutil.Error:
                    continue
        except psutil.Error:
            return

    while proc.poll() is None:
        sample_rss()
        time.sleep(0.02)
    sample_rss()
    stdout, stderr = proc.communicate()
    return proc.returncode, stdout or "", stderr or "", peak_rss_bytes / (1024.0 * 1024.0)


def _bin_path(bin_name: str) -> Path:
    ext = ".exe" if platform.system().lower().startswith("win") else ""
    return Path("target") / "debug" / f"{bin_name}{ext}"


def _ensure_bin(bin_name: str):
    subprocess.run(["cargo", "build", "--quiet", "--bin", bin_name], check=True)
    path = _bin_path(bin_name)
    if not path.exists():
        raise RuntimeError(f"compiled binary not found: {path}")
    return path


def _run_ab(arm: str, scale: int, k_run: int, out_path: Path, require_cert: bool):
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
        "--out",
        str(out_path),
    ]
    if require_cert and arm == "B_note":
        cmd.append("--require-cert")
    subprocess.run(cmd, check=True)
    return json.loads(out_path.read_text(encoding="utf-8"))


def _run_ext(scale: int, k_run: int, out_path: Path):
    bin_path = _ensure_bin("ext_halo2wrong_full_local")
    cmd = [str(bin_path), "--scale", str(scale), "--k-run", str(k_run)]
    rc, stdout, stderr, peak_rss_mb = _run_with_peak_rss(cmd)
    if rc != 0:
        tail = stderr.strip().splitlines()[-1] if stderr.strip() else "unknown error"
        raise RuntimeError(f"ext_halo2wrong_full_local failed: {tail}")
    payload = _extract_last_json_object(stdout)
    payload["peak_rss_mb"] = peak_rss_mb
    out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return payload


def _f(v):
    return f"{v:.3f}"


def main():
    parser = argparse.ArgumentParser(
        description="Run external standard-library comparison (A_secure vs B_note vs ext_halo2wrong)."
    )
    parser.add_argument("--scales", default="16,32")
    parser.add_argument("--k-run", type=int, default=13)
    parser.add_argument("--out-dir", default="benches/external_h2w_compare")
    parser.add_argument("--report", default="docs/external_h2w_compare.md")
    parser.add_argument("--require-cert", action="store_true")
    args = parser.parse_args()

    scales = [int(x.strip()) for x in args.scales.split(",") if x.strip()]
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    rows = []
    for scale in scales:
        a_path = out_dir / f"a_secure_s{scale}.json"
        b_path = out_dir / f"b_note_s{scale}.json"
        e_path = out_dir / f"ext_h2w_s{scale}.json"

        a = _run_ab("A_secure", scale, args.k_run, a_path, require_cert=False)
        b = _run_ab("B_note", scale, args.k_run, b_path, require_cert=args.require_cert)
        e = _run_ext(scale, args.k_run, e_path)

        rows.append(
            {
                "scale": scale,
                "A_secure": a,
                "B_note": b,
                "ext_halo2wrong": e,
                "ratio_B_over_A_prove": b["prove_ms"] / a["prove_ms"],
                "ratio_B_over_ext_prove": b["prove_ms"] / e["prove_ms"],
                "ratio_B_over_A_verify": b["verify_ms"] / a["verify_ms"],
                "ratio_B_over_ext_verify": b["verify_ms"] / e["verify_ms"],
            }
        )

    summary = {
        "mode": "full-local",
        "k_run": args.k_run,
        "scales": scales,
        "rows": rows,
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

    lines = [
        "# External Standard-Library Comparison (full-local)",
        "",
        f"- k_run: `{args.k_run}`",
        f"- scales: `{','.join(str(s) for s in scales)}`",
        "",
        "## Prove/Verify Time Table (ms)",
        "",
        "| scale | A prove | B prove | ext prove | B/A | B/ext | A verify | B verify | ext verify | B/A | B/ext |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for r in rows:
        a = r["A_secure"]
        b = r["B_note"]
        e = r["ext_halo2wrong"]
        lines.append(
            "| {scale} | {ap} | {bp} | {ep} | {rba} | {rbe} | {av} | {bv} | {ev} | {rbav} | {rbev} |".format(
                scale=r["scale"],
                ap=_f(a["prove_ms"]),
                bp=_f(b["prove_ms"]),
                ep=_f(e["prove_ms"]),
                rba=_f(r["ratio_B_over_A_prove"]),
                rbe=_f(r["ratio_B_over_ext_prove"]),
                av=_f(a["verify_ms"]),
                bv=_f(b["verify_ms"]),
                ev=_f(e["verify_ms"]),
                rbav=_f(r["ratio_B_over_A_verify"]),
                rbev=_f(r["ratio_B_over_ext_verify"]),
            )
        )

    lines.extend(
        [
            "",
            "## Notes",
            "",
            "- `ext_halo2wrong` uses row-family-equivalent `maingate::MainGate::to_bits` decomposition workload from external standard library.",
            "- This external arm is a decomposition-only baseline (it does not include this repo's full row-family relation gates, digest binding, or certificate checks).",
            "- This is an external library baseline and is intentionally kept separate from frozen `v1.0.0` arm schema.",
            "",
            f"Raw outputs: `{out_dir.as_posix()}`",
        ]
    )

    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote summary: {out_dir / 'summary.json'}")
    print(f"wrote report: {report_path}")


if __name__ == "__main__":
    main()
