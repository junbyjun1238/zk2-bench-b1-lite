#!/usr/bin/env python3
"""h2dq-cert: wrapper command for deterministic certificate checks."""

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone


def run_checker(certificate: str, manuscript: str, backend_instance: str) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "scripts/check_public_certificate.py",
        "--certificate",
        certificate,
        "--manuscript",
        manuscript,
    ]
    if backend_instance:
        cmd.extend(["--backend-instance", backend_instance])

    proc = subprocess.run(cmd, capture_output=True, text=True)
    return proc.returncode, proc.stdout.strip(), proc.stderr.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run certificate/backend binding checks")
    parser.add_argument("--certificate", default="certificates/public_certificate.json")
    parser.add_argument("--manuscript", default="core_papers/wrapper_note_option2.tex")
    parser.add_argument("--backend-instance", default="certificates/h2dq_backend_instance.json")
    parser.add_argument("--json-out", default="")
    args = parser.parse_args()

    code, out, err = run_checker(args.certificate, args.manuscript, args.backend_instance)
    status = "ok" if code == 0 else "fail"

    report = {
        "tool": "h2dq-cert",
        "status": status,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "certificate": args.certificate,
        "manuscript": args.manuscript,
        "backend_instance": args.backend_instance,
        "stdout": out,
        "stderr": err,
    }

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as f:
            json.dump(report, f, indent=2)
            f.write("\n")

    print(f"h2dq-cert: {status}")
    if out:
        print(out)
    if err:
        print(err, file=sys.stderr)

    return code


if __name__ == "__main__":
    raise SystemExit(main())
