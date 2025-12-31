# Phase 2: VB-Cable Integration - Context

**Gathered:** 2025-12-29
**Status:** Ready for planning

<vision>
## How This Should Work

VB-Cable Integration soll sich so automatisch wie möglich anfühlen. Der User geht in die Settings, findet dort einen kompakten VB-Cable Bereich, und mit einem Klick kümmert sich SonicDeck um alles was technisch möglich ist.

SonicDeck lädt VB-Cable herunter, startet den Installer - der User muss nur den unvermeidbaren Windows Driver-Approval-Dialog bestätigen. Danach wird VB-Cable automatisch als Secondary Output konfiguriert, und das ursprüngliche Windows Default Audio Device wird wiederhergestellt (VB-Cable ändert das während der Installation).

Der ganze Prozess ist inline in den Settings, kein separater Wizard oder Modal. Kompakt, integriert, professionell.

</vision>

<essential>
## What Must Be Nailed

- **Nahtlose Installation** - Der Prozess muss sich mühelos anfühlen, trotz der Windows-Einschränkungen
- **Zuverlässige Detection** - SonicDeck muss VB-Cable immer korrekt erkennen (installiert/nicht installiert)
- **Keine Audio-Störung** - Nach Installation darf am Audio-Setup nichts kaputt sein (Default Device wiederherstellen)
- **Auto-Configuration** - Nach erfolgreicher Installation wird VB-Cable automatisch als Secondary Output gewählt

</essential>

<boundaries>
## What's Out of Scope

- Voicemeeter oder andere komplexe VB-Audio Produkte - nur VB-Cable
- Bezahlte VB-Cable Varianten (A+B, C+D) - nur das kostenlose Standard VB-Cable
- Discord-spezifische Konfiguration - User muss Discord selbst auf VB-Cable Output stellen
- Komplexes Troubleshooting bei Fehlern - simple Fehlermeldung reicht, nicht überkomplizieren

</boundaries>

<specifics>
## Specific Ideas

- **Trigger:** Eigener Menüpunkt in Settings, User startet es bewusst
- **UI:** Inline in Audio Settings, kompakter Bereich mit Status + Action Button
- **Nach Install:** VB-Cable automatisch als Secondary Output wählen
- **Default Device:** Automatisch wiederherstellen nach Installation
- **Fehlerfall:** Einfache Fehlermeldung, simpel halten

</specifics>

<notes>
## Additional Context

User will "Maximum Automation" - alles was technisch möglich ist soll automatisiert werden. Der einzige manuelle Schritt ist der Windows Driver-Approval-Dialog (kann nicht umgangen werden).

Die Vision passt gut zur bereits abgeschlossenen Recherche (02-RESEARCH.md), die genau diese Punkte adressiert:
- `com-policy-config` crate für Default Device Save/Restore
- cpal für VB-Cable Detection
- Silent Install Flags (`-i -h`) für maximale Automatisierung

</notes>

---

*Phase: 02-vb-cable-integration*
*Context gathered: 2025-12-29*
