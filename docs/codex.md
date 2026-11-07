# 📜 The Order of the Eternal Codex — Progression Reference

_The complete, machine-checked reference for ROCKET's medieval-scholarly progression
layer. Numeric truth lives in `rules.json → codex` (extracted from `src/codex.rs` by
the generator test); design intent lives here._

**Status: current as of v0.2.0 (November 2026).** Companion to [knowledge.md](knowledge.md).

---

## 1. Identity and tone

The whole experience is membership in the **Order of the Eternal Codex**: every game in
the registry is a **Game-Realm** whose curated knowledge pack is its **grimoire**. Asking
questions is scholarship; answers are pages transcribed; quests are pinned to a board by
the Order. Surface copy uses the Order's voice (marginalia, proclamations, quills); the
underlying data stays precise and machine-readable. Modern jargon is avoided in new
narration; legacy strings (XP toasts from the older economy) remain as historical copy.

**Rocket** is re-framed as the Order's **familiar** — not the primary oracle (the
grimoires are), but the user's research assistant and agent of practical action.

---

## 2. Character model

| Attribute | Source |
|---|---|
| Level / XP / coins | the pre-existing economy (knowledge.md §3) — the Codex never duplicates it |
| **Order Rank** | derived from level — see ladder below |
| **Lore Points per Game-Realm** | `codex_character.json → realms.<name>.lore_points` |
| **Realm mastery tier** | derived from Lore Points — see tiers below |
| **Quest board** | `codex_character.json → board` |
| **Chronicle** | `codex_character.json → chronicle` (notable deeds, cap 60) |
| **Audit log** | `codex_character.json → actions` (every state modification, cap 200) |
| **Desk history** | `codex_character.json → desk_history` (Rocket side-chat, cap 120) |
| Chronicle export | `codex_chronicle.md` (written by the `chronicle` desk command) |

State file: **`codex_character.json`**, `version: 1`, tolerant load (missing/corrupt →
fresh character). All records are human-inspectable JSON.

### 2.1 Ranks of the Order (level bands)

| Rank | Sigil | Levels | Unlocks |
|---|---|---|---|
| 🪶 Novice Scribe | 1–4 | Lore Mastery quests |
| 🖋️ Ink Apprentice | 5–9 | + Edge-Case Expeditions |
| 📜 Journeyman Scribe | 10–14 | + Comparative Scholarship, desk quest-authoring |
| 🕯️ Adept Scholar | 15–24 | + Restoration Quests |
| 📖 Loremaster | 25–34 | + Trials by Ordeal |
| 🗝️ Master Archivist | 35–44 | all of the above (deep-analysis privilege) |
| 👑 Grand Archivist | 45+ | everything |

The rank is displayed in the sidebar identity card and the Scriptorium header; the
level-up toast is narrated as a proclamation (`📜 PROCLAMATION — the Order elevates
you…`).

### 2.2 Realm mastery tiers (Lore Points)

`Uncharted (0) → Charted (10) → Studied (25) → Examined (50) → Mastered (100) →
Sanctified (200)`. Tier rises are narrated (`📖 The <realm> grimoire rises to <tier>`)
and inscribed in the chronicle.

---

## 3. The XP model (exact)

The Codex adds bonuses **on top of** the standard economy (knowledge.md §3), awarded by
`codex.rs record_realm_question` for every **knowledge-grounded** answer:

| Factor | XP | Lore | Condition |
|---|---|---|---|
| Grounded answer | 0 | **+1** | answer came from a grimoire (Game / CrossGame / UniversalItem) |
| Novelty | **+5** | (included → +2 total) | first time this realm touches that tag/discipline |
| Depth | **+3** | — | question is ≥ 6 words |

Non-grounded answers (the legacy topic brain) award nothing and **reset the ordeal
streak** — noted in the chronicle without punishment (the Order values honest inquiry;
partial progress is never clawed back).

---

## 4. Quest architecture

Board capacity: **6 open quests**; quests are pinned automatically whenever the board
has room (on Scriptorium open, after every grounded answer, after desk sends) and are
**derived from the live knowledge base — never invented**.

### 4.1 The five disciplines

| Discipline | Quest shape | Target | Reward (pages/lore) | Rank gate |
|---|---|---|---|---|
| 📖 Lore Mastery | transcribe N answers in a realm's strongest discipline | 3 | 60 / 5 | Novice Scribe |
| 🧭 Edge-Case Expedition | surface N never-before-seen entries of a realm's rarest discipline | 2 | 80 / 8 | Ink Apprentice |
| ⚖️ Comparative Scholarship | ask in the same discipline across two named realms | 1 + 1 | 90 / 10 | Journeyman Scribe |
| 🔧 Restoration | teach N facts to fill a thin grimoire (≤ 2 entries) | 2 | 100 / 12 | Adept Scholar |
| ⚔️ Trial by Ordeal | N consecutive grounded answers without faltering | 3 | 120 / 15 | Loremaster |

Every quest declares: id, discipline, title, realm(s), category, target, progress,
rewards, required rank, and the **grimoire sections it exercises** (visible on the
board and in desk reports).

### 4.2 Generation rules

- Candidate realms are ranked **underexplored-first** (fewest recorded questions among
  realms that have grimoire entries at all).
- Lore/Edge quests pick the realm's strongest / rarest discipline respectively.
- Comparative quests pick two realms that share a discipline.
- One open quest per discipline maximum; generation stops when the rank unlocks nothing
  new or the grimoires are exhausted for fresh material.
- All pinning is logged (`order / quest_pinned`).

### 4.3 Completion

Success criteria are **pure counters driven by real events** (answers with their tag,
teach events, ordeal streak) — nothing is self-reported. Sealing pays quest XP through
the standard `add_experience` and quest lore to the primary realm, narrates
`✒️ Quest sealed: … (+X pages, +Y lore)`, and inscribes the deed in the chronicle and
audit log. Changing the knowledge base does not corrupt open quests: progress counters
keep counting events; quest targets are event counts, not content hashes (migration
rule of the spec §5 satisfied by construction).

---

## 5. Rocket's desk (side-chat agent)

A distinct conversational channel on the **📜 Scriptorium** page — separate history,
separate input, never mixed with main-channel scholarship (main-channel messages feed
XP/quests; desk messages do not).

### 5.1 Command surface

| Command | Effect |
|---|---|
| `help` | capability list |
| `quest report` / `board` | open quests with progress, rewards, exercised sections |
| `set me a trial` / `generate challenge …` | pins a fresh quest (Journeyman+; refused with requirement named below that) |
| `audit <game>` | completeness audit: entries, disciplines covered 6/6, missing list, realm standing, verdict |
| `gaps <game>` | missing disciplines of a grimoire |
| `compare <a> and <b>` | markdown table of discipline counts across two realms |
| `where stand I` / `report` | rank, realms charted, open/sealed quests, ordeal streak, top realms with tiers |
| `chronicle` | exports `codex_chronicle.md` (deeds, newest first) |
| `log` | last 12 audit-log entries |
| `interjections on/off` | permission: may Rocket append marginalia to main-channel answers? |
| `clear the desk` | wipes desk history only |

Anything else gets a conversational reply with the command list. Game names are
fuzzy-matched against the registry (substring, ≥ 3 chars).

### 5.2 Data integrity (spec §5)

- Rocket **never invents game rules**: audits/gaps/compare read live `KnowledgeBase`
  stats; all factual output is counts and names from the registry/packs.
- Every desk action that modifies state is appended to the **audit log**
  (`actor: rocket`) and is reviewable via `log` — the chronicle requirement of the spec.
- Desk history is persistent (resumable long-running research); Rocket's working
  context = the desk history + codex state.

### 5.3 Interjections (permission-gated)

With marginalia ON, main-channel answers that seal quests or raise realm tiers append a
one-line note (`🕯️ Rocket's marginalia: …`). With OFF (default), the same events appear
only as toasts on the main channel.

---

## 6. 🌳 CREATE TREE — the quest-tree atelier

A first-class subsystem under Rocket's domain: a **persistent, multi-instance
authoring environment** for Quest Trees. Each tree is a self-contained hierarchical
overlay of quests, challenges, lore-gates and progression nodes — commissioned by the
user, generated by Rocket **exclusively from the authoritative grimoires**, and kept as
a living, fully addressable document in its own tab.

**Trees are never a source of truth for game rules** — they are pedagogical overlays.
Every node's `grimoire_sections` are validated against the live knowledge base; stale
nodes are flagged (⚠️) and repairable.

### 6.0 The visual graph and the Create Tree button

- **Entry point**: a prominent **🌳 Create Tree** (or **Create Tree (Game)** when a
  game is selected) button sits atop the atelier. Pressing it starts Rocket's guided
  interrogation, whose first question is always *"What game quest tree do you want me
  to build?"* — one game answer, one shape answer (or `default`), and the full graph
  is generated. No confirmation loops.
- **The graph**: each quest is a **circular node** — ❓ available (frontier, gold
  ring), ✓ sealed (filled, stroke-drawn check), · locked (dim dot) — laid out
  automatically in tiers as columns with edges showing prerequisites. **Edges glow and
  pulse** the moment their source node seals, lighting the forward path. Stale nodes
  ring red; optional branches wear a halo.
- **Synchronized text**: clicking a node selects it; the detail card below shows the
  full decree (description, criteria progress, XP, failure guidance, linked grimoire
  sections) with a **✔ Mark Done** action for available nodes. Locked nodes explain
  their prerequisites in character instead of erroring.
- **Outline on demand**: `outline tree <name>` prints the pure textual outline;
  `confirm saved` makes Rocket verify and report that every tree's state is safely
  written.
- **Tab reorder**: `reorder tree <name> up|down` or the ◀ ▶ chips beside the tab row.
  Closing a tab (minimize/archive) never destroys a tree — only `destroy tree` does,
  and it is audited.

### 6.1 Commission grammar (via Rocket's desk)

```
create tree [archetype] for <game> [and <game2>] [depth N] [breadth N]
            [gentle|steep] [flavor: <free text>] [named <title>]
```

- `<game>` must resolve to a registry realm (fuzzy match); a blank grimoire refuses
  generation (teach it first).
- Archetypes and rank gates: Linear Mastery (Novice+), Branching Web / Narrative
  Chronicle (Ink Apprentice+), Comparative Network / Restoration Campaign (Journeyman+),
  Ordeal Ladder / Timed Trials (Adept Scholar+), Edge-Case Gauntlet (Loremaster+).
- Defaults: depth 3, breadth 2, linear curve, name `<Game> — <Archetype> Tree`.
- `depth` clamps to 1–8, `breadth` to 1–3; `gentle`/`steep` shape the XP economy
  (`30+15t` / `40+20t` / `30+30t` per tier).

### 6.2 Node anatomy (every node declares)

title · description · machine-checkable success criteria (grounded answers bound to a
discipline, teach events, ordeal runs, per-realm comparative answers) · failure/partial
guidance · XP reward · rank prerequisite · exact grimoire sections exercised ·
relationships (parents = prerequisites, optional side-branches, mutually exclusive
rivals, convergence points).

### 6.3 Tab lifecycle (desk commands)

| Command | Effect |
|---|---|
| `trees` | every living document, one-line summaries (incl. minimized) |
| `open tree <name>` / `restore tree <name>` | restore + focus (minimized/archived keep all state) |
| `minimize tree [name]` | compact tab; progress, notes, revisions preserved |
| `rename tree <name> to <new>` | rename, inscribed as a revision |
| `duplicate tree <name>` | independent copy ("(copy)"), same blueprint |
| `archive tree <name>` | leaves the tab bar; restorable |
| `destroy tree <name>` | unmade permanently; the deed is audited |
| `expand tree <name> depth N` | grafts N more tiers onto the frontier (surgical; ids re-mapped) |
| `rebalance tree <name> [to N]` | rescale the XP economy (optionally to an exact total) |
| `note tree <name>: <text>` | append a research note |
| `validate trees` | re-check all nodes against the current grimoires |
| `repair tree <name>` | re-bind stale nodes to living sections |
| `tree report <name>` | full node-by-node detail with the live frontier (⚔️) |

A tree is **focused** after creation/restore; deictic references (`minimize tree`,
`expand this tree`) address it. Clicking a tree tab in the Scriptorium restores it.

### 6.4 Progress and integration

Node criteria are **pure event counters** — grounded answers routed through the main
scholarship hook (with their discipline tag), teaching events, consecutive-grounded
ordeal runs. Events advance **every** living tree whose realm matches (minimized trees
keep accruing). Sealing a node pays its XP through the global `add_experience`
(economy, levels, coins, chronicle all continue); sealing all required nodes pays a
**10% Order bonus**. Mutual exclusivity rests rivals (skipped, not failed). Every
refinement is inscribed in the tree's revision history and the audit log.

### 6.5 Grimoire-drift integrity

On `validate trees`, any node whose section no longer exists in its grimoire is marked
⚠️ stale; `repair tree` re-binds it to a living discipline and inscribes the repair.
(Knowledge packs are compile-time, so drift only occurs across versions — validation is
available on demand and reported honestly.)

---

## 7. UI surfaces

- **Sidebar identity card** — level, realm rank, Order rank, progress bar captioned
  "✒️ N pages to the next volume", coins.
- **📜 Scriptorium page** — Order rank header, Rocket's desk (transcript + input +
  🎲 trial button + marginalia toggle + record counts), the 🌳 CREATE TREE atelier
  (tabbed trees, node panels with live progress), and the Quest Board.
- **Toasts** — proclamations (level-ups), marginalia (quest seals), realm rises.

---

## 8. 🕯️ The Rite of First Entry — guided initiation

A first-run **Guided Initiation Protocol**: for any profile without a completed
initiation flag, Rocket receives the scholar through eight resumable stages — in the
Order's voice, on the desk, with precise visual interventions.

### 8.1 State machine

`Stage::Detection → Greeting → FirstQuestion → MeetRocket → CommissionTree →
TabLiteracy → Synthesis → Closing → (complete)`, with **sub-phases** (sequenced
highlights) inside Greeting (2), CommissionTree (3) and TabLiteracy (2). State lives in
`codex_character.json → initiation` (`active, completed, stage_index, sub_step,
yielded, nudged, skipped, highlight`) — a scholar who leaves mid-rite resumes exactly
where they stopped, with a welcome-back note.

**Stage → highlight target → observable mastery action:**

| Stage | Target(s) | The scholar must actually… |
|---|---|---|
| Greeting (The Summons) | `main_chat_input`, then `desk_input` | send a word on the desk (×2 phases) |
| FirstQuestion (First Counsel) | `main_chat_input` | ask a question in the Main Hall — e.g. the *real* grimoire example `how do i find diamonds in minecraft` |
| MeetRocket (Meeting the Familiar) | `desk_input` | address Rocket (`help`) |
| CommissionTree | `scriptorium_nav` → `desk_input` → `tree_atelier` | open the Scriptorium → send `create tree for minecraft depth 2 gentle` → behold the new tab |
| TabLiteracy (The Art of Many Trees) | `tree_tabs` | `minimize tree` → then restore it |
| Synthesis (The Scholar's Compass) | `rank_card` | view progression (Profile / `where stand I`) |
| Closing | `desk_input` | final word → **+100 pages**, rite inscribed in the chronicle |

### 8.2 Highlight API

`Highlight { target, label, dim }` — targets are **layout-stable names**
(`HIGHLIGHT_TARGETS`), rendered by `main.rs::glow_or_card` as a gold border plus a
callout banner attached to the named element; `dim = true` adds a translucent,
click-through **spotlight veil** over the whole frame (iced stack overlay). Sequenced
highlights = ordered sub-phases. All effects clear on completion, skip, dismissal and
yield; the controller never highlights an undeclared target (unit-tested).

### 8.3 Behavioral contracts

- **Real-time detection** — events are emitted only from real outcomes: main questions
  (`add_message`), desk sends, tree commissions/minimizations/restorations, Scriptorium
  visits, progression views. The controller never advances on self-report.
- **Graceful yield** — an out-of-order real action hides the glow-marks ("you wander
  freely") until any desk message re-engages; nothing is skipped and no progress lost.
- **Gentle correction** — a wrong desk message nudges once per phase ("Almost, scholar
  — First Counsel asks for a question in the Main Hall"), then stays quiet.
- **Acknowledged override** — `skip the rite` marks every stage seen and inscribes the
  override in the chronicle; `restart the rite` re-runs from the Summons.
- **Re-explanation forever** — `remind me …` / `show me again …` on the desk
  re-teaches any topic (dual channels, asking & earning, Rocket, commissioning trees,
  minimizing-and-starting-another, progression).
- **Completion** — chronicle entry "Rite of First Entry", **+100 pages** through the
  standard economy, transition to free use.

---

## 9. Coverage map (progression layer)

| Question type | Where the answer lives |
|---|---|
| "What rank am I / what unlocks next?" | §2.1 ladder + rules.json → codex.ranks |
| "How much XP for X?" | §3 + rules.json → codex.xp_model |
| "What quests exist / how are they made?" | §4 + rules.json → codex.quest_types, quest_rewards |
| "How do quest trees work?" | §6 + rules.json → codex.tree_atelier |
| "Why did Rocket change a tree?" | tree revisions + audit log (`log`, `tree report`) |
| "How does the rite work / what stage am I on?" | §8 + rules.json → codex.initiation |
| "Can Rocket re-teach something?" | §8.3 re-explanation contract (`remind me …`) |
| "Why did Rocket do that?" | audit log (`log` command) — every modification recorded |
| "What has my character done?" | chronicle (`chronicle` command) |
| "What can Rocket do?" | §5.1 command surface |
| "Is anything invented?" | no — §5.2 integrity rules + unit tests |
