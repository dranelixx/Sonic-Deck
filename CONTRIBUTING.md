# Contributing to SonicDeck 🎵

Thank you for your interest in contributing to SonicDeck! We welcome contributions from everyone, whether you're fixing bugs, adding features, testing, or creating visual assets.

## 🤝 Ways to Contribute

### 1. 🐛 Bug Reports
Found a bug? Help us improve SonicDeck:
- Check [existing issues](https://github.com/DraneLixX/SonicDeck/issues) first
- Use the bug report template when creating new issues
- Include logs from `%LOCALAPPDATA%\com.sonicdeck.app\logs\`
- Run with `--debug` flag for detailed logs (see [README.md](README.md))

### 2. 🧪 Beta Testing
We need beta testers! Help us find bugs and improve UX:
- Download the latest alpha/beta release
- Test core features (audio playback, hotkeys, device switching, trim editor)
- Report issues with detailed steps to reproduce
- See [docs/testing/TESTING_GUIDE_EN.html](docs/testing/TESTING_GUIDE_EN.html) for testing instructions

### 3. 🎨 Visual Assets & Design
**We're actively looking for an artist!** Help us create:
- App icon and logo
- UI/UX design elements
- Banners and promotional graphics
- Screenshots and demo videos
- Marketing visuals

**This is an open-source community project** - contributions are unpaid but with credit.

**Contact:** Adrian Konopczynski
- 📧 Email: adrikonop@gmail.com
- 💬 Discord: dranelixx (ID: 624679678573150219)

### 4. 💻 Code Contributions
Contribute bug fixes, features, or improvements:
- Follow the development workflow below
- Read [AGENTS.md](AGENTS.md) for coding conventions
- Ensure code passes TypeScript, ESLint, and Rust checks
- Write clear commit messages (see Commit Guidelines)

### 5. 📝 Documentation
Improve docs, guides, or translations:
- Fix typos or unclear sections
- Add examples or tutorials
- Translate guides to other languages
- Update outdated information

---

## 🚀 Development Setup

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Yarn](https://yarnpkg.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup Steps
1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/SonicDeck.git
   cd SonicDeck
   ```

2. **Install dependencies:**
   ```bash
   yarn install
   ```

3. **Run development server:**
   ```bash
   yarn tauri dev
   ```

4. **Verify everything works:**
   ```bash
   yarn typecheck
   yarn lint
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

---

## 🔄 Development Workflow

SonicDeck uses **Git Flow Light** with a `develop` branch:

### Branching Strategy
- **Never commit directly to `main` or `develop`**
- **Branch naming:**
  - `feature/description` - New features
  - `fix/description` - Bug fixes
  - `chore/description` - Maintenance (docs, tooling)
  - `refactor/description` - Code refactoring

### Contribution Process
1. **Create a feature branch from `develop`:**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/my-awesome-feature
   ```

2. **Make your changes:**
   - Write clean, readable code
   - Follow conventions in [AGENTS.md](AGENTS.md)
   - Test your changes thoroughly

3. **Commit your changes:**
   - Use [Conventional Commits](https://www.conventionalcommits.org/)
   - Pre-commit hooks will run automatically (lint, format, typecheck)
   - See Commit Guidelines below

4. **Push and create a Pull Request:**
   ```bash
   git push -u origin feature/my-awesome-feature
   ```
   - Create PR to `develop` (not `main`!)
   - Fill out the PR template
   - CI will run automated checks

5. **Code Review:**
   - Address review feedback
   - Keep commits clean (use interactive rebase if needed)

6. **Merge:**
   - PRs to `develop` use **merge commits** (preserves history)
   - Maintainer will merge after approval

---

## 📋 Commit Guidelines

### Conventional Commits Format
```
type: short description

Problem:
- What issue this addresses

Solution:
- How you fixed it

Frontend (React):  [if applicable]
- Specific changes

Backend (Rust):  [if applicable]
- Specific changes
```

### Commit Types
- `feat:` - New features
- `fix:` - Bug fixes
- `refactor:` - Code refactoring
- `docs:` - Documentation
- `chore:` - Maintenance (deps, config)
- `test:` - Tests
- `perf:` - Performance improvements

### Examples
```
feat: add global volume control

Problem:
- Users cannot adjust master volume

Solution:
- Added volume slider in DashboardHeader
- Volume applies to all playback streams

Frontend (React):
- DashboardHeader.tsx: Volume slider component
- AudioContext: Global volume state
```

```
fix: external links not opening in browser

Problem:
- Clicking links in Settings did nothing

Solution:
- Added Tauri shell plugin integration
- Added shell:allow-open permission

Frontend (React):
- SettingsAbout.tsx: Use shell.open() with error handling

Backend (Tauri):
- capabilities/main-capability.json: Added shell:allow-open
```

---

## 🎨 Code Style

### TypeScript/React
- Functional components with hooks (no classes)
- Const arrow functions for components
- TypeScript strict mode - explicit types required
- ESLint + Prettier enforced (pre-commit hooks)
- See [AGENTS.md](AGENTS.md) for detailed conventions

### Rust
- `cargo fmt` standard formatting
- `snake_case` for functions/variables
- `///` doc comments for public functions
- Use `tracing` for logging (not `println!`)
- Explicit error handling (avoid `unwrap()` in production)

### General
- Max line length: 100 characters
- Indentation: 2 spaces (TS/TSX), 4 spaces (Rust)
- No trailing whitespace
- LF line endings (Git will auto-convert on Windows)

---

## ✅ Pre-Commit Checklist

Pre-commit hooks automatically run these checks, but verify manually if needed:

1. **TypeScript compilation:**
   ```bash
   yarn typecheck
   ```

2. **Linting:**
   ```bash
   yarn lint
   ```

3. **Rust compilation:**
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

4. **Run the app:**
   ```bash
   yarn tauri dev
   ```

5. **Test your changes:**
   - Manual testing of affected features
   - Check for console errors
   - Verify no regressions

---

## 🐛 Bug Report Guidelines

### Good Bug Reports Include:
1. **Clear description** of the issue
2. **Steps to reproduce** (be specific!)
3. **Expected behavior** vs actual behavior
4. **System information:**
   - OS version (e.g., Windows 11)
   - SonicDeck version (e.g., v0.8.0-alpha)
   - Audio devices used
5. **Debug logs:**
   - Run SonicDeck with `--debug` flag
   - Attach log file from `%LOCALAPPDATA%\com.sonicdeck.app\logs\`
6. **Screenshots/videos** if applicable

### Example Bug Report
```markdown
**Describe the bug**
Clicking a sound button with hotkey assigned causes app to freeze.

**To Reproduce**
1. Assign Ctrl+1 to "airhorn.mp3"
2. Click the airhorn button
3. App freezes for 2-3 seconds

**Expected behavior**
Sound should play immediately without freezing.

**System Information**
- OS: Windows 11 Pro (22H2)
- SonicDeck Version: v0.8.0-alpha
- Audio Devices: Logitech G935 + VB-Audio Cable
- Debug Mode: Yes

**Logs**
[Attach sonicdeck.2025-12-28.log]
```

---

## 💡 Feature Request Guidelines

### Good Feature Requests Include:
1. **Clear use case** - Why is this needed?
2. **Proposed solution** - How should it work?
3. **Alternatives considered** - What else did you think of?
4. **Additional context** - Screenshots, examples, etc.

### Example Feature Request
```markdown
**Feature Request: Soundboard Profiles**

**Use Case:**
I use different sound setups for gaming vs streaming. Switching between them is tedious.

**Proposed Solution:**
Add "Profiles" feature:
- Save current soundboard state (sounds, categories, hotkeys)
- Quick-switch between profiles via dropdown
- Import/export profiles as JSON

**Alternatives:**
- Manually duplicate and rename sounds
- Use multiple SonicDeck installations (clunky)

**Additional Context:**
Similar to OBS scene collections.
```

---

## 📞 Contact & Community

**Maintainer:** Adrian Konopczynski (DraneLixX)
- 📧 Email: adrikonop@gmail.com
- 💬 Discord: dranelixx (ID: 624679678573150219)
- 🐛 GitHub Issues: [Report a Bug](https://github.com/DraneLixX/SonicDeck/issues)

**Resources:**
- [AGENTS.md](AGENTS.md) - Developer documentation and conventions
- [VERSION.md](VERSION.md) - Version management workflow
- [CHANGELOG.md](CHANGELOG.md) - Project changelog
- [Testing Guides](docs/testing/) - Beta testing instructions

---

## 📄 License

By contributing to SonicDeck, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

**Thank you for contributing to SonicDeck!** 🎉

Every contribution helps make SonicDeck better for the community. Whether you're fixing a typo, reporting a bug, or adding a major feature - we appreciate your help!
