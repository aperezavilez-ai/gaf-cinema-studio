# Cross-module tests

Integration tests live under **`engine/tests/`** (phases 1–13).

| Test file | Scope |
|-----------|--------|
| `integration_phase1.rs` … `phase12.rs` | MVP gates |
| `integration_phase13.rs` | Cloud providers + infrastructure scaffold |

Root `/tests` is reserved for future cross-platform contract tests (JSON schema validation against `shared/`).

Run all:

```bash
cd engine && cargo test
```

Or via GitHub Actions: `.github/workflows/engine-ci.yml`
