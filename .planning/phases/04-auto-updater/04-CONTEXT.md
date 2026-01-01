# Phase 4: Auto-Updater - Context

**Gathered:** 2026-01-01
**Status:** Ready for planning

<vision>
## How This Should Work

Wenn ein Update verfügbar ist, erscheint eine dezente Benachrichtigung in der Header/Titelleiste der App - kein aufdringlicher Dialog oder Popup. Der User entscheidet selbst, wann er das Update installieren möchte.

Beim Klick auf die Benachrichtigung sieht der User eine Kurzfassung der Änderungen (z.B. "3 neue Features, 2 Bugfixes") bevor er sich entscheidet. Das Update wird erst heruntergeladen wenn der User aktiv zustimmt.

Das Erlebnis soll ruhig und unaufdringlich sein - die App drängt nicht zum Update, informiert aber transparent über Neuerungen.

</vision>

<essential>
## What Must Be Nailed

- **Transparenz** - User sieht genau was sich ändert bevor er updated (Kurzfassung der Änderungen)
- **User-Kontrolle** - Kein automatischer Download, User entscheidet wann und ob
- **Dezente Benachrichtigung** - Header/Titelleiste zeigt Update-Verfügbarkeit ohne zu stören

</essential>

<boundaries>
## What's Out of Scope

- **Rollback-Funktion** - Zurück zur vorherigen Version zu komplex für v1.0
- **Code Signing** - Keine finanziellen Mittel für Zertifikat, SmartScreen-Warnung akzeptabel für Alpha-Tester
- **Update-Channels** - Keine separaten Alpha/Beta/Stable Kanäle
- **Automatischer Hintergrund-Download** - Download nur nach User-Aktion

</boundaries>

<specifics>
## Specific Ideas

- Benachrichtigung in der Header/Titelleiste (kleines Icon oder Badge)
- Changelog als Kurzfassung: "3 neue Features, 2 Bugfixes" statt volle Release Notes
- GitHub Releases als Update-Quelle
- tauri-plugin-updater als technische Basis

</specifics>

<notes>
## Additional Context

Aus der Assumptions-Diskussion vor diesem Kontext:
- Kein Code Signing geplant (finanzielle Gründe) - SmartScreen bleibt
- User-kontrollierte Updates sind wichtiger als Convenience

Zielgruppe sind Alpha-Tester die verstehen dass Software in Entwicklung ist.

</notes>

---

*Phase: 04-auto-updater*
*Context gathered: 2026-01-01*
