//! Knowledge-artifact generator — the machine-readable half of ROCKET's
//! documentation package.
//!
//! `cargo test` runs [`generate_knowledge_artifacts`], which walks the live
//! code (XP table, quest pool, shop, title catalog, vibes, providers, game
//! registry, knowledge base) and writes:
//!
//! * `docs/rules.json`  — every system, formula, constant and catalog
//! * `docs/games.json`  — the game registry plus per-game knowledge stats
//!
//! Because the values are extracted from code at test time, these files can
//! never drift from the implementation. The hand-written companions
//! (`docs/knowledge.md`, `docs/faq.md`, `docs/strategy_notes.md`,
//! `docs/changelog.md`, `docs/schema.json`) describe design intent and
//! formulas; this test also asserts they exist.
//!
//! Values that cannot be extracted from a function body (XP award amounts,
//! dispatch order) are emitted with a `source` field pointing at the exact
//! function that implements them, so the claim is auditable.

use crate::memory::{DecorationRarity, TitleCondition};
use serde_json::{json, Map, Value};

const DOCS_DIR: &str = "docs";

fn docs_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DOCS_DIR)
        .join(name)
}

fn rarity_label(r: &DecorationRarity) -> &'static str {
    r.label()
}

fn title_condition(c: &TitleCondition) -> Value {
    match c {
        TitleCondition::Level(n) => json!({ "type": "level", "value": n }),
        TitleCondition::Streak(n) => json!({ "type": "streak_days", "value": n }),
        TitleCondition::Questions(n) => json!({ "type": "questions_answered", "value": n }),
        TitleCondition::Facts(n) => json!({ "type": "facts_taught", "value": n }),
        TitleCondition::Topics(n) => json!({ "type": "topics_mastered", "value": n }),
        TitleCondition::Badges(n) => json!({ "type": "badges_earned", "value": n }),
        TitleCondition::TotalXp(n) => json!({ "type": "total_xp", "value": n }),
    }
}

/// The live XP award rules. Each entry names the exact function that
/// implements it, so the doc is auditable against the code.
fn xp_awards() -> Value {
    json!({
        "conversation_per_user_message": {
            "amount": "10 + level bonus",
            "level_bonuses": { ">=10": 5, ">=25": 10, ">=40": 15 },
            "source": "memory::LevelingData::award_conversation_xp",
            "trigger": "every user chat message (main.rs App::add_message)"
        },
        "per_recorded_qa": {
            "amount": 10,
            "source": "memory::Memory::add_conversation",
            "trigger": "every answered question that is recorded to history",
            "note": "a single question can therefore award XP on two paths (add_message + add_conversation); intentional economy pacing, see docs/knowledge.md §Known quirks"
        },
        "teaching": {
            "amount": 25,
            "also": "awards the ⚡ Quick Learner path badge KnowledgeSeeker on first teach",
            "source": "memory::Memory::award_teaching_xp",
            "trigger": "teaching Rocket a new fact (main.rs)"
        },
        "correction": {
            "amount": "15 + level bonus",
            "level_bonuses": { ">=30": 10 },
            "source": "memory::LevelingData::award_correction_xp",
            "trigger": "a wrong: correction is processed (main.rs App::get_brain)"
        },
        "followup": {
            "amount": 15,
            "also": "FollowUpExpert badge after 5 follow-up questions",
            "source": "memory::Memory::award_followup_xp",
            "trigger": "asking more/another/next on a topic (main.rs)"
        },
        "topic_mastery": {
            "amount": 20,
            "note": "first time per topic only (repeat: 0 XP)",
            "source": "memory::Memory::award_mastery_xp"
        },
        "game_exploration": {
            "amount": 5,
            "also": "GameMaster badge once 3 distinct games played",
            "source": "memory::Memory::award_game_exploration_xp"
        },
        "streak_daily_bonus": {
            "amounts_by_streak": { "1-3": 5, "4-7": 15, "8-14": 30, "15-30": 50, "31+": 75 },
            "also": "StreakWarrior(7) badge + achievement at 7 days; StreakWarrior(30) at 30",
            "source": "memory::LevelingData::award_streak_xp",
            "trigger": "user message while streak > 1 day (main.rs add_message)"
        },
        "mission_completed": {
            "amount": 75,
            "source": "main.rs parse_mission_completion handler"
        },
        "untracked_win_celebrated": {
            "amount": 25,
            "source": "main.rs parse_mission_completion None branch"
        },
        "quest_completion": "quest.xp_reward (see daily_quests.pool)",
        "streak_rules": {
            "source": "memory::update_streak",
            "rules": [
                "streak increments when last activity was exactly yesterday",
                "streak resets to 1 after a fully missed calendar day",
                "same-day activity leaves the streak unchanged",
                "activity timestamps are stored as unix seconds in last_activity"
            ]
        }
    })
}

fn answer_pipeline() -> Value {
    json!({
        "dispatch_order": [
            { "step": 1, "handler": "ai.rs Brain::try_correction", "rule": "if the message starts with a correction prefix, it permanently fixes the last answered fact and returns immediately" },
            { "step": 2, "handler": "ai.rs Brain::follow_up", "rule": "exact match on more/another/next/another tip/next tip/tell me more/more tips fires another tip on the previous topic" },
            { "step": 3, "handler": "ai.rs Brain::smalltalk", "rule": "greetings, jokes, thanks, identity questions answered conversationally" },
            { "step": 4, "handler": "ai.rs Brain::try_captures", "rule": "personal facts are learned: my name is / my favorite game is / add game <name> / skill level / loadout context" },
            { "step": 5, "handler": "ai.rs Brain::recall_learned", "rule": "taught + corrected answers matched FIRST over the knowledge base" },
            { "step": 6, "handler": "ai.rs Brain::knowledge", "rule": "curated knowledge pack of the selected game, then universal item brain, then generic topic brain; if nothing matches Rocket asks to be taught" }
        ],
        "correction_prefixes": ["wrong:", "wrong ", "correction:", "correction ", "thats wrong"],
        "followup_triggers": ["more", "another", "next", "another tip", "next tip", "tell me more", "more tips"],
        "vibe_command_prefixes": ["be ", "act ", "go ", "switch to ", "turn on ", "vibe ", "set vibe to "],
        "correction_semantics": "the correction overwrites the memory slot for the last answered question; the corrected answer is then served at step 5, before built-in facts, permanently (persisted in rocket-brain.json)",
        "edge_cases": [
            "a normal question that merely contains the word 'wrong' is NOT treated as a correction (tested: normal_questions_containing_wrong_are_not_corrections)",
            "'wrong:' with no prior answered question explains how corrections work instead of failing (tested: correction_without_question_explains)",
            "corrections always beat learned answers at recall (tested: corrections_overwrite_last_answer)"
        ]
    })
}

fn online_ai() -> Value {
    let providers: Vec<Value> = crate::llm::Provider::ALL
        .iter()
        .map(|p| {
            json!({
                "id": format!("{:?}", p),
                "label": p.label(),
                "base_url": p.default_base_url(),
                "default_model": p.default_model(),
                "requires_key": p.requires_key(),
                "key_hint": p.key_hint(),
            })
        })
        .collect();
    json!({
        "mode_storage": "rocket-llm.json (mode, provider, api_key, model, custom_url)",
        "providers": providers,
        "request_shape": "OpenAI-compatible POST {base_url}/chat/completions",
        "system_prompt": "llm.rs system_prompt() bakes the user's learned facts, name, favorite game and corrections into the online system instructions",
        "fallback_rules": [
            "if online mode fails (no key, bad model, network error, HTTP error) Rocket answers from the local brain and says why",
            "Local mode never touches the network",
            "Ollama and LM Studio are local runtimes and require no key"
        ],
        "source": "llm.rs Settings::base_url/is_ready + main.rs llm error handling"
    })
}

fn game_detection() -> Value {
    let processes: Vec<Value> = crate::detect::GAME_PROCESSES
        .iter()
        .map(|(exe, game)| json!({ "process": exe, "game": game.get_name() }))
        .collect();
    json!({
        "enabled_by_default": true,
        "toggle_storage": "rocket_detect.json {\"enabled\": bool}",
        "platform": "Windows (tasklist CSV); other platforms return none and manual selection still works",
        "poll_interval_seconds": 6,
        "process_count": processes.len(),
        "processes": processes,
        "source": "detect.rs GAME_PROCESSES + main.rs subscription (6s timer)"
    })
}

fn leveling() -> Value {
    let reqs = crate::memory::LEVEL_XP_REQUIREMENTS;
    let mut by_level = Map::new();
    for (i, xp) in reqs.iter().enumerate() {
        by_level.insert((i + 1).to_string(), json!({ "xp_required": xp, "rank": crate::memory::level_title(i as u32 + 1) }));
    }
    json!({
        "max_level": reqs.len(),
        "xp_requirements_indexed_by_level": reqs.to_vec(),
        "rank_bands": [
            { "rank": "🌱 Rookie", "levels": "1-4" }, { "rank": "🛰️ Cadet", "levels": "5-9" },
            { "rank": "🚀 Pilot", "levels": "10-14" }, { "rank": "✈️ Ace Pilot", "levels": "15-19" },
            { "rank": "🛡️ Guardian", "levels": "20-24" }, { "rank": "⚔️ Veteran", "levels": "25-29" },
            { "rank": "🏆 Elite", "levels": "30-34" }, { "rank": "👑 Champion", "levels": "35-39" },
            { "rank": "🌟 Legend", "levels": "40-44" }, { "rank": "💫 Mythic", "levels": "45+" }
        ],
        "rank_by_level": by_level,
        "coins_rule": {
            "invariant": "coins always equal total_xp_earned",
            "note": "spending coins never reduces level or level progress; XP is never consumed",
            "source": "memory.rs LevelingData::add_experience + test coins mirror total xp"
        },
        "xp_awards": xp_awards(),
        "badges": [
            { "badge": "FirstConversation", "label": "💬 First Chat", "unlock": "1st question answered" },
            { "badge": "QuickLearner", "label": "⚡ Quick Learner", "unlock": "10 questions answered" },
            { "badge": "KnowledgeSeeker", "label": "🔍 Knowledge Seeker", "unlock": "first teaching event" },
            { "badge": "FollowUpExpert", "label": "🔁 Follow-up Pro", "unlock": "5 follow-up questions" },
            { "badge": "StreakWarrior(7|30)", "label": "🔥 Streak x7 / x30", "unlock": "7- or 30-day streak" },
            { "badge": "SkillExplorer", "label": "🧭 Skill Explorer", "unlock": "first mastered topic" },
            { "badge": "ExpertGamer", "label": "🎮 Expert Gamer", "unlock": "reach level 5" },
            { "badge": "GameMaster", "label": "👑 Game Master", "unlock": "play 3 distinct games" },
            { "badge": "RocketFanatic", "label": "🚀 ROCKET Fanatic", "unlock": "reach level 10 (achievement)" },
            { "badge": "CreativeBuilder", "label": "🏗️ Builder", "unlock": "reach level 25" },
            { "badge": "ConversationMaster(n)", "label": "🗣️ Chatter xN", "unlock": "conversation milestones" },
            { "badge": "EarlyAdopter", "label": "⭐ Early Adopter", "unlock": "special (reserved)" },
            { "badge": "HelpfulAssistant", "label": "🤝 Helper", "unlock": "special (reserved)" },
            { "badge": "MinecraftChampion", "label": "⛏️ MC Champion", "unlock": "special (reserved)" },
            { "badge": "RobloxExpert", "label": "🧱 Roblox Expert", "unlock": "special (reserved)" }
        ],
        "source": "memory.rs LEVEL_XP_REQUIREMENTS / level_title / award_* functions"
    })
}

fn daily_quests() -> Value {
    let pool: Vec<Value> = crate::memory::QUEST_POOL
        .iter()
        .map(|(kind, title, target, xp)| {
            json!({ "kind": kind, "title": title, "target": target, "xp_reward": xp })
        })
        .collect();
    json!({
        "rolled_per_day": 3,
        "roll_algorithm": "Fisher-Yates shuffle of the full pool, take first 3 (distinct by construction)",
        "reroll_rule": "rerolled when the stored quest_day differs from today's local date (YYYY-MM-DD); games_today is cleared on reroll",
        "progress_rule": "quest_progress(kind, n) advances every active quest of that kind; progress clamps at target; completion pays xp_reward immediately via add_experience",
        "kinds": ["ask", "chat", "teach", "explore", "item", "kb"],
        "kind_triggers": {
            "ask": "every user question (main.rs)",
            "chat": "conversations (main.rs)",
            "teach": "teaching a fact (main.rs)",
            "explore": "switching to a different game in one day",
            "item": "asking how to get any item/gear",
            "kb": "opening the Knowledge Base page"
        },
        "completion_line": "✅ Quest complete: <title> (+<xp> XP)",
        "pool": pool,
        "source": "memory.rs QUEST_POOL / ensure_daily_quests / quest_progress"
    })
}

fn missions() -> Value {
    json!({
        "model": "co-op missions: a title + game, done flag",
        "open_cap": 5,
        "cap_behavior": "adding a 6th open mission is refused with 'Mission board is full' (tested: mission_board_caps_at_five_open)",
        "duplicate_rule": "re-adding an active mission is refused ('already on our board')",
        "completion": {
            "tracked": "+75 XP + vibe celebration line with {mission} placeholder",
            "untracked": "saying you beat something not on the board: +25 XP + celebration",
            "detection": "memory::parse_mission_completion matches win phrases; complete_mission_by_text matches board entries"
        },
        "source": "memory.rs Mission / parse_mission_completion / complete_mission_by_text + main.rs handlers"
    })
}

fn xp_shop() -> Value {
    let prices = json!({
        "Basic": crate::memory::decoration_price(&DecorationRarity::Basic),
        "Common": crate::memory::decoration_price(&DecorationRarity::Common),
        "Rare": crate::memory::decoration_price(&DecorationRarity::Rare),
        "Epic": crate::memory::decoration_price(&DecorationRarity::Epic),
        "Legendary": crate::memory::decoration_price(&DecorationRarity::Legendary),
    });
    json!({
        "decoration_prices_by_rarity": prices,
        "mystery_box": {
            "price": crate::memory::MYSTERY_BOX_PRICE,
            "outcome_distribution": "uniform over currently LOCKED decorations: pick = rand(0..locked.len())",
            "exhaustion": "if every decoration is owned the purchase is refused and no coins are spent",
            "insufficient_coins": "refused with the exact cost and balance in the message; no partial spend",
            "source": "memory.rs LevelingData::buy_mystery_box"
        },
        "buy_decoration_rules": [
            "unlocking charges decoration_price(rarity) once",
            "already-owned decorations cannot be re-bought",
            "equipping requires unlocked state (tested: equip_decoration_only_when_unlocked)",
            "equipping a decoration unequips every other decoration in the same category (one per category)"
        ],
        "categories": ["Frame", "Background", "Icon", "Animation", "Title"],
        "source": "memory.rs decoration_price / buy_decoration / equip_decoration"
    })
}

fn knowledge_base() -> Value {
    let kb = crate::knowledge::KnowledgeBase::new();
    let games = crate::all_games();
    let mut total = 0usize;
    let mut with_packs = 0usize;
    let mut per_category: std::collections::BTreeMap<String, usize> = Default::default();
    let mut per_game: Vec<Value> = Vec::new();
    for g in &games {
        let entries = kb.entries_for_game(g);
        let n = entries.len();
        total += n;
        if n > 0 {
            with_packs += 1;
        }
        let mut cats: std::collections::BTreeMap<String, usize> = Default::default();
        for e in &entries {
            let label = e.category.label().to_string();
            *cats.entry(label.clone()).or_insert(0) += 1;
            *per_category.entry(label).or_insert(0) += 1;
        }
        per_game.push(json!({
            "game": g.get_name(),
            "entries": n,
            "by_category": cats,
        }));
    }
    json!({
        "categories": [
            "🎯 Strategy", "⚙️ Mechanics", "🗡️ Items & Gear",
            "👥 Characters & Roles", "📈 Progression", "📖 Glossary"
        ],
        "entry_shape": { "game": "GameType", "category": "Category", "title": "&str", "content": "&str", "tags": "&[&str]" },
        "search_rules": [
            "stopword-filtered token scoring (see knowledge.rs STOPWORDS)",
            "substring match counts; a token also matches when it is a prefix of the tag or the tag is a prefix of the token and BOTH sides are >= 4 chars (so 'netherite' hits 'nether', 'diamonds' hits 'diamond')",
            "search(game, q, limit) only returns that game's entries; search_any(q, limit) crosses games and is used in General Chat mode only",
            "a selected game NEVER returns another game's pack entries (tested in game_qa)"
        ],
        "answer_sources": {
            "Game": "curated guide from the currently selected game's pack (+ its first tag)",
            "CrossGame": "another game's pack — only in General Chat mode",
            "UniversalItem": "universal 'how to get X' brain for any item in any game"
        },
        "total_entries": total,
        "games_with_packs": with_packs,
        "per_game": per_game,
        "per_category_totals": per_category,
        "source": "knowledge.rs + live KnowledgeBase::entries_for_game"
    })
}

fn registry() -> Value {
    let mut games: Vec<Value> = crate::CATALOG
        .iter()
        .map(|g| {
            json!({
                "id": g.id.clone(),
                "name": g.name,
                "emoji": g.emoji,
                "genre": format!("{:?}", g.genre),
                "platforms": g.platforms.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "year": g.year,
            })
        })
        .collect();
    games.push(json!({ "id": "Generic", "name": "General Chat", "emoji": "💬", "genre": "Other", "platforms": [], "year": 0 }));
    json!({
        "game_count": games.len(),
        "note": "CATALOG is the single source of truth for the picker/filters/names; Generic (General Chat) is the pseudo-game added by all_games()",
        "games": games,
        "source": "main.rs games_table! macro + CATALOG"
    })
}

fn vibes() -> Value {
    let list: Vec<Value> = crate::vibe::VIBES
        .iter()
        .map(|v| {
            json!({
                "id": v.id,
                "label": v.label,
                "intro": v.intro,
                "game_switch_lines": v.game_switch.len(),
                "reaction_lines": v.reactions.len(),
                "tail_lines": v.tails.len(),
                "mission_created_lines": v.mission_created.len(),
                "mission_done_lines": v.mission_done.len(),
            })
        })
        .collect();
    json!({
        "vibes": list,
        "switch_commands": "see answer_pipeline.vibe_command_prefixes; unknown vibe falls back to buddy (tested)",
        "persistence": "current vibe id stored in rocket_memory.json (.vibe)",
        "placeholders": "{mission} in mission_done lines is replaced with 'title (game)'",
        "source": "vibe.rs VIBES / parse_vibe_command / fill"
    })
}

fn persistence() -> Value {
    json!([
        { "file": "rocket_memory.json", "owner": "memory.rs", "purpose": "profile, learned personal facts, XP/level/coins, badges, achievements, decorations, titles, quests, missions, vibe, streak" },
        { "file": "rocket-brain.json", "owner": "ai.rs Brain", "purpose": "taught answers, corrections, name, favorite game, custom games, topic hits, counters" },
        { "file": "rocket_chat_history.json", "owner": "main.rs", "purpose": "chat transcript persistence" },
        { "file": "rocket-llm.json", "owner": "llm.rs", "purpose": "online mode: mode, provider, api key, model, custom URL" },
        { "file": "rocket_theme.json", "owner": "theme.rs/main.rs", "purpose": "theme selection" },
        { "file": "rocket_detect.json", "owner": "detect.rs", "purpose": "auto-detection toggle" }
    ])
}

fn title_catalog() -> Value {
    let titles: Vec<Value> = crate::memory::PROFILE_TITLES
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "icon": t.icon,
                "name": t.name,
                "rarity": rarity_label(&t.rarity),
                "condition": title_condition(&t.condition),
                "requirement": t.requirement(),
            })
        })
        .collect();
    json!({
        "count": titles.len(),
        "titles": titles,
        "rules": [
            "titles unlock automatically via check_progress_rewards once the stat meets the condition",
            "equipping a title is profile-only cosmetics; exactly one title can be equipped",
            "requirement() renders the human-readable unlock text"
        ],
        "source": "memory.rs PROFILE_TITLES"
    })
}

fn achievement_catalog() -> Value {
    let achievements = crate::memory::get_default_achievements();
    json!({
        "count": achievements.len(),
        "achievements": achievements,
        "source": "memory.rs get_default_achievements"
    })
}

fn decoration_catalog() -> Value {
    let decorations = crate::memory::get_default_decorations();
    json!({
        "count": decorations.len(),
        "decorations": decorations,
        "source": "memory.rs get_default_decorations"
    })
}

fn meta() -> Value {
    json!({
        "app": "ROCKET — personal gaming AI companion",
        "generator": "cargo test docs_gen::generate_knowledge_artifacts (src/docs_gen.rs)",
        "provenance_legend": {
            "extracted": "value read from live code at test time — cannot drift",
            "source": "pointer to the implementing function — auditable"
        },
        "companion_documents": [
            "docs/knowledge.md — full human-readable reference",
            "docs/faq.md — common questions",
            "docs/strategy_notes.md — optimization math and play patterns",
            "docs/changelog.md — versioned history",
            "docs/schema.json — machine-readable schema for this file",
            "docs/games.json — game registry + knowledge stats",
            "rocket-qa-report.md — verbatim Q&A evidence from the live pipeline"
        ]
    })
}

#[test]
fn generate_knowledge_artifacts() {
    std::fs::create_dir_all(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOCS_DIR),
    )
    .expect("create docs dir");

    let rules = json!({
        "meta": meta(),
        "persistence": persistence(),
        "answer_pipeline": answer_pipeline(),
        "leveling": leveling(),
        "daily_quests": daily_quests(),
        "missions": missions(),
        "xp_shop": xp_shop(),
        "knowledge_base": knowledge_base(),
        "online_ai": online_ai(),
        "game_detection": game_detection(),
        "vibes": vibes(),
        "titles": title_catalog(),
        "achievements": achievement_catalog(),
        "decorations": decoration_catalog(),
        "codex": crate::codex::codex_schema_summary(),
    });
    let games = registry();

    // Sanity gates — the generator itself must never emit nonsense.
    let reqs = crate::memory::LEVEL_XP_REQUIREMENTS;
    assert_eq!(reqs.len(), 50, "XP table must stay 50 levels");
    for w in reqs.windows(2) {
        assert!(w[0] <= w[1], "XP requirements must be nondecreasing");
    }
    assert_eq!(crate::memory::QUEST_POOL.len(), 7, "quest pool size");
    assert!(crate::vibe::VIBES.len() >= 5, "five vibes minimum");
    assert!(!crate::llm::Provider::ALL.is_empty());
    assert!(
        crate::CATALOG.len() >= 160,
        "game registry should stay large, found {}",
        crate::CATALOG.len()
    );

    let rules_str = serde_json::to_string_pretty(&rules).expect("rules.json serialize");
    let games_str = serde_json::to_string_pretty(&games).expect("games.json serialize");
    // Round-trip: guarantees the emitted files parse.
    serde_json::from_str::<Value>(&rules_str).expect("rules.json must re-parse");
    serde_json::from_str::<Value>(&games_str).expect("games.json must re-parse");

    std::fs::write(docs_path("rules.json"), rules_str).expect("write rules.json");
    std::fs::write(docs_path("games.json"), games_str).expect("write games.json");

    // The hand-written half of the package must exist — removing any of
    // these fails the suite, keeping the docs honest.
    for required in [
        "knowledge.md",
        "codex.md",
        "faq.md",
        "strategy_notes.md",
        "changelog.md",
        "schema.json",
        "README.md",
    ] {
        assert!(
            docs_path(required).exists(),
            "docs/{required} is part of the knowledge package — restore it"
        );
    }
}
