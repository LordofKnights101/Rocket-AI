# ROCKET — Complete Knowledge Reference

_The human-readable, machine-checkable reference for everything ROCKET is and does.
Every claim is grounded in code: exact values live in [`rules.json`](rules.json)
(regenerated from source by `cargo test`); design intent lives here._

**Status: current as of v0.2.0 (November 2026).** See [changelog.md](changelog.md) for
history. The medieval-scholarly progression layer (Order ranks, lore realms, the Codex
quest board, and Rocket's desk) is specified separately in [codex.md](codex.md).

---

## 1. What ROCKET is

ROCKET is a **single-user desktop gaming AI companion** (Rust + Iced 0.14, GPU-rendered
native window). It answers gaming questions from a curated knowledge base, learns facts
you teach it, remembers personal details, tracks your progress through an XP/coin
economy, co-op missions and daily quests, and can optionally answer *anything* through
an online LLM you bring a key for.

**It is not a multiplayer game.** The "player" is you; the co-op layer (missions) is
framing. Everything is local-first: no account, no server, all state in JSON files
next to the executable (see [§10 Persistence](#10-persistence)).

### 1.1 Module map

| Module | Role |
|---|---|
| `src/main.rs` | Iced app: UI pages, chat loop, game registry (`CATALOG`), XP triggers |
| `src/ai.rs` | The brain: answer pipeline, teaching, corrections, smalltalk |
| `src/knowledge.rs` | Curated knowledge packs + search scoring |
| `src/memory.rs` | Profile, XP/levels/coins, quests, missions, shop, titles |
| `src/llm.rs` | Online AI mode (9 providers), system prompt, fallback |
| `src/vibe.rs` | 5 personalities + line selection |
| `src/rocket.rs` | Canvas-drawn mascot (deterministic; flame grows while typing) |
| `src/detect.rs` | Auto game detection via running processes (Windows) |
| `src/theme.rs` | Colors/styles |
| `src/game_qa.rs` | Test that regenerates `rocket-qa-report.md` from the live pipeline |
| `src/docs_gen.rs` | Test that regenerates `docs/rules.json` + `docs/games.json` |

---

## 2. The answer pipeline (complete dispatch order)

Every chat message is normalized then dispatched **in this exact order**; the first
handler that matches answers and stops. (Source: `ai.rs Brain::answer`.)

| # | Handler | Fires when |
|---|---|---|
| 1 | `try_correction` | message starts with a correction prefix: `wrong:`, `wrong `, `correction:`, `correction `, `thats wrong` |
| 2 | follow-up | message is exactly `more`, `another`, `next`, `another tip`, `next tip`, `tell me more`, `more tips` |
| 3 | smalltalk | greetings, jokes (`joke`), thanks, "who are you" |
| 4 | `try_captures` | learnable personal facts: `my name is …`, `my favorite game is …`, `add game <name>`, skill level, loadout context |
| 5 | `recall_learned` | the question matches a **taught or corrected** answer |
| 6 | knowledge | curated pack of the selected game → universal item brain → generic topic brain → teach-me prompt |

### 2.1 Semantics and edge cases

- **Corrections (step 1) are permanent and highest priority.** A correction overwrites
  the memory slot for *the last question ROCKET answered* (`Brain.last_answered`).
  Afterwards the corrected answer is served at step 5 — i.e. before built-in facts — forever.
  Persisted in `rocket-brain.json`.
  - `wrong:` with no prior answered question → ROCKET explains how corrections work
    (unit test: `correction_without_question_explains`).
  - A question merely *containing* "wrong" is not a correction
    (test: `normal_questions_containing_wrong_are_not_corrections`).
- **Learned answers beat built-in facts** (step 5 before step 6). The more you teach,
  the more ROCKET answers from its fast path.
- **Teaching flow:** if the knowledge pipeline cannot answer, ROCKET says it doesn't
  know and asks to be taught; your next `teach: <answer>`-style reply is stored and
  then served first on the same question.
- **Selected-game isolation:** with a game selected, step 6 returns ONLY that game's
  pack entries (plus the universal item brain). Another game's entries surface only in
  **General Chat** mode (the `Generic` pseudo-game), via `search_any`
  (enforced by `game_qa` tests).
- **Invalid/empty input** falls through to the knowledge step and produces the
  teach-me/help response — the pipeline never panics on user input.

### 2.2 Other chat commands

| Input | Effect |
|---|---|
| `add game <name>` | registers a new game (custom games persist in `rocket-brain.json`), then it can be taught |
| `what games do you know` | lists the full registry (166 + custom + General Chat) |
| `what do you know about me` | dumps memory: name, favorite, facts, interests |
| `forget everything` | clears learned memory (tested: `forget_everything_clears_and_confirms`) |
| `be funny` / `vibe: mentor` / `act chill` … | switches personality (prefixes: `be`, `act`, `go`, `switch to`, `turn on`, `vibe`, `set vibe to`); unknown vibe → buddy fallback |
| mission creation | "help me beat the ender dragon"-style asks add a co-op mission (see §6) |
| win claims | "I finally beat X" completes a tracked mission (+75 XP) or celebrates untracked (+25 XP) |

---

## 3. XP economy (exact values)

All values below are implemented in `memory.rs` / triggered in `main.rs`; the JSON
mirror lives in `rules.json → leveling.xp_awards`. XP is only ever added.

| Event | XP | Notes |
|---|---|---|
| User chat message | **10** (+5 at level ≥10, +10 at ≥25, +15 at ≥40) | `LevelingData::award_conversation_xp`, fired in `add_message` |
| Recorded Q&A | **+10** | `Memory::add_conversation` — see [§9 Quirks](#9-known-quirks-and-limitations) |
| Teach a fact | **+25** | + `🔍 Knowledge Seeker` badge on first teach |
| Correction (`wrong:`) | **15** (+10 at level ≥30) | fired when a correction is processed |
| Follow-up (`more`) | **+15** | `🔁 Follow-up Pro` badge after 5 follow-ups |
| New topic mastered | **+20** | once per topic; repeats give 0 |
| Game switch | **+5** | `👑 Game Master` badge at 3 distinct games |
| Daily streak bonus | streak 1–3 → **5**; 4–7 → **15**; 8–14 → **30**; 15–30 → **50**; 31+ → **75** | fired with a user message when streak > 1 |
| Mission completed (tracked) | **+75** | |
| Untracked win celebrated | **+25** | |
| Daily quest completed | **25–60** (per-quest, see rules.json → daily_quests.pool) | paid the instant the target is hit |

### 3.1 Streak rules (`memory::update_streak`)

- Activity on a new calendar day with yesterday active → streak +1.
- Any fully missed day → streak resets to **1**.
- Same-day repeat activity → unchanged.
- Timestamps: unix seconds in `last_activity` (local-date comparisons).

### 3.2 Leveling

- `LEVEL_XP_REQUIREMENTS` — 50 entries, XP needed **at** each level (level 1 = 0 XP,
  level 2 = 100, …, level 50 = 180000). Nondecreasing (asserted by the generator test).
- Level-up happens inside `add_experience`: while `experience_points >= requirement`,
  the level advances and surplus XP carries over.
- **Rank titles by level:** 1–4 🌱 Rookie · 5–9 🛰️ Cadet · 10–14 🚀 Pilot · 15–19 ✈️ Ace Pilot ·
  20–24 🛡️ Guardian · 25–29 ⚔️ Veteran · 30–34 🏆 Elite · 35–39 👑 Champion ·
  40–44 🌟 Legend · 45+ 💫 Mythic. Full per-level map in rules.json.

### 3.3 Coins

- **Invariant: `coins == total_xp_earned`** at all times. Every XP earned mints a coin.
- Spending never touches level or level-progress XP — only the spendable coin balance.

---

## 4. Daily quests

- **3 distinct quests per day**, rolled by Fisher–Yates shuffle of the 7-quest pool,
  taking the first 3.
- Rerolled when the local calendar day changes (`quest_day` = `YYYY-MM-DD`); the
  per-day `games_today` list resets with it.
- Progress: `quest_progress(kind, n)` advances every active quest of that kind;
  progress clamps at target; **completion pays XP immediately** and toasts
  `✅ Quest complete: <title> (+<xp> XP)`.
- Quest kinds and their triggers: `ask` (user question), `chat` (conversation),
  `teach` (fact taught), `explore` (switch between ≥2 games in one day),
  `item` (item/gear question), `kb` (open the Knowledge Base page).
- Pool (kind · target · XP): ask 5/50 · ask 3/30 · chat 10/45 · teach 1/60 ·
  explore 2/40 · item 1/30 · kb 1/25. **Machine-readable:** rules.json → `daily_quests.pool`.

---

## 5. XP Shop & mystery box

**Decoration prices (coins, by rarity):** Basic 0 · Common 300 · Rare 800 · Epic 1500 · Legendary 3000.

- Buying charges once; owned decorations can't be re-bought.
- Equipping requires unlocked state; **one decoration per category** — equipping
  unequips all others in that category (Frame / Background / Icon / Animation / Title).

**Mystery box — 500 🪙:**
- Outcome: **uniformly random among currently LOCKED decorations**
  (`pick = rand(0..locked.len())` — no rarity weighting).
- If everything is owned: purchase refused, **no coins spent**.
- Insufficient coins: refused with exact cost and balance; no partial spend.

---

## 6. Co-op missions

- Model: `{title, game, done}` — e.g. "beat the ender dragon" (Minecraft).
- **Maximum 5 open missions**; a 6th is refused ("Mission board is full").
- Re-adding an active mission is refused ("already on our board").
- Completion detection: `parse_mission_completion` matches win phrases in chat;
  `complete_mission_by_text` matches against open board entries (fuzzy text match).
- Tracked completion → **+75 XP** + vibe-specific celebration with `{mission}`
  replaced by `title (game)`. Untracked win → **+25 XP** + celebration.

---

## 7. Progression content catalogs

- **Titles:** 105 collectible titles (`PROFILE_TITLES`), each `{id, icon, name, rarity,
  condition}`; conditions: Level / Streak / Questions / Facts / Topics / Badges / TotalXp.
  Unlock automatically via `check_progress_rewards`; exactly one equipped at a time.
  Full list: rules.json → `titles`.
- **Badges:** 15 types (see rules.json → `leveling.badges`) — conversation, learning,
  streak, exploration and special/reserved categories.
- **Achievements:** predefined catalog from `get_default_achievements()` with rarity
  Common/Rare/Epic/Legendary and unlock dates. Full list: rules.json → `achievements`.
- **Decorations:** default catalog from `get_default_decorations()` with category +
  rarity; unlock via XP thresholds, purchase, or mystery boxes. Full list: rules.json
  → `decorations`.

---

## 8. Knowledge base (curated content)

- Entry shape: `{game, category, title, content, tags}` with 6 categories:
  🎯 Strategy, ⚙️ Mechanics, 🗡️ Items & Gear, 👥 Characters & Roles, 📈 Progression, 📖 Glossary.
- **Search scoring:** stopword-filtered token scoring; substring matches count; a token
  also matches when either side is a prefix of the other and **both sides ≥ 4 chars**
  (`netherite`→`nether`, `diamonds`→`diamond`).
- **Live stats** (regenerated into rules.json → `knowledge_base` and games.json):
  166 catalog games (+ the General Chat pseudo-game = 167 pickable), **44 with
  curated packs**, **130 entries**, per-game/per-category counts.
- **Verification:** `game_qa.rs` re-asks 133 battery questions through the real
  pipeline and fact-checks them against ground truth; verbatim output in
  `rocket-qa-report.md`.

---

## 9. Known quirks and limitations

Documented honestly so the higher-level AI never has to guess:

1. **Double XP per question.** A single question fires both `add_message`
   (conversation XP, 10+bonus) and `Memory::add_conversation` (+10). A level-1 user
   therefore earns ~20 XP per question, not 10. Intentional pacing (kept for feel);
   documented here so economy math is correct.
2. **Correction XP path.** Correction XP is awarded in `App::get_brain`, which scans
   chat history for `wrong:` user messages — the award rides the system-prompt build,
   not the answer step itself.
3. **Mystery box has no rarity weighting** — uniform over locked decorations. A 3000-
   coin legendary and a 0-coin basic are equally likely from a 500-coin box.
4. **Streak XP can re-fire on every message of a new day** (streak > 1), not once per day.
5. **Auto-detection is Windows-only** (tasklist CSV); other platforms fall back to manual
   game selection. Detection polls every **6 seconds** and only while auto-detect is on.
6. **Browser-tab throttling note from the wasm era is obsolete** — the app is native now.
7. **The online AI is only as correct as its provider**; the local brain always stays
   available and any provider failure falls back with an explanation (see §11).
8. **API keys are stored in plaintext** in `rocket-llm.json` next to the app (by design,
   local-only; documented as a known tradeoff).
9. **Chat history persistence is capped** where it matters (`conversation_history` keeps
   the last 50 turns) — older turns are not recalled.

---

## 10. Persistence

| File | Owner | Contents |
|---|---|---|
| `rocket_memory.json` | memory.rs | profile, facts, XP/level/coins, badges, achievements, decorations, titles, quests, missions, vibe, streak |
| `rocket-brain.json` | ai.rs | taught answers, corrections, name, favorite game, custom games, topic hits |
| `rocket_chat_history.json` | main.rs | chat transcript |
| `rocket-llm.json` | llm.rs | online mode settings (mode, provider, key, model, custom URL) |
| `rocket_theme.json` | theme | theme selection |
| `rocket_detect.json` | detect.rs | auto-detect toggle |

All files are JSON, written next to the executable's working directory, loaded at
startup with tolerant defaults if missing/corrupt (fresh-profile safe — old saves are
migrated by `ensure_defaults`).

---

## 11. Online AI mode

- 11 providers (Groq, Cerebras, Gemini, Mistral, OpenRouter, DeepSeek, Together,
  OpenAI, Ollama, LM Studio, Custom), each with default base URL + model —
  **machine-readable list** in rules.json → `online_ai.providers` (extracted from
  `llm.rs Provider::ALL`).
- Request shape: OpenAI-compatible `POST {base_url}/chat/completions`.
- The system prompt (`llm.rs system_prompt()`) bakes in the user's learned facts, name,
  favorite game and corrections — online answers are still "your Rocket".
- **Fallback:** any online failure (no key, bad model, network, HTTP error) → local
  brain answers and ROCKET explains why. Local mode never touches the network.
- Ollama / LM Studio need no key (local runtimes).

---

## 12. UI / UX behavior

- **Pages:** Chat · Games (picker with genre/platform/era filters) · Profile (level,
  XP bar, coins, badges, titles, decorations, vibe, quests) · XP Shop · Knowledge
  (browsable packs) · Cheat Sheet · Settings (online AI, auto-detect).
- **Window:** 1020×780 default, centered, titled `ROCKET`, rocket window/taskbar icon
  (embedded via `build.rs`; runtime icon from `assets/rocket-icon.rgba`).
- **Clipboard:** Ctrl+V pastes into the input; per-message 📋 copy button
  (`iced::clipboard::write`).
- **Mascot:** deterministic canvas redraws (no jitter); flame grows while you type
  (`warming`); a launch bounce fires when a reply lands.

---

## 13. Coverage map (for the higher-level AI)

| Question type | Where the answer lives |
|---|---|
| "Exact rules for X?" | §2–§8 here + rules.json |
| "Rare edge case?" | §2.1, §9 + unit tests named inline |
| "Scoring when A and B happen?" | §3 XP table (both paths documented), §4, §6 |
| "Optimal play?" | strategy_notes.md |
| "Why this design?" | intent notes inline + changelog.md rationale |
| "Differences between versions/modes?" | changelog.md + rules.json → online_ai |
| "Every legal action in state S?" | §2.2 commands + §4 quest kinds + §6 mission rules |
| "Full progression/economy?" | §3–§7 |
| "Known bugs/limitations?" | §9 |
| "What content exists?" | rules.json → knowledge_base, titles, achievements, decorations; games.json; rocket-qa-report.md |
