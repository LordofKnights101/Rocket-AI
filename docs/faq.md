# ROCKET — FAQ

_Every answer below is grounded in the formal rules ([knowledge.md](knowledge.md),
[rules.json](rules.json)). Values in bold are exact._

## Chat & answers

**How does ROCKET decide what to answer?**
Six-step dispatch: corrections → follow-ups ("more") → smalltalk → personal-fact
capture → learned answers → knowledge base. First match wins; see §2 for the full
state machine including edge cases.

**I taught ROCKET something — will it ever forget?**
No. Taught answers persist in `rocket-brain.json` and are matched *before* built-in
facts, permanently, unless you overwrite them with `wrong: <correct answer>`.

**What exactly does `wrong:` do?**
It overwrites the memory slot for the **last question ROCKET answered**, awards
**15 XP** (+10 at level ≥30), and from then on the corrected answer is served first.
`wrong:` with nothing answered yet explains the mechanic instead of failing.

**Does "more" always work?**
`more` (and `another`, `next`, …) fires another tip on the previous topic — it needs a
previous answer to follow up on; otherwise it falls through to the normal pipeline.

**Will ROCKET leak another game's guide while I selected Minecraft?**
No. With a game selected, curated answers come only from that game's pack plus the
universal item brain. Cross-game answers exist only in General Chat mode
(enforced by tests in `game_qa.rs`).

## XP, levels & coins

**How much XP per question?**
Two documented paths fire per question: conversation XP (**10** base, +5/+10/+15 at
levels 10/25/40) plus **+10** for the recorded Q&A — so **~20 XP per question at level 1**.
This double-award is intentional pacing (knowledge.md §9.1).

**Fastest ways to earn XP?**
Teaching (**25**), completing daily quests (**25–60** each), missions (**75**), and
keeping a streak going (day 4+ bonus **15**–**75**). Full math in strategy_notes.md.

**What's the XP needed per level?**
50 levels from 0 XP (level 1) to **180,000 XP** (level 50). The full table is in
rules.json → `leveling.xp_requirements_indexed_by_level`. Ranks: Rookie → Cadet →
Pilot → Ace Pilot → Guardian → Veteran → Elite → Champion → Legend → Mythic.

**Do coins get spent when I level? Never?**
Never. **Coins always equal total XP earned**; spending only touches the coin
balance, never level progress.

**What does a mystery box give me?**
One random **locked** decoration, uniformly chosen — a 500-coin box has the same
chance at a Basic as a Legendary (no rarity weighting). If you own everything, the
purchase is refused and no coins are spent.

**Can I buy a decoration twice?**
No. Buying charges the rarity price once (Common 300 / Rare 800 / Epic 1500 /
Legendary 3000 / Basic 0); re-buying is refused.

**How many decorations can I equip?**
One per category (Frame / Background / Icon / Animation / Title); equipping one
unequips the others in its category.

## Quests & missions

**How are daily quests picked?**
**3 distinct** quests per day: Fisher–Yates shuffle of the 7-quest pool, take 3.
Rerolled the moment the calendar day changes.

**When do quests pay out?**
The instant the target is reached — `progress` clamps at `target` and the XP is
added immediately with a ✅ toast.

**How many missions can I track?**
**5 open maximum**; a 6th is refused. Duplicates are refused. Completing a tracked
mission pays **75 XP**; saying you beat something off-board pays **25 XP**.

**What if I never complete a quest?**
Nothing expires mid-day; unfinished quests simply get replaced at the next day's reroll.

## Games & knowledge

**Which games does ROCKET know?**
**166 registered games** (167 including General Chat), **44 with curated packs**
(130 entries total — live counts in games.json and rules.json → knowledge_base).
Ask `what games do you know` in the app for the list.

**How does auto-detection work?**
On Windows ROCKET polls running processes every **6 seconds** and switches to the
matched game (e.g. `robloxplayerbeta.exe` → Roblox). Off-platform: manual selection.

**How does the knowledge search match my question?**
Stopwords are dropped; substring matches count; prefix matching needs **both sides
≥ 4 characters** (so "netherite" finds "nether").

## Online AI

**Which providers are supported?**
Eleven: Groq (recommended), Cerebras, Google Gemini, Mistral, OpenRouter, DeepSeek,
Together, OpenAI, Ollama, LM Studio, Custom — each with a default model
(rules.json → online_ai.providers).

**What happens if my key is wrong or the network is down?**
ROCKET automatically answers from the local brain and tells you why online failed.
Local mode never touches the network.

**Where is my key stored?**
In `rocket-llm.json` next to the app, plaintext, local-only (known tradeoff — knowledge.md §9.8).

## The Order of the Eternal Codex

**What is the Order?**
The scholarly frame around everything: games are Realms, their knowledge packs are
grimoires, your progress is a character (rank, lore, quests, chronicle). See
[codex.md](codex.md) for the complete reference.

**How do I rise in rank?**
Your Order rank follows your level: Novice Scribe (1), Ink Apprentice (5), Journeyman
Scribe (10), Adept Scholar (15), Loremaster (25), Master Archivist (35), Grand
Archivist (45). Each rank unlocks new quest tiers.

**What are Lore Points?**
Per-realm scholarship credit: every grounded answer gives +1 (+2 if it touches a
discipline the realm has never seen). Tiers: Uncharted → Charted (10) → Studied (25) →
Examined (50) → Mastered (100) → Sanctified (200).

**What can Rocket's desk do?**
Practical tasks, not oracle work: quest reports, trial generation, grimoire audits and
gap lists, comparative tables, character reports, chronicle export, and the auditable
action log. It never invents game rules — audits read live knowledge-base stats.

**Who watches Rocket?**
You do: every state-modifying desk action is appended to an audit log (`log` command),
and notable deeds are inscribed in the chronicle (`chronicle` exports it to
`codex_chronicle.md`).

**What is CREATE TREE?**
Rocket's quest-tree atelier: commission a persistent, hierarchical quest tree for any
charted game (`create tree for minecraft`), keep many trees side-by-side as tabs,
minimize/restore/rename/duplicate/archive/destroy them at will, and refine them
(`expand`, `rebalance`, `note`, `validate`, `repair`). Every node traces to real
grimoire sections — trees never invent rules. See [codex.md](codex.md) §6.

**Do quest trees keep progressing while minimized?**
Yes. Events route through every living tree whose realm matches; a minimized tab is
just a compact view — progress, notes and revisions are all kept.

**What is the Rite of First Entry?**
Rocket's guided initiation for new scholars: eight resumable stages that teach the
dual channels, asking-and-earning, Rocket's office, CREATE TREE commissioning, tab
literacy and progression — with glowing highlights marking exactly where to act.
Completing it grants **+100 pages** and a chronicle entry. `skip the rite` marks it
seen; `restart the rite` reruns it; `remind me …` re-teaches any part at any time.
See [codex.md](codex.md) §8.

## Housekeeping

**Where does ROCKET keep everything?**
Six JSON files next to the app (knowledge.md §10). Backing up = copying them.

**Is there anything ROCKET can't do?**
See knowledge.md §9 (known quirks): Windows-only auto-detect, no rarity weighting on
mystery boxes, plaintext key storage, chat history capped at 50 turns, and per-day
repeating streak XP — all documented with code provenance.
