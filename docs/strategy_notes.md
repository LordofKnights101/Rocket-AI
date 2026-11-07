# ROCKET — Strategy Notes

_Optimization math for the meta-game (XP, coins, progression). All constants from
[rules.json](rules.json); formulas from [knowledge.md](knowledge.md) §3–§7._

## 1. XP rates (level 1 baseline, no bonuses)

| Activity | XP | Cadence |
|---|---|---|
| One question asked & answered | ~20 (10 conversation + 10 recorded) | per question |
| Teaching one fact | 25 | per fact |
| Follow-up (`more`) | 15 | per follow-up |
| Daily quest (avg) | (50+30+45+60+40+30+25)/7 ≈ **40** | 3 rolled/day, so ~120/day achievable if all 3 complete |
| Tracked mission | 75 | per mission |
| Streak bonus (day 2+) | 5 → 75 by streak length | per user message on the new day |
| Topic mastery | 20 | once per topic |
| Game switch | 5 | per switch |

**Best XP-per-interaction:** teaching (25) > follow-up (15) > question (~20 total).
**Best passive income:** quests (~120 XP/day for ordinary usage) — completing daily
quests is the single largest daily chunk for a casual player.

### 1.1 Level-10 milestone math

The exact per-level XP table lives in rules.json →
`leveling.xp_requirements_indexed_by_level` (extracted from code, always current).
At ~20 XP/question plus ~120 XP/day of quests, the early levels fall within **a few
evenings of regular use**; the +5 level-10 conversation bonus then compounds every
question afterwards.

## 2. Coin economy

- Coins == total XP ever earned; spendable without touching level progress.
- **Prices:** Common 300 · Rare 800 · Epic 1500 · Legendary 3000 · Mystery box 500.

### 2.1 Mystery box vs direct buy — the honest math

The box is a **uniform draw over locked decorations** (no rarity weighting). If your
remaining locked pool is mostly cheap/Common, the box is great value; if you already
own the commons, the box's expected value rises but its **variance** also rises.

Rule of thumb derived from the mechanics:
- If a specific decoration is your goal → **direct buy is strictly better** (prices are
  known, outcomes are certain).
- If you want *anything* new and your locked pool is large → boxes are efficient
  (500 per unlock vs 300–3000 direct, as long as locked count is high).
- Near-complete collection → boxes are the only way to finish (they're the only
  randomized source; direct buying still works if coins allow).

### 2.2 Earning order (recommended)

1. Do the 3 daily quests every day (free ~40 XP each).
2. Keep the streak alive — the day-4+ bonus (15–75) applies to every message that day.
3. Track 5 missions simultaneously (cap) — each is +75 on completion.
4. Teach ROCKET your usual questions' answers you already know — 25 XP each and it
   makes future answers faster (learned answers are served first).

## 3. Progression routes

- **Titles:** unlock automatically by Level / Streak / Questions / Facts / Topics /
  Badges / TotalXp (104 in the catalog). No purchase path — pure play.
- **Badges:** first question (First Chat), 10 questions (Quick Learner), first teach
  (Knowledge Seeker), 5 follow-ups (Follow-up Pro), 7/30-day streak (Streak Warrior),
  first mastery (Skill Explorer), level 5 (Expert Gamer), 3 games (Game Master).
- **Ranks:** Rookie (1–4) → Cadet (5–9) → Pilot (10–14) → … → Mythic (45+).

### 3.1 Known strong/weak plays

**Strong:** stacking teach + quest("teach") — one teaching event advances the quest
AND pays 25 XP. Same for game switching (+5 XP, "explore" quest progress, Game
Master badge progress, and games_today for the day's explore quest).

**Weak:** mystery-box spam at high collection completion (uniform draw over a small
locked pool can still brick you, though it cannot ever pay for an owned decoration —
the draw is over *locked* only, so there is no true "dupe" outcome).

## 4. Anti-spoiler note

None of the above concerns *game* content (the guides ROCKET gives are played
straight); this file only optimizes ROCKET's own meta-economy.
