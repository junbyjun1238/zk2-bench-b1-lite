#!/usr/bin/env python3
"""Deterministic checker for the public synthesis certificate.

Usage:
  python scripts/check_public_certificate.py \
      --certificate certificates/public_certificate.json \
      --manuscript wrapper_note_option2.tex

Optional:
  python scripts/check_public_certificate.py \
      --certificate certificates/public_certificate.json \
      --manuscript wrapper_note_option2.tex \
      --update-payload-digest
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Dict, Iterable, List, Tuple


EXPECTED_P = 2**31 - 1
EXPECTED_R = int(
    "21888242871839275222246405745257275088548364400416034343698204186575808495617"
)

EXPECTED_FAMILIES = {
    "inv": {
        "class_bits": 66,
        "u_left_dec": 158456324880954722595264004098,
        "u_right_dec": 2147483646,
    },
    "EF": {
        "class_bits": 31,
        "u_left_dec": 4611686009837453316,
        "u_right_dec": 2147483646,
    },
    "EE": {
        "class_bits": 66,
        "u_left_dec": 158456324585806817830375522176,
        "u_right_dec": 2147483646,
    },
    "hor": {
        "class_bits": 66,
        "u_left_dec": 158456324585806817830375522176,
        "u_right_dec": 158456324585806817830375522176,
    },
    "car": {
        "class_bits": 31,
        "u_left_dec": 4294967299,
        "u_right_dec": 4294967299,
    },
}


def parse_int(value) -> int:
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        return int(value)
    raise TypeError(f"Expected int-like value, got: {type(value)}")


def canonical_json_bytes(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256_hex_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file_hex(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def require(condition: bool, msg: str, errors: List[str]) -> None:
    if not condition:
        errors.append(msg)


def eval_expr_bound(expr: Dict, intervals: Dict[str, int]) -> int:
    total = 0
    for term in expr.get("linear", []):
        coeff = parse_int(term["coeff"])
        var = term["var"]
        total += coeff * intervals[var]
    for term in expr.get("quadratic", []):
        coeff = parse_int(term["coeff"])
        u = term["u"]
        v = term["v"]
        total += coeff * intervals[u] * intervals[v]
    return total


def validate_metadata(
    cert: Dict, manuscript_path: Path | None, errors: List[str]
) -> None:
    metadata = cert.get("metadata")
    require(isinstance(metadata, dict), "metadata must be an object", errors)
    if not isinstance(metadata, dict):
        return
    for k in ["manuscript_file", "manuscript_sha256", "schema_version"]:
        require(k in metadata, f"metadata.{k} missing", errors)
    if manuscript_path is not None and manuscript_path.exists():
        actual = sha256_file_hex(manuscript_path)
        declared = metadata.get("manuscript_sha256")
        require(
            declared == actual,
            f"manuscript sha mismatch: declared={declared}, actual={actual}",
            errors,
        )


def validate_constants(payload: Dict, errors: List[str]) -> Tuple[int, int]:
    constants = payload.get("constants")
    require(isinstance(constants, dict), "payload.constants must be an object", errors)
    if not isinstance(constants, dict):
        return 0, 0
    p = parse_int(constants.get("p"))
    r = parse_int(constants.get("r"))
    require(p == EXPECTED_P, f"p mismatch: {p}", errors)
    require(r == EXPECTED_R, "r mismatch", errors)
    return p, r


def validate_families_and_templates(
    payload: Dict, p: int, r: int, errors: List[str]
) -> Dict[Tuple[str, str], Dict]:
    families = payload.get("families")
    templates = payload.get("template_library")
    require(isinstance(families, list), "payload.families must be an array", errors)
    require(
        isinstance(templates, list), "payload.template_library must be an array", errors
    )
    if not isinstance(families, list) or not isinstance(templates, list):
        return {}

    fam_by_id = {}
    for fam in families:
        fid = fam.get("id")
        require(fid not in fam_by_id, f"duplicate family id: {fid}", errors)
        fam_by_id[fid] = fam

    require(
        set(fam_by_id.keys()) == set(EXPECTED_FAMILIES.keys()),
        f"family ids mismatch: got={sorted(fam_by_id.keys())}",
        errors,
    )

    template_by_key: Dict[Tuple[str, str], Dict] = {}
    templates_by_family: Dict[str, List[Dict]] = {k: [] for k in EXPECTED_FAMILIES}

    for t in templates:
        fid = t.get("family_id")
        tid = t.get("template_id")
        key = (fid, tid)
        require(key not in template_by_key, f"duplicate template key: {key}", errors)
        template_by_key[key] = t
        if fid in templates_by_family:
            templates_by_family[fid].append(t)

        intervals_raw = t.get("intervals", {})
        require(
            isinstance(intervals_raw, dict),
            f"template {key} intervals must be object",
            errors,
        )
        intervals: Dict[str, int] = {}
        if isinstance(intervals_raw, dict):
            for var, vmax in intervals_raw.items():
                vmax_i = parse_int(vmax)
                require(vmax_i >= 0, f"template {key}: interval {var} negative", errors)
                intervals[var] = vmax_i

        left = t.get("left", {})
        right = t.get("right", {})
        for side_name, expr in [("left", left), ("right", right)]:
            require(isinstance(expr, dict), f"template {key}: {side_name} must be object", errors)
            if not isinstance(expr, dict):
                continue
            for term in expr.get("linear", []):
                coeff = parse_int(term["coeff"])
                require(coeff >= 0, f"template {key}: negative linear coeff", errors)
                require(term["var"] in intervals, f"template {key}: unknown var in linear", errors)
            for term in expr.get("quadratic", []):
                coeff = parse_int(term["coeff"])
                require(coeff >= 0, f"template {key}: negative quadratic coeff", errors)
                require(term["u"] in intervals, f"template {key}: unknown u var", errors)
                require(term["v"] in intervals, f"template {key}: unknown v var", errors)

    for fid, expected in EXPECTED_FAMILIES.items():
        require(fid in fam_by_id, f"missing family {fid}", errors)
        if fid not in fam_by_id:
            continue
        fam = fam_by_id[fid]
        class_bits = parse_int(fam.get("class_bits"))
        u_left_decl = parse_int(fam.get("u_left_dec"))
        u_right_decl = parse_int(fam.get("u_right_dec"))
        require(
            class_bits == expected["class_bits"],
            f"family {fid}: class_bits mismatch",
            errors,
        )
        require(
            u_left_decl == expected["u_left_dec"],
            f"family {fid}: u_left_dec mismatch",
            errors,
        )
        require(
            u_right_decl == expected["u_right_dec"],
            f"family {fid}: u_right_dec mismatch",
            errors,
        )

        tpls = templates_by_family.get(fid, [])
        require(len(tpls) > 0, f"family {fid}: no templates", errors)
        computed_lefts = []
        computed_rights = []
        for t in tpls:
            intervals = {k: parse_int(v) for k, v in t["intervals"].items()}
            lbound = eval_expr_bound(t["left"], intervals)
            rbound = eval_expr_bound(t["right"], intervals)
            computed_lefts.append(lbound)
            computed_rights.append(rbound)
            require(
                parse_int(t.get("class_bits")) == class_bits,
                f"template {(fid, t.get('template_id'))}: class mismatch with family",
                errors,
            )
        if computed_lefts:
            require(
                max(computed_lefts) == u_left_decl,
                f"family {fid}: compiled left bound != declared",
                errors,
            )
        if computed_rights:
            require(
                max(computed_rights) == u_right_decl,
                f"family {fid}: compiled right bound != declared",
                errors,
            )

        b = 2**class_bits
        require(
            u_left_decl < p * b + p,
            f"family {fid}: headroom check U_left < p*B+p failed",
            errors,
        )
        require(
            u_right_decl + p * (b - 1) < r,
            f"family {fid}: RHS no-wrap check failed",
            errors,
        )

    return template_by_key


def validate_coverage(payload: Dict, errors: List[str]) -> Dict[str, set]:
    coverage = payload.get("coverage")
    require(isinstance(coverage, dict), "payload.coverage must be an object", errors)
    if not isinstance(coverage, dict):
        return {}

    q_groups = coverage.get("q_groups")
    require(isinstance(q_groups, dict), "coverage.q_groups must be object", errors)
    if not isinstance(q_groups, dict):
        return {}

    expected_group_keys = {"inv", "ef", "ee", "hor", "car"}
    require(
        set(q_groups.keys()) == expected_group_keys,
        "coverage.q_groups keys mismatch",
        errors,
    )

    sets: Dict[str, set] = {}
    for k, vals in q_groups.items():
        require(isinstance(vals, list), f"coverage.q_groups.{k} must be list", errors)
        vals_list = vals if isinstance(vals, list) else []
        vals_set = set(vals_list)
        require(
            len(vals_list) == len(vals_set),
            f"coverage.q_groups.{k} contains duplicates",
            errors,
        )
        sets[f"q_{k}"] = vals_set

    q31 = set(coverage.get("q31", []))
    q66 = set(coverage.get("q66", []))
    qall = set(coverage.get("q_all", []))
    require(
        sets["q_ef"].union(sets["q_car"]) == q31,
        "coverage.q31 must equal q_ef U q_car",
        errors,
    )
    require(
        sets["q_inv"].union(sets["q_ee"]).union(sets["q_hor"]) == q66,
        "coverage.q66 must equal q_inv U q_ee U q_hor",
        errors,
    )
    require(
        q31.isdisjoint(q66),
        "coverage.q31 and coverage.q66 must be disjoint",
        errors,
    )
    require(
        q31.union(q66) == qall,
        "coverage.q_all must equal q31 U q66",
        errors,
    )

    nonq = coverage.get("nonquotient")
    require(
        isinstance(nonq, dict), "coverage.nonquotient must be object", errors
    )
    if isinstance(nonq, dict):
        p_set = set(nonq.get("p", []))
        bit_set = set(nonq.get("bit", []))
        u8_set = set(nonq.get("u8", []))
        require(
            p_set.isdisjoint(bit_set) and p_set.isdisjoint(u8_set) and bit_set.isdisjoint(u8_set),
            "nonquotient class sets must be pairwise disjoint",
            errors,
        )
        sets["p"] = p_set
        sets["bit"] = bit_set
        sets["u8"] = u8_set

    sets["q31"] = q31
    sets["q66"] = q66
    sets["q_all"] = qall
    return sets


def validate_wiring(payload: Dict, coverage_sets: Dict[str, set], errors: List[str]) -> None:
    wiring = payload.get("wiring")
    require(isinstance(wiring, dict), "payload.wiring must be object", errors)
    if not isinstance(wiring, dict):
        return

    q_tags = wiring.get("quotient_slot_tags", [])
    q_wiring = wiring.get("quotient_wiring", [])
    require(isinstance(q_tags, list), "wiring.quotient_slot_tags must be list", errors)
    require(isinstance(q_wiring, list), "wiring.quotient_wiring must be list", errors)
    if not isinstance(q_tags, list) or not isinstance(q_wiring, list):
        return

    q_tag_map = {}
    for e in q_tags:
        k = (e["selector"], e["slot"])
        require(k not in q_tag_map, f"duplicate quotient slot tag: {k}", errors)
        q_tag_map[k] = parse_int(e["class_bits"])

    q_local_seen = set()
    q_image = set()
    for e in q_wiring:
        k = (e["selector"], e["slot"])
        col = e["column"]
        require(k in q_tag_map, f"quotient wiring missing class tag for slot {k}", errors)
        require(k not in q_local_seen, f"quotient functionality violated at {k}", errors)
        q_local_seen.add(k)
        require(col in coverage_sets["q_all"], f"undeclared quotient symbol: {col}", errors)
        q_image.add(col)
        if k in q_tag_map:
            cbits = q_tag_map[k]
            if cbits == 31:
                require(col in coverage_sets["q31"], f"class mismatch: {k} -> {col} not in q31", errors)
            elif cbits == 66:
                require(col in coverage_sets["q66"], f"class mismatch: {k} -> {col} not in q66", errors)
            else:
                errors.append(f"invalid class bits for {k}: {cbits}")

    require(
        q_image == coverage_sets["q_all"],
        "quotient exact coverage failed: Im(omega_Q) != Q_all",
        errors,
    )

    n_tags = wiring.get("nonquotient_slot_tags", [])
    n_wiring = wiring.get("nonquotient_wiring", [])
    require(
        isinstance(n_tags, list), "wiring.nonquotient_slot_tags must be list", errors
    )
    require(
        isinstance(n_wiring, list), "wiring.nonquotient_wiring must be list", errors
    )
    if not isinstance(n_tags, list) or not isinstance(n_wiring, list):
        return

    n_tag_map = {}
    for e in n_tags:
        k = (e["selector"], e["slot"])
        require(k not in n_tag_map, f"duplicate nonquotient slot tag: {k}", errors)
        n_tag_map[k] = e["tag"]

    n_local_seen = set()
    image_by_tag = {"p": set(), "bit": set(), "u8": set()}
    for e in n_wiring:
        k = (e["selector"], e["slot"])
        col = e["column"]
        require(k in n_tag_map, f"nonquotient wiring missing tag for slot {k}", errors)
        require(k not in n_local_seen, f"nonquotient functionality violated at {k}", errors)
        n_local_seen.add(k)
        tag = n_tag_map[k]
        require(tag in image_by_tag, f"unknown nonquotient tag: {tag}", errors)
        if tag in image_by_tag:
            image_by_tag[tag].add(col)
            require(
                col in coverage_sets[tag],
                f"nonquotient tag mismatch: {k} tagged {tag} but maps to {col}",
                errors,
            )

    for tag in ["p", "bit", "u8"]:
        require(
            image_by_tag[tag] == coverage_sets[tag],
            f"nonquotient exact coverage failed for tag {tag}",
            errors,
        )


def validate_relation_and_activity(
    payload: Dict, template_by_key: Dict[Tuple[str, str], Dict], errors: List[str]
) -> None:
    relation = payload.get("relation_realization", [])
    require(
        isinstance(relation, list), "payload.relation_realization must be list", errors
    )
    if not isinstance(relation, list):
        return

    rel_ids = set()
    for e in relation:
        rid = e["relation_id"]
        require(rid not in rel_ids, f"duplicate relation_id: {rid}", errors)
        rel_ids.add(rid)
        key = (e["template_family"], e["template_id"])
        require(
            key in template_by_key,
            f"relation {rid} references unknown template {key}",
            errors,
        )

    activity = payload.get("row_activity")
    require(
        isinstance(activity, dict), "payload.row_activity must be object", errors
    )
    if not isinstance(activity, dict):
        return
    omega_rows = activity.get("omega_rows", [])
    active_pairs = activity.get("active_pairs", [])
    require(isinstance(omega_rows, list), "row_activity.omega_rows must be list", errors)
    require(isinstance(active_pairs, list), "row_activity.active_pairs must be list", errors)
    if not isinstance(omega_rows, list) or not isinstance(active_pairs, list):
        return
    omega_set = set(omega_rows)
    seen_rel = set()
    for pair in active_pairs:
        rid = pair["relation_id"]
        row = pair["row"]
        require(rid in rel_ids, f"row_activity references unknown relation_id {rid}", errors)
        require(row in omega_set, f"row_activity row {row} not in omega_rows", errors)
        seen_rel.add(rid)
    require(
        seen_rel == rel_ids,
        "row_activity must include at least one active pair for each relation_id",
        errors,
    )
    require(
        bool(activity.get("inactive_zero_extension")),
        "row_activity.inactive_zero_extension must be true",
        errors,
    )


def validate_digest_binding(
    cert: Dict, manuscript_path: Path | None, errors: List[str]
) -> None:
    digest = cert.get("digest_binding")
    require(isinstance(digest, dict), "digest_binding must be object", errors)
    if not isinstance(digest, dict):
        return
    payload = cert.get("payload")
    require(isinstance(payload, dict), "payload must be object", errors)
    if not isinstance(payload, dict):
        return
    expected_payload_hash = digest.get("payload_sha256")
    actual_payload_hash = sha256_hex_bytes(canonical_json_bytes(payload))
    require(
        expected_payload_hash == actual_payload_hash,
        f"payload_sha256 mismatch: declared={expected_payload_hash}, actual={actual_payload_hash}",
        errors,
    )
    if manuscript_path is not None and manuscript_path.exists():
        declared_m = digest.get("manuscript_sha256")
        actual_m = sha256_file_hex(manuscript_path)
        require(
            declared_m == actual_m,
            f"digest_binding manuscript sha mismatch: declared={declared_m}, actual={actual_m}",
            errors,
        )


def validate_backend_instance(
    backend_instance_path: Path,
    cert: Dict,
    cert_path: Path,
    manuscript_path: Path | None,
    errors: List[str],
) -> None:
    with backend_instance_path.open("r", encoding="utf-8-sig") as f:
        inst = json.load(f)

    require(isinstance(inst, dict), "backend instance must be an object", errors)
    if not isinstance(inst, dict):
        return

    require(
        isinstance(inst.get("backend_instance_id"), str)
        and len(inst["backend_instance_id"]) > 0,
        "backend_instance_id missing",
        errors,
    )

    binding = inst.get("instance_binding")
    require(isinstance(binding, dict), "instance_binding must be an object", errors)
    if not isinstance(binding, dict):
        return

    for k in [
        "certificate_path",
        "certificate_payload_sha256",
        "checker_path",
        "checker_sha256",
        "manuscript_path",
        "manuscript_sha256",
    ]:
        require(k in binding, f"instance_binding.{k} missing", errors)

    cert_payload_hash = (
        cert.get("digest_binding", {}).get("payload_sha256")
        if isinstance(cert.get("digest_binding"), dict)
        else None
    )
    require(
        binding.get("certificate_payload_sha256") == cert_payload_hash,
        "backend instance certificate_payload_sha256 mismatch",
        errors,
    )

    declared_cert_path = str(binding.get("certificate_path", ""))
    require(
        declared_cert_path.replace("\\", "/").endswith(str(cert_path).replace("\\", "/"))
        or Path(declared_cert_path).name == cert_path.name,
        "backend instance certificate_path does not point to the checked certificate",
        errors,
    )

    checker_path = Path(binding.get("checker_path", ""))
    if checker_path.exists():
        checker_hash_actual = sha256_file_hex(checker_path)
        require(
            binding.get("checker_sha256") == checker_hash_actual,
            "backend instance checker_sha256 mismatch",
            errors,
        )
    else:
        errors.append(f"backend instance checker_path not found: {checker_path}")

    if manuscript_path is not None and manuscript_path.exists():
        mhash = sha256_file_hex(manuscript_path)
        require(
            binding.get("manuscript_sha256") == mhash,
            "backend instance manuscript_sha256 mismatch",
            errors,
        )
        declared_manuscript_path = str(binding.get("manuscript_path", ""))
        require(
            declared_manuscript_path.replace("\\", "/").endswith(
                str(manuscript_path).replace("\\", "/")
            )
            or Path(declared_manuscript_path).name == manuscript_path.name,
            "backend instance manuscript_path does not match checked manuscript",
            errors,
        )

    fs = inst.get("fiat_shamir_schedule")
    require(
        isinstance(fs, dict), "fiat_shamir_schedule must be an object", errors
    )
    if isinstance(fs, dict):
        order = fs.get("challenge_order")
        require(isinstance(order, list), "challenge_order must be an array", errors)
        if isinstance(order, list):
            names = [x.get("name") for x in order if isinstance(x, dict)]
            require(
                names == ["lambda_vec", "zeta"],
                "challenge_order must be exactly [lambda_vec, zeta]",
                errors,
            )
            if len(order) >= 2 and isinstance(order[1], dict):
                require(
                    order[1].get("constraint") == "zeta_not_in_Omega",
                    "zeta constraint must be zeta_not_in_Omega",
                    errors,
                )


def update_payload_digest(cert_path: Path, manuscript_path: Path | None) -> None:
    with cert_path.open("r", encoding="utf-8") as f:
        cert = json.load(f)
    payload = cert["payload"]
    cert.setdefault("digest_binding", {})
    cert["digest_binding"]["payload_sha256"] = sha256_hex_bytes(
        canonical_json_bytes(payload)
    )
    if manuscript_path is not None and manuscript_path.exists():
        mhash = sha256_file_hex(manuscript_path)
        cert.setdefault("metadata", {})
        cert["metadata"]["manuscript_sha256"] = mhash
        cert["digest_binding"]["manuscript_sha256"] = mhash
    with cert_path.open("w", encoding="utf-8", newline="\n") as f:
        json.dump(cert, f, indent=2, ensure_ascii=True)
        f.write("\n")
    print(f"Updated digest fields in {cert_path}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--certificate",
        default="certificates/public_certificate.json",
        help="Path to certificate JSON",
    )
    parser.add_argument(
        "--manuscript",
        default="wrapper_note_option2.tex",
        help="Path to manuscript file for digest binding checks",
    )
    parser.add_argument(
        "--update-payload-digest",
        action="store_true",
        help="Rewrite payload/manuscript digest fields in certificate",
    )
    parser.add_argument(
        "--backend-instance",
        default="",
        help="Optional backend instance manifest JSON to validate",
    )
    args = parser.parse_args()

    cert_path = Path(args.certificate)
    manuscript_path = Path(args.manuscript) if args.manuscript else None

    if args.update_payload_digest:
        update_payload_digest(cert_path, manuscript_path)
        return 0

    with cert_path.open("r", encoding="utf-8") as f:
        cert = json.load(f)

    errors: List[str] = []
    validate_metadata(cert, manuscript_path, errors)

    payload = cert.get("payload")
    require(isinstance(payload, dict), "payload must be an object", errors)
    if not isinstance(payload, dict):
        payload = {}

    p, r = validate_constants(payload, errors)
    template_by_key = validate_families_and_templates(payload, p, r, errors)
    coverage_sets = validate_coverage(payload, errors)
    validate_wiring(payload, coverage_sets, errors)
    validate_relation_and_activity(payload, template_by_key, errors)
    validate_digest_binding(cert, manuscript_path, errors)
    if args.backend_instance:
        validate_backend_instance(
            Path(args.backend_instance), cert, cert_path, manuscript_path, errors
        )

    if errors:
        print("Certificate validation FAILED:")
        for i, err in enumerate(errors, start=1):
            print(f"{i}. {err}")
        return 1

    print("Certificate validation OK.")
    print(
        "Checked: constants, deterministic family bounds, headroom/no-wrap, "
        "coverage partitions, selector wiring, relation/activity maps, and digest binding."
    )
    if args.backend_instance:
        print(f"Checked backend instance manifest: {args.backend_instance}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
