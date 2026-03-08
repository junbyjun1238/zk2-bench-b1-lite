#!/usr/bin/env python3
"""h2dq-lint: schema + structural consistency checks for benchmark outputs."""

import argparse
import ast
import json
import re
import sys
from dataclasses import dataclass


@dataclass
class LintResult:
    ok: bool
    errors: list[str]


def parse_const_map(path: str) -> dict[str, int]:
    consts: dict[str, int] = {}
    pattern = re.compile(
        r"pub\s+const\s+([A-Z0-9_]+)\s*:\s*usize\s*=\s*(.*?);",
        re.S,
    )

    with open(path, encoding="utf-8") as f:
        text = f.read()

    for m in pattern.finditer(text):
        name, expr = m.group(1), m.group(2)
        # Drop // comments and collapse whitespace for multiline expressions.
        expr = re.sub(r"//.*", "", expr)
        expr = " ".join(expr.split())
        consts[name] = eval_expr(expr, consts)

    return consts


def eval_expr(expr: str, env: dict[str, int]) -> int:
    node = ast.parse(expr, mode="eval")

    def rec(n):
        if isinstance(n, ast.Expression):
            return rec(n.body)
        if isinstance(n, ast.Constant) and isinstance(n.value, int):
            return int(n.value)
        if isinstance(n, ast.Name):
            if n.id not in env:
                raise ValueError(f"unknown symbol in const expression: {n.id}")
            return int(env[n.id])
        if isinstance(n, ast.BinOp):
            a = rec(n.left)
            b = rec(n.right)
            if isinstance(n.op, ast.Add):
                return a + b
            if isinstance(n.op, ast.Sub):
                return a - b
            if isinstance(n.op, ast.Mult):
                return a * b
            if isinstance(n.op, ast.FloorDiv):
                return a // b
            if isinstance(n.op, ast.Div):
                return a // b
            raise ValueError(f"unsupported operator: {type(n.op).__name__}")
        if isinstance(n, ast.UnaryOp) and isinstance(n.op, ast.USub):
            return -rec(n.operand)
        raise ValueError(f"unsupported expression node: {type(n).__name__}")

    return int(rec(node))


def validate_schema(result: dict, schema: dict) -> list[str]:
    errors: list[str] = []

    props = schema["properties"]
    required = schema["required"]

    missing = [k for k in required if k not in result]
    extra = [k for k in result if k not in props]
    if missing:
        errors.append(f"missing required keys: {missing}")
    if extra:
        errors.append(f"unexpected keys: {extra}")

    if "arm" in result:
        arms = props["arm"]["enum"]
        if result["arm"] not in arms:
            errors.append(f"invalid arm enum: {result['arm']}")

    if "mode" in result:
        modes = props["mode"]["enum"]
        if result["mode"] not in modes:
            errors.append(f"invalid mode enum: {result['mode']}")

    return errors


def expect_equal(errors: list[str], key: str, actual, expected):
    if actual != expected:
        errors.append(f"{key} mismatch: actual={actual}, expected={expected}")


def validate_structural(result: dict, b_consts: dict[str, int], a_consts: dict[str, int]) -> list[str]:
    errors: list[str] = []
    arm = result.get("arm")
    scale = int(result.get("workload_scale", 0))

    if arm == "B_note":
        expect_equal(errors, "logical_lookup_cells", result.get("logical_lookup_cells"), b_consts["LOOKUP_CELLS_PER_REP"] * scale)
        expect_equal(errors, "logical_mul_constraints", result.get("logical_mul_constraints"), b_consts["MUL_CONSTRAINTS_PER_REP"] * scale)
        expect_equal(errors, "logical_lin_constraints", result.get("logical_lin_constraints"), b_consts["LIN_CONSTRAINTS_PER_REP"] * scale)
        expect_equal(errors, "physical_rows", result.get("physical_rows"), b_consts["ROWS_PER_REP"] * scale)
        expect_equal(errors, "advice_cols", result.get("advice_cols"), b_consts["ADVICE_COLS"])
        expect_equal(errors, "fixed_cols", result.get("fixed_cols"), b_consts["FIXED_COLS"])
        expect_equal(errors, "instance_cols", result.get("instance_cols"), b_consts["INSTANCE_COLS"])

    if arm == "A_secure":
        expect_equal(errors, "logical_lookup_cells", result.get("logical_lookup_cells"), a_consts["LOOKUP_CELLS_PER_REP"] * scale)
        expect_equal(errors, "logical_mul_constraints", result.get("logical_mul_constraints"), a_consts["MUL_CONSTRAINTS_PER_REP"] * scale)
        expect_equal(errors, "logical_lin_constraints", result.get("logical_lin_constraints"), a_consts["LIN_CONSTRAINTS_PER_REP"] * scale)
        expect_equal(errors, "physical_rows", result.get("physical_rows"), a_consts["ROWS_PER_REP"] * scale)
        expect_equal(errors, "advice_cols", result.get("advice_cols"), a_consts["ADVICE_COLS"])
        expect_equal(errors, "fixed_cols", result.get("fixed_cols"), a_consts["FIXED_COLS"])
        expect_equal(errors, "instance_cols", result.get("instance_cols"), a_consts["INSTANCE_COLS"])

    return errors


def validate_mode_metrics(result: dict) -> list[str]:
    errors: list[str] = []
    mode = result.get("mode")

    numeric_keys = [
        "synth_ms",
        "keygen_vk_ms",
        "keygen_pk_ms",
        "prove_ms",
        "verify_ms",
        "peak_rss_mb",
    ]
    for k in numeric_keys:
        if float(result.get(k, -1)) < 0:
            errors.append(f"{k} must be non-negative")

    if mode == "fast-structural":
        for k in ["synth_ms", "prove_ms", "verify_ms", "peak_rss_mb", "proof_bytes"]:
            if float(result.get(k, 0)) != 0:
                errors.append(f"{k} must be 0 in fast-structural mode")

    if mode == "full-local":
        for k in ["prove_ms", "verify_ms", "peak_rss_mb"]:
            if float(result.get(k, 0)) <= 0:
                errors.append(f"{k} must be > 0 in full-local mode")

    return errors


def run_lint(result_path: str, schema_path: str, b_mod_path: str, a_mod_path: str) -> LintResult:
    with open(result_path, encoding="utf-8") as f:
        result = json.load(f)
    with open(schema_path, encoding="utf-8") as f:
        schema = json.load(f)

    b_consts = parse_const_map(b_mod_path)
    a_consts = parse_const_map(a_mod_path)

    errors = []
    errors.extend(validate_schema(result, schema))
    errors.extend(validate_structural(result, b_consts, a_consts))
    errors.extend(validate_mode_metrics(result))

    return LintResult(ok=(len(errors) == 0), errors=errors)


def main() -> int:
    parser = argparse.ArgumentParser(description="Lint benchmark JSON output")
    parser.add_argument("--result", required=True)
    parser.add_argument("--schema", default="docs/results_schema.json")
    parser.add_argument("--baseline-b", default="src/baseline_b/mod.rs")
    parser.add_argument("--baseline-a", default="src/baseline_a/mod.rs")
    args = parser.parse_args()

    res = run_lint(args.result, args.schema, args.baseline_b, args.baseline_a)
    if res.ok:
        print(f"h2dq-lint: ok ({args.result})")
        return 0

    print(f"h2dq-lint: fail ({args.result})")
    for i, e in enumerate(res.errors, 1):
        print(f"{i}. {e}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
