# SonicDeck

## Current State (Updated: 2025-12-29)

**Shipped:** v0.8.1-alpha (2025-12)
**Status:** Alpha / Internal Testing
**Users:** Solo developer + 2-3 friends as testers (planned)
**Feedback:** No external testing yet - prototype phase complete, now polishing

**Codebase:**
- ~15,000 lines (TypeScript + Rust)
- Frontend: React 19 + Vite 7 + TailwindCSS 4
- Backend: Tauri v2 + Rust (cpal + symphonia)
- CI/CD: GitHub Actions, Codecov

**Known Issues:**
- Dual Output + Discord: Discord's Krisp/Noise Gate filters constant tones (airhorn) but passes variable sounds (pew) - not a SonicDeck bug, needs user guidance
- Test Coverage: Foundation laid, but needs expansion
- UI Polish: Works, but not "finished" - subject to change

## Vision

SonicDeck is a desktop soundboard app for creators and streamers. Low-latency audio playback, dual-output routing (speakers + virtual device for Discord/OBS), intuitive waveform visualization, and global hotkeys.

The project started from personal need, desire to learn (Tauri/Rust), and wanting to build something portfolio-worthy. Existing soundboard solutions were either too complex, too expensive, or technically unsatisfying.

## v1.0 Goals

**Vision:** Stable, local soundboard app that I can use daily - and that others can install and understand without my help.

**Motivation:**
- Prototype phase complete, now polish and stability
- Enable deep testing with friends
- Build solid foundation before feature expansion

**Scope (v1.0):**
- Audio core fully polished (Volume Engine V2, LUFS Normalization)
- Auto-updater for seamless updates
- Import/Export of sound library (JSON/ZIP)
- VB-Cable integration for reliable Discord routing
- Expand test coverage (Rust + Frontend)
- UI/UX polish where needed

**Success Criteria:**
- [ ] I use SonicDeck daily without frustration
- [ ] 2-3 friends can install and use it without my help
- [ ] Dual Output + Discord works reliably
- [ ] Auto-updater works
- [ ] Test coverage provides confidence when making changes

**Not Building (v1.0):**
- Cloud features (accounts, sync, marketplace)
- OBS integration
- Audio effects / voice changer
- Linux/Mac support
- Mobile remote

## Constraints

- **Platform:** Windows-only for v1.0
- **Tech Stack:** Tauri v2 + React + Rust - no major changes
- **Pace:** Hobby project, flexible, no hard deadlines

## Current Strengths

What already works well:
- Audio core playback (recently improved)
- Caching (feels fast)
- Waveform in trim editor
- Drag & drop import
- Global hotkeys

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| VB-Cable Integration | Bundle with attribution | Free distribution allowed with VB-Audio credit, link, and "donationware" mention. Auto-detect + silent install. |
| Discord Audio Issues | Document workaround | Discord's Krisp/Noise Gate is the cause - recommend users disable noise suppression. LUFS normalization may help. |

## Open Questions

- [ ] Which UI areas need the most polish?
- [ ] When is "enough" tested for v1.0 release?
- [ ] TeamSpeak and in-game voice chat behavior? (untested)

---

<details>
<summary>Original Vision (v0.1 - Archived)</summary>

## Vision

High-performance desktop soundboard for creators and streamers with:
- Dual-audio routing (Primary + Secondary Output)
- Sound library management with categories
- Waveform visualization and audio trimming
- Global hotkeys for quick access

## Problem

Existing soundboard solutions are either:
- Too complex (Voicemeeter setup nightmare)
- Too expensive (subscription models)
- Technically unsatisfying (latency, no dual-output)
- Not open-source

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Framework | Tauri v2 | Rust performance, small bundle size, modern API |
| Audio | cpal + symphonia | Native Rust, no FFmpeg dependency |
| Frontend | React + Vite | Fast development, good ecosystem |
| Styling | TailwindCSS | Utility-first, rapid prototyping |

</details>

---
*Initialized: 2025-12-29*
