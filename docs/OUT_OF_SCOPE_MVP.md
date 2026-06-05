# CinemaStudio — Out of Scope (MVP)

> Frozen list. Items here do NOT get built until Phase 6+ with their own phase document and gate.
> Any request to add these during MVP phases must be rejected or deferred.

## Explicitly excluded from MVP

### Editing & Production
- Multicam sync and angle switching
- Advanced color grading (wheels, curves, HDR grading)
- VFX compositing (tracking, keying, particles)
- Motion graphics / titles engine
- Advanced audio mixing (multiband, VST plugins)
- Nested timelines
- Compound clips
- Keyframe animation system
- LUT marketplace / management beyond basic apply

### AI & Cloud
- AI generative video (text-to-video)
- AI voice cloning
- Cloud rendering farm
- Real-time collaborative editing
- AI free-form chat as primary interface
- Automatic full film editing without user control

### Platform & Social
- Social network / feed
- In-app community
- Public project sharing
- TikTok-style vertical-first editor UX
- Web-based editor
- Desktop editor (macOS/Windows)

### Business & Ecosystem
- Plugin marketplace
- Third-party plugin SDK
- Asset marketplace (stock footage, music)
- White-label / enterprise multi-tenant
- Advanced analytics dashboard

### Technical
- Microservices architecture
- GraphQL API
- Real-time WebSocket sync
- Blockchain / NFT integration
- Custom scripting language

## Allowed in MVP (reminder)

- Single project workflow: import → timeline → basic edit → export
- 1 video track + 1 audio track
- Cut, trim, split, reorder, fade
- Proxy-based playback
- Local autosave + recovery
- Rule-based AI suggestions (offline)
- MP4 1080p export
- Premium minimal UI

## How to request post-MVP features

1. Create `docs/proposals/FEATURE_NAME.md` with problem, scope, perf impact
2. Assign to a new phase (7+)
3. Define gate criteria
4. Get explicit approval before any code

## Change policy

This file changes only with explicit product owner approval.
Adding scope during active phases is a **process violation** (Rule #7).
