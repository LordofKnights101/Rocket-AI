# ROCKET — Changelog

_Versioned knowledge history. Entries marked **[reconstructed]** were rebuilt from
file dates, the QA report and code archaeology rather than written at the time —
they are historical context, not claims about exact commit history._

## v0.2.3 — 2026-11-06 (current)

**🌳 CREATE TREE — the interactive graph atelier** ([docs/codex.md](codex.md) §6):

- **Visual graph**: new `src/tree_graph.rs` canvas — circular nodes (❓ available /
  ✓ sealed / · locked) in an automatic tier-column layout, edges that **glow and
  pulse** the moment their source node seals, click-to-select hit-testing.
- **Create Tree button**: prominent `🌳 Create Tree (Game)` control opens a
  **conversational commission** — *"What game quest tree do you want me to build?"* —
  one game answer, one shape answer (or `default`), then the full graph. No extra
  confirmation loops; `cancel` rests it.
- **Mark Done**: clicking an available node shows its full decree with a Mark Done
  action (XP through the global economy, lit paths, tree-seal bonus). Locked nodes
  explain their prerequisites in character — never a hard error.
- **Tab management**: reorder via ◀ ▶ chips or `reorder tree <name> up|down`;
  `outline tree <name>` for the pure textual outline; `confirm saved` verifies every
  tree is safely written (auto-save already covers every change).
- Onboarding: the Rite's commissioning stage now spotlights the Create Tree button.
- 5 new tests (167 total): manual-completion gating, graph layout, outline,
  the full button→conversation→tree flow, desk outline/confirm/reorder.

## v0.2.2 — 2026-11-06

**🕯️ The Rite of First Entry** — guided first-run initiation
([docs/codex.md](codex.md) §8):

- New module `src/initiation.rs`: 8-stage resumable state machine (Detection,
  Summons, First Counsel, Meeting the Familiar, Commissioning, The Art of Many
  Trees, The Scholar's Compass, Closing) with sequenced sub-phases, persisted in
  `codex_character.json → initiation`.
- **Highlight API**: `Highlight { target, label, dim }` anchored to named,
  layout-stable elements — gold glow borders + callout banners
  (`main.rs::glow_or_card`, `rite_callout`) plus a translucent click-through
  spotlight veil; all effects fully reversible (unit-tested).
- Real-event advancement: main questions, desk sends, tree
  commissions/minimizations/restorations, Scriptorium visits, progression views.
  Graceful yield on free exploration; gentle once-per-phase corrections;
  acknowledged `skip the rite` override; `restart the rite`.
- Completion inscribes the **Rite of First Entry** in the chronicle and grants
  **+100 pages** through the standard economy.
- `remind me …` / `show me again …` re-explains any taught topic forever.
- 7 new tests (164 total), incl. the full rite walk, mid-rite persistence resume,
  yield/re-engage, skip override, re-explanations, and grounding of every taught
  example in real grimoires.

## v0.2.1 — 2026-11-06

**🌳 CREATE TREE** — Rocket's quest-tree atelier ([docs/codex.md](codex.md) §6):

- New module `src/codex_tree.rs`: multi-instance quest trees (tab semantics:
  open/minimized/archived/destroyed, rename, duplicate), 8 commissioning archetypes
  with rank gates, grimoire-grounded node generation, event-driven node completion
  (answers with discipline tags, teachings, ordeal runs), prerequisite/optional/
  mutually-exclusive/convergence relationships, revision history, grimoire-drift
  validation & repair, Order bonus (10%) on tree seal.
- Desk command surface: `create tree …`, `trees`, `open/restore/minimize/rename/
  duplicate/archive/destroy/expand/rebalance/note/validate/repair`, `tree report`.
- Scriptorium UI: tabbed atelier panel with per-node live progress.
- Integration: node seals pay XP through the global economy (levels/coins/chronicle
  unchanged); trees accrue progress while minimized.
- 11 new tests (150 total), including the full spec cycle: commission → generation →
  minimization → restoration → refinement → progress tracking.

## v0.2.0 — 2026-11-06

**The Order of the Eternal Codex** — the medieval-scholarly progression layer
([docs/codex.md](codex.md)):

- **New module `src/codex.rs`** (character state, ranks, realm lore, quest engine,
  chronicle, audit log, Rocket's desk) persisted to versioned
  `codex_character.json` (+ `codex_chronicle.md` export).
- **Order ranks** mapped onto the existing 50 levels: Novice Scribe → Ink Apprentice →
  Journeyman Scribe → Adept Scholar → Loremaster → Master Archivist → Grand Archivist,
  each unlocking quest tiers.
- **Per-realm Lore Points** with six mastery tiers (Uncharted → Sanctified); grounded
  answers award lore, novelty (+5 XP on first tag-touch per realm) and depth (+3 XP
  for 6+ word questions) — on top of the standard economy.
- **Five quest disciplines** generated from the live knowledge base (Lore Mastery,
  Edge-Case Expeditions, Comparative Scholarship, Restoration, Trials by Ordeal) with
  rank gates, declared rewards and exercised grimoire sections; success criteria are
  pure event counters.
- **Rocket's desk**: new 📜 Scriptorium page — side-chat with a practical command
  surface (quest report, trial generation, grimoire audits/gaps, comparative tables,
  character reports, chronicle export, audit log, interjection permission).
- **Auditable modifications**: every desk/order state change is logged (cap 200).
- **Order-voice narration**: level-up proclamations, "pages to the next volume"
  progress caption, quest-seal and realm-rise marginalia (permission-gated).
- Docs: `docs/codex.md`; `rules.json` gained the `codex` section (generator-extracted);
  12 new unit tests (rank bands, privileges, lore/novelty/depth, all five quest
  lifecycles, board gating, desk commands, audit caps, JSON round-trip).

## v0.1.1 — 2026-11-06

- **Rocket icon everywhere:** drew the mascot as a multi-size `.ico`
  (16/32/48/256) embedded into `rocket-ai.exe` via new `build.rs` (`winresource`);
  window/taskbar icon set at runtime from `assets/rocket-icon.rgba`; window title
  is now `ROCKET` (was `Rocket Ai - Iced`).
- **Knowledge package (this folder):** added `docs/` — knowledge.md, faq.md,
  strategy_notes.md, changelog.md, schema.json, plus `src/docs_gen.rs`, a test that
  regenerates `docs/rules.json` + `docs/games.json` from live code so data-driven
  docs cannot drift. Made `memory::QUEST_POOL` and `detect::GAME_PROCESSES` public
  for the generator (no behavior change).
- Launcher script fixes: `LaunchROCKET.bat` double-launch bug (missing `echo`),
  silent no-op `.Save` in `install_rocket.bat` / `create_shortcut.ps1`.
- Housekeeping: removed stale backups (`src/*.backup`, `main_backup.rs`) and junk
  files (`nul`, `dom.txt`, `edge-err*.txt`, `server.log`); rewrote root README for
  the desktop app.

## v0.1.0 — 2026-11-05 **[reconstructed]**

The desktop application as QA-verified on Nov 5 (see `rocket-qa-report.md`):

- 166-game registry, 43 curated knowledge packs, 133 fact-checked battery questions.
- **Meta-game layer:** XP/levels (50 levels, 100+ titles), coins economy, daily
  quests (7-quest pool, 3/day), co-op mission board (cap 5), XP Shop (decorations +
  mystery boxes), badges & achievements, profile (nickname/bio/picture).
- **Vibes:** 5 personalities (buddy, chill, funny, hyped, mentor) with vibe commands.
- **Online AI mode:** 9 providers incl. Ollama/LM Studio local runtimes, baked-in
  memory system prompt, automatic local-brain fallback.
- **Learning:** teaching, permanent `wrong:` corrections, personal-fact capture,
  custom games, corrections served before built-in facts.

## Earlier (2026-11-01 → 2026-11-03) — **[reconstructed]**

- **wasm era:** ROCKET began as an Iced app compiled to WebAssembly (trunk/browser,
  software renderer). Removed entirely — no wasm targets or entry points remain;
  the browser-tab throttling note in old READMEs is obsolete.
- Desktop transition (Nov 2–3): native window, launcher scripts (`ROCKET.bat`,
  `UPDATE-ROCKET.bat`, installers), auto game detection added (Nov 2), theme
  system, mascot canvas rendering.

## Knowledge-snapshot policy

- Current behavior is defined by `docs/rules.json` (regenerated from code) and
  `docs/knowledge.md` (kept in sync per change set).
- Historical entries here are never deleted; superseded values stay listed under
  their version so "what changed and when" questions are answerable.
- When you change a rule, add a new `## unreleased` section at the top and move it
  to a dated version on release.
