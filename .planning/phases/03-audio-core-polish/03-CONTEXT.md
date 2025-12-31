# Phase 3: Audio Core Polish - Context

**Gathered:** 2025-12-31
**Status:** Ready for planning

<vision>
## How This Should Work

Alle Sounds sollen automatisch auf einer konsistenten Lautstärke abgespielt werden - "Set and forget". Einmal die Normalisierung in den Settings aktivieren, und dann klingen alle Sounds ähnlich laut, egal ob sie ursprünglich viel zu laut, normal oder sehr leise waren.

Gleichzeitig soll der User die Möglichkeit haben, einzelne Sounds relativ zu anderen lauter oder leiser zu machen. Der bestehende Volume-Slider pro Sound bleibt erhalten, und optional gibt es einen zusätzlichen Gain-Offset für Feintuning.

Das Ganze soll sich nahtlos in die bestehende UI einfügen und klare visuelle Rückmeldung geben, was passiert (z.B. LUFS-Wert anzeigen). Performance darf nicht leiden - Sounds müssen genauso schnell abspielen wie vorher.

</vision>

<essential>
## What Must Be Nailed

- **Konsistente Lautstärke ohne Nachdenken** - Sounds sollen einfach gleich laut klingen, ohne dass man jeden einzeln einstellen muss
- **Natürlicher Volume-Slider** - Der Slider soll so reagieren, wie man es intuitiv erwartet
- **Performance** - LUFS-Berechnung darf die Wiedergabe nicht verzögern

</essential>

<boundaries>
## What's Out of Scope

- Keine expliziten Ausschlüsse definiert - offen für das, was sinnvoll ist
- Entscheidungen zu erweiterten Audio-Features (EQ, Effekte) können während der Planung getroffen werden

</boundaries>

<specifics>
## Specific Ideas

**Aktivierung:**
- Global an/aus Toggle in Settings (nicht pro Sound)

**Target-Level:**
- Preset-Auswahl statt Slider: "Leiser", "Normal", "Lauter"
- Einfache Optionen statt technische LUFS-Werte

**Kurze Sounds (<1 Sekunde):**
- Warnung anzeigen wenn LUFS-Wert unzuverlässig ist
- Trotzdem normalisieren, aber User informieren

**UI-Platzierung:**
- Gain-Offset Platzierung: Claude soll basierend auf bestehender UI-Struktur entscheiden
- Klare visuelle Rückmeldung (LUFS-Wert, Gain-Indikator)
- Nahtlose Integration in bestehende UI

**Volume-Slider:**
- Keine spezifische Präferenz für Kurventyp
- Was auch immer sich am natürlichsten anfühlt

</specifics>

<notes>
## Additional Context

Soundboard-Sounds sind sehr heterogen - von Meme-Clips bis Soundeffekten, von "viel zu laut" bis "sehr leise". Die Normalisierung muss mit dieser Vielfalt umgehen können.

User ist offen für technische Empfehlungen, da die Domäne (LUFS, Loudness) nicht sein Fachgebiet ist.

Bestehendes System hat Volume-Slider pro Sound + globalen Volume-Slider - diese Struktur soll erhalten bleiben, LUFS kommt als zusätzliche Schicht dazu.

</notes>

---

*Phase: 03-audio-core-polish*
*Context gathered: 2025-12-31*
