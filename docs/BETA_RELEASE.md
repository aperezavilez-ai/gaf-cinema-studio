# CinemaStudio — Beta Release Guide

> Phase 12: TestFlight (iOS) + Play Internal Testing (Android) + MVP ship gates.

## MVP version

**1.0.0** — first public beta.

## Automated gates

| Gate | Target | Verify |
|------|--------|--------|
| 12.1 | Beta projects | 10 completions in `beta_registry.json` |
| 12.2 | Crash rate | < 1% sessions (telemetry opt-in) |
| 12.3 | Build uploads | TestFlight + Play Internal |
| 12.4 | MVP readiness | `mvp_readiness_report().ready_to_ship` |

```powershell
.\scripts\beta_gate_check.ps1
cd engine && cargo test --test integration_phase12
```

---

## iOS — TestFlight

### Prerequisites

- Apple Developer account ($99/year)
- Mac with Xcode 15+
- `DEVELOPMENT_TEAM` in `ios/project.yml`

### Steps

1. Build Rust engine (optional but recommended):
   ```bash
   ./scripts/build_ios.sh
   ./scripts/generate_bindings.sh
   ```

2. Generate Xcode project:
   ```bash
   cd ios && xcodegen generate
   ```

3. Archive + upload:
   ```bash
   ./scripts/release_ios.sh
   # or
   cd fastlane && fastlane beta
   ```

4. App Store Connect → TestFlight → add internal testers (up to 100)

---

## Android — Play Internal Testing

### Prerequisites

- Google Play Developer account ($25 one-time)
- Release keystore (create once, store securely)

### Steps

1. Create keystore (once):
   ```bash
   keytool -genkey -v -keystore cinemastudio-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias cinemastudio
   ```

2. Set env vars (or `android/local.properties` — never commit):
   ```properties
   CINEMASTUDIO_KEYSTORE=/path/to/cinemastudio-release.jks
   CINEMASTUDIO_KEYSTORE_PASSWORD=***
   CINEMASTUDIO_KEY_ALIAS=cinemastudio
   CINEMASTUDIO_KEY_PASSWORD=***
   ```

3. Build release bundle:
   ```bash
   ./scripts/release_android.sh
   ```

4. Play Console → Testing → Internal testing → Create release → upload AAB

---

## Beta cohort workflow

1. Tester installs via TestFlight / Play Internal link
2. Creates project → imports media → edits → exports
3. Taps **Mark project complete** in Settings → Beta Program
4. Engine records completion in `beta_registry.json`
5. When 10 completions + crash rate OK → `ready_to_ship: true`

---

## Store metadata

Templates in `docs/store/` — fill before public release.

---

## Post-MVP

See `docs/OUT_OF_SCOPE_MVP.md` for features deferred after 1.0.
