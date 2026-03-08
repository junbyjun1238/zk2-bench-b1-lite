#!/usr/bin/env python3
import argparse
import json
import math
import platform
import subprocess
import time
from pathlib import Path

try:
    import psutil
except Exception:  # pragma: no cover - fallback path when psutil is unavailable
    psutil = None


def safe_cmd_output(cmd):
    try:
        return subprocess.check_output(cmd, stderr=subprocess.DEVNULL).decode().strip()
    except Exception:
        return "unknown"


def _extract_last_json_object(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            return json.loads(line)
    raise ValueError("no JSON object found in process output")


def _run_with_peak_rss(cmd):
    proc = subprocess.Popen(
        cmd,
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
    return proc.returncode, stdout, stderr, (peak_rss_bytes / (1024.0 * 1024.0))


def _bin_path(bin_name: str) -> Path:
    ext = ".exe" if platform.system().lower().startswith("win") else ""
    return Path("target") / "debug" / f"{bin_name}{ext}"


def _ensure_bin(bin_name: str) -> Path:
    path = _bin_path(bin_name)
    subprocess.run(
        ["cargo", "build", "--quiet", "--bin", bin_name],
        check=True,
    )
    if not path.exists():
        raise RuntimeError(f"compiled binary not found: {path}")
    return path


def build_a_secure_fast_structural(scale: int) -> dict:
    # A_secure row-family equivalent structure (src/baseline_a/mod.rs):
    # - explicit bit decomposition for x/y/z/q on all active rows
    # - no-equality guards for x/y/z
    # - family equations + digest binding + inactive-row zero extension
    rows_per_rep = 29
    physical_rows = rows_per_rep * scale
    k_min = max(10, math.ceil(math.log2(max(physical_rows, 2))))
    return {
        "n": scale,
        "R_hor": scale,
        "R_car": scale,
        "k_min": k_min,
        "k_run": k_min,
        "logical_lookup_cells": 0,
        "logical_mul_constraints": 4560 * scale,
        "logical_lin_constraints": 146 * scale,
        "physical_rows": physical_rows,
        "advice_cols": 167,
        "fixed_cols": 6,
        "instance_cols": 0,
        "synth_ms": 0.0,
        "keygen_vk_ms": 0.0,
        "keygen_pk_ms": 0.0,
        "prove_ms": 0.0,
        "verify_ms": 0.0,
        "peak_rss_mb": 0.0,
        "proof_bytes": 0,
        "status": "structural-ok",
        "notes": (
            "A_secure fast-structural counters from row-family-equivalent explicit "
            "bit decomposition baseline; "
            "no proof generation in this mode"
        ),
    }


def build_b_note_fast_structural(scale: int) -> dict:
    lookup_per_rep = 284
    mul_per_rep = 108
    lin_per_rep = 146
    rows_per_rep = 29
    physical_rows = rows_per_rep * scale
    k_min = max(13, math.ceil(math.log2(max(physical_rows, 2))))
    return {
        "n": scale,
        "R_hor": scale,
        "R_car": scale,
        "k_min": k_min,
        "k_run": k_min,
        "logical_lookup_cells": lookup_per_rep * scale,
        "logical_mul_constraints": mul_per_rep * scale,
        "logical_lin_constraints": lin_per_rep * scale,
        "physical_rows": physical_rows,
        "advice_cols": 19,
        "fixed_cols": 9,
        "instance_cols": 0,
        "synth_ms": 0.0,
        "keygen_vk_ms": 0.0,
        "keygen_pk_ms": 0.0,
        "prove_ms": 0.0,
        "verify_ms": 0.0,
        "peak_rss_mb": 0.0,
        "proof_bytes": 0,
        "status": "structural-ok",
        "notes": (
            "B_note fast-structural counters from baseline_b template "
            "(family rows + canonical/q-binding + digest + inactive-zero checks)"
        ),
    }


def build_a_secure_full_local(scale: int, k_run: int | None = None) -> dict:
    bin_path = _ensure_bin("a_secure_full_local")
    cmd = [
        str(bin_path),
        "--scale",
        str(scale),
    ]
    if k_run is not None:
        cmd.extend(["--k-run", str(k_run)])
    rc, stdout, stderr, peak_rss_mb = _run_with_peak_rss(cmd)
    stdout = stdout or ""
    stderr = stderr or ""
    if rc != 0:
        stderr_tail = stderr.strip().splitlines()[-1] if stderr.strip() else "unknown error"
        raise RuntimeError(f"A_secure full-local failed: {stderr_tail}")

    payload = _extract_last_json_object(stdout)
    payload["peak_rss_mb"] = peak_rss_mb
    return payload


def build_b_note_full_local(scale: int, k_run: int | None = None) -> dict:
    bin_path = _ensure_bin("b_note_full_local")
    cmd = [
        str(bin_path),
        "--scale",
        str(scale),
    ]
    if k_run is not None:
        cmd.extend(["--k-run", str(k_run)])
    rc, stdout, stderr, peak_rss_mb = _run_with_peak_rss(cmd)
    stdout = stdout or ""
    stderr = stderr or ""
    if rc != 0:
        stderr_tail = stderr.strip().splitlines()[-1] if stderr.strip() else "unknown error"
        raise RuntimeError(f"B_note full-local failed: {stderr_tail}")

    payload = _extract_last_json_object(stdout)
    payload["peak_rss_mb"] = peak_rss_mb
    return payload


def build_mock_payload(scale: int) -> dict:
    return {
        "n": scale,
        "R_hor": scale * 2,
        "R_car": scale,
        "k_min": 17,
        "k_run": 17,
        "logical_lookup_cells": 152,
        "logical_mul_constraints": 56,
        "logical_lin_constraints": 56,
        "physical_rows": 72,
        "advice_cols": 8,
        "fixed_cols": 4,
        "instance_cols": 1,
        "synth_ms": 1.0,
        "keygen_vk_ms": 1.0,
        "keygen_pk_ms": 1.0,
        "prove_ms": 1.0,
        "verify_ms": 1.0,
        "peak_rss_mb": 128.0,
        "proof_bytes": 1024,
        "status": "mock-ok",
        "notes": "mock run only; no Halo2 circuit execution",
    }


def main():
    parser = argparse.ArgumentParser(description="Mock A/B benchmark runner")
    parser.add_argument("--arm", choices=["U", "A_secure", "B_note"], required=True)
    parser.add_argument(
        "--mode",
        choices=["fast-structural", "full-local", "full-cloud"],
        required=True,
    )
    parser.add_argument("--scale", type=int, default=1)
    parser.add_argument("--k-run", type=int, default=None)
    parser.add_argument("--family", default="demo_family")
    parser.add_argument("--package-type", default="demo_package")
    parser.add_argument("--out", default="benches/mock_result.json")
    parser.add_argument("--require-cert", action="store_true")
    parser.add_argument("--lint-output", action="store_true")
    parser.add_argument("--cert-path", default="certificates/public_certificate.json")
    parser.add_argument("--manuscript", default="core_papers/wrapper_note_option2.tex")
    parser.add_argument("--backend-instance", default="certificates/h2dq_backend_instance.json")
    args = parser.parse_args()

    print(f"running... arm={args.arm}, mode={args.mode}, scale={args.scale}")

    commit_hash = safe_cmd_output(["git", "rev-parse", "HEAD"])
    rust_version = safe_cmd_output(["rustc", "--version"])
    backend_commit = safe_cmd_output(["git", "rev-parse", "HEAD"])

    if args.require_cert and args.arm == "B_note":
        cert_cmd = [
            "python",
            "scripts/h2dq_cert.py",
            "--certificate",
            args.cert_path,
            "--manuscript",
            args.manuscript,
            "--backend-instance",
            args.backend_instance,
        ]
        cert_proc = subprocess.run(cert_cmd)
        if cert_proc.returncode != 0:
            raise RuntimeError("certificate check failed for B_note run")

    if args.arm == "A_secure" and args.mode == "fast-structural":
        payload = build_a_secure_fast_structural(args.scale)
    elif args.arm == "A_secure" and args.mode == "full-local":
        payload = build_a_secure_full_local(args.scale, args.k_run)
    elif args.arm == "B_note" and args.mode == "fast-structural":
        payload = build_b_note_fast_structural(args.scale)
    elif args.arm == "B_note" and args.mode == "full-local":
        payload = build_b_note_full_local(args.scale, args.k_run)
    else:
        payload = build_mock_payload(args.scale)

    result = {
        "arm": args.arm,
        "mode": args.mode,
        "family": args.family,
        "package_type": args.package_type,
        "workload_scale": args.scale,
        "n": payload["n"],
        "R_hor": payload["R_hor"],
        "R_car": payload["R_car"],
        "k_min": payload["k_min"],
        "k_run": payload["k_run"],
        "logical_lookup_cells": payload["logical_lookup_cells"],
        "logical_mul_constraints": payload["logical_mul_constraints"],
        "logical_lin_constraints": payload["logical_lin_constraints"],
        "physical_rows": payload["physical_rows"],
        "advice_cols": payload["advice_cols"],
        "fixed_cols": payload["fixed_cols"],
        "instance_cols": payload["instance_cols"],
        "synth_ms": payload["synth_ms"],
        "keygen_vk_ms": payload["keygen_vk_ms"],
        "keygen_pk_ms": payload["keygen_pk_ms"],
        "prove_ms": payload["prove_ms"],
        "verify_ms": payload["verify_ms"],
        "peak_rss_mb": payload["peak_rss_mb"],
        "proof_bytes": payload["proof_bytes"],
        "commit_hash": commit_hash,
        "rust_version": rust_version,
        "backend_commit": backend_commit,
        "machine_profile": f"{platform.system()}-{platform.machine()}",
        "status": payload["status"],
        "notes": payload["notes"],
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(result, indent=2), encoding="utf-8")

    if args.lint_output:
        lint_cmd = [
            "python",
            "scripts/h2dq_lint.py",
            "--result",
            str(out_path),
        ]
        lint_proc = subprocess.run(lint_cmd)
        if lint_proc.returncode != 0:
            raise RuntimeError("h2dq-lint failed")

    print(f"wrote output to: {out_path}")


if __name__ == "__main__":
    main()
