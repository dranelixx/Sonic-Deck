# Roadmap: SonicDeck v1.0

## Overview

SonicDeck v1.0 focuses on stability, testability, and a smooth experience for testers. We start with a solid test foundation, then add VB-Cable integration for seamless dual-output, polish the audio core with LUFS normalization, enable auto-updates for testers, add import/export for sound library sharing, and finish with UI/UX polish.

## Domain Expertise

None

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Test Coverage** - Expand Rust and Frontend tests for a solid foundation
- [ ] **Phase 2: VB-Cable Integration** - Auto-detection and silent install for Discord routing
- [ ] **Phase 3: Audio Core Polish** - Volume Engine V2 and LUFS normalization
- [ ] **Phase 4: Auto-Updater** - Seamless updates via Tauri updater
- [ ] **Phase 5: Import/Export** - Sound library as JSON/ZIP with atomic persistence
- [ ] **Phase 6: UI/UX Polish** - Final improvements for v1.0 release

## Phase Details

### Phase 1: Test Coverage
**Goal**: Expand test coverage to provide confidence for subsequent changes
**Depends on**: Nothing (first phase)
**Research**: Unlikely (extending existing test patterns)
**Plans**: TBD

**Issues:**
- [x] #77 Improve Rust unit test coverage
- [x] #75 Add frontend component tests

### Phase 2: VB-Cable Integration
**Goal**: Automatic VB-Cable detection and silent installation for seamless dual-output
**Depends on**: Phase 1
**Research**: Likely (external software integration)
**Research topics**: VB-Cable silent install, Windows audio device detection, VB-Audio licensing requirements
**Plans**: TBD

**Issues:**
- [ ] #39 VB-Cable Integration
- [ ] #32 Device enumeration retry logic
- [ ] #38 Device Enumeration Caching

### Phase 3: Audio Core Polish
**Goal**: Improved volume control and loudness normalization for consistent audio
**Depends on**: Phase 2
**Research**: Likely (LUFS standard implementation)
**Research topics**: LUFS normalization (ITU-R BS.1770), Rust LUFS libraries, volume curve algorithms
**Plans**: TBD

**Issues:**
- [ ] #40 LUFS-Based Loudness Normalization
- [ ] #45 Volume Engine V2
- [ ] #29 Improve volume scaling UX

### Phase 4: Auto-Updater
**Goal**: Seamless updates so testers always have the latest version
**Depends on**: Phase 3
**Research**: Likely (Tauri updater plugin, code signing)
**Research topics**: tauri-plugin-updater, Windows code signing, GitHub Releases integration
**Plans**: TBD

**Issues:**
- [ ] #78 Implement Tauri Auto-Updater

### Phase 5: Import/Export
**Goal**: Export and import sound libraries for sharing and backup
**Depends on**: Phase 4
**Research**: Unlikely (standard JSON/ZIP handling)
**Plans**: TBD

**Issues:**
- [ ] #42 Import/Export Sound Library (JSON/ZIP)
- [ ] #61 Atomic JSON Persistence
- [ ] #51 Race Condition Protection (File Locking)

### Phase 6: UI/UX Polish
**Goal**: Final polish and fixes for a stable v1.0 release
**Depends on**: Phase 5
**Research**: Unlikely (internal UI patterns)
**Plans**: TBD

**Issues:**
- [ ] #43 Clear logs button
- [ ] #57 Settings Menu Overhaul
- [ ] #35 Hotkey edge case handling

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Test Coverage | 3/3 | Complete | 2025-12-29 |
| 2. VB-Cable Integration | 0/TBD | Not started | - |
| 3. Audio Core Polish | 0/TBD | Not started | - |
| 4. Auto-Updater | 0/TBD | Not started | - |
| 5. Import/Export | 0/TBD | Not started | - |
| 6. UI/UX Polish | 0/TBD | Not started | - |
