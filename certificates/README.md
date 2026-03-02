# Public Certificate Artifact

This folder contains the machine-readable synthesis certificate used in
`wrapper_note_option2.tex`.

- Certificate: `public_certificate.json`
- Checker: `../scripts/check_public_certificate.py`

## Verify

```bash
python scripts/check_public_certificate.py \
  --certificate certificates/public_certificate.json \
  --manuscript wrapper_note_option2.tex
```

Verify with pinned backend instance manifest:

```bash
python scripts/check_public_certificate.py \
  --certificate certificates/public_certificate.json \
  --manuscript wrapper_note_option2.tex \
  --backend-instance certificates/h2dq_backend_instance.json
```

## Digest Maintenance

If the manuscript changes, refresh digest bindings:

```bash
python scripts/check_public_certificate.py \
  --certificate certificates/public_certificate.json \
  --manuscript wrapper_note_option2.tex \
  --update-payload-digest
```
