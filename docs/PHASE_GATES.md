# CinemaStudio — Phase Gates

Each phase must pass **all** criteria before advancing.
Human approval required: respond with `CONTINUE PHASE` in chat.

---

## Phase 0 — Foundation

**Goal:** Architecture frozen, repo structured, schemas defined, engine skeleton compiles.

### Deliverables
- [x] Monorepo structure
- [x] `docs/ARCHITECTURE.md`
- [x] `docs/PROJECT_STATE.schema.json`
- [x] `docs/PHASE_GATES.md`
- [x] `docs/OUT_OF_SCOPE_MVP.md`
- [x] `docs/EVENT_BUS.md`
- [x] `.cursor/rules/cinemastudio.md`
- [x] Rust engine crate with `ProjectState` types
- [x] Basic validation + unit tests

### Gate Criteria
| # | Criterion | Pass |
|---|-----------|------|
| 0.1 | Repo structure matches architecture doc | ☐ |
| 0.2 | Project State schema validates example JSON | ☐ |
| 0.3 | `cargo test` passes in `/engine` | ☐ |
| 0.4 | No unapproved dependencies | ☐ |
| 0.5 | Cursor rules file present | ☐ |
| 0.6 | Architecture diagram + module contracts documented | ☐ |

**Status:** Awaiting validation → then `CONTINUE PHASE` for Phase 1

---

## Phase 1 — Project State + Storage + Recovery

**Goal:** Create, open, save, recover projects locally. Media vault basics.

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 1.1 | Kill app mid-save → recovery succeeds | 100% |
| 1.2 | Open project with 50 media entries | < 3s |
| 1.3 | Autosave cycle | < 500ms background |
| 1.4 | 100 open/save/close cycles | 0 data loss |
| 1.5 | Automated persistence tests | All pass |
| 1.6 | Proxy generation | Non-blocking UI |

---

## Phase 2 — Video Engine MVP

**Goal:** Timeline + fluid playback with proxies.

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 2.1 | 1080p proxy playback (mid device) | Fluid |
| 2.2 | Scrub freeze | < 100ms |
| 2.3 | 30 min session RAM | Stable |
| 2.4 | 1 hour session | 0 crashes |
| 2.5 | 20 clip timeline sync | Correct |
| 2.6 | Frame drop rate | < 5% |

---

## Phase 3 — Minimal Cinematic Editing

**Goal:** Usable edit workflow: import → edit → export.

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 3.1 | Complete workflow unassisted | Pass |
| 3.2 | Export blocks UI | 0ms |
| 3.3 | Undo/redo consistency | 100% |
| 3.4 | Main flow depth | ≤ 3 levels |
| 3.5 | Export quality | 1080p H.264 |

---

## Phase 4 — AI Orchestrator v1

**Goal:** Context-aware guidance, not chatbot.

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 4.1 | Offline functionality | 100% |
| 4.2 | Suggestions from real state | No generic text |
| 4.3 | AI actions undoable | 100% |
| 4.4 | No free-form chat dependency | Required |

---

## Phase 5 — Device Adaptive + Hardening

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 5.1 | Low/mid/high tier policies | All work |
| 5.2 | 2h profiling | 0 leaks |
| 5.3 | Battery within threshold | Measured |
| 5.4 | Thermal throttling response | Auto-adapt |

---

## Phase 6 — Beta + Optional Cloud

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 6.1 | Core without account | Works |
| 6.2 | Cloud backup/restore | Tested |
| 6.3 | Beta user projects | 10 complete |
| 6.4 | Crash rate | < 1% sessions |

---

## Phase 7 — Integration Scaffold

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 7.1 | Decoder trait + stub | Works |
| 7.2 | Render pipeline + export refactor | Works |
| 7.3 | UniFFI full API surface | Exported |
| 7.4 | Native bridge hooks | Registered |

---

## Phase 8 — App Shell + CI

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 8.1 | iOS XcodeGen project | Generatable |
| 8.2 | Android Gradle project | Openable |
| 8.3 | GitHub Actions CI | Tests pass |
| 8.4 | Editor UI both platforms | Scaffold |

---

## Phase 9 — Native Preview Pipeline

### Gate Criteria
| # | Criterion | Target |
|---|-----------|--------|
| 9.1 | iOS AVFoundation preview | Works at playhead |
| 9.2 | Android frame preview | Works at playhead |
| 9.3 | Mobile build scripts | iOS + Android scaffolds |
| 9.4 | Document picker | iOS open .csproj |

---

## Validation Protocol (every phase)

1. Run automated tests
2. Run manual checklist
3. Document results in `docs/phases/phase-N-report.md`
4. Present: structure, files, risks, improvements
5. **STOP** — wait for `CONTINUE PHASE`
