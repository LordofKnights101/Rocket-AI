//! The Order of the Eternal Codex — the medieval-scholarly progression layer.
//!
//! Sits ON TOP of the existing XP/level economy (memory.rs) without replacing
//! it: the numeric backbone stays, the Codex adds the Order identity — ranks,
//! per-realm Lore Points, a dynamic quest board generated from the knowledge
//! base itself, a chronicle of scholarly deeds, an auditable action log, and
//! Rocket's desk (the practical-assistant side channel).
//!
//! Integrity rules (spec §5):
//! * No game rule is ever invented here — quests are derived from the live
//!   `KnowledgeBase` and every quest names the grimoire sections it exercises.
//! * All state is durable, versioned, human-inspectable JSON
//!   (`codex_character.json`), with a markdown chronicle export.
//! * Every state-modifying desk action is logged (`actions`).

use crate::knowledge::KnowledgeBase;
use crate::GameType;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const CODEX_FILE: &str = "codex_character.json";
pub const CHRONICLE_FILE: &str = "codex_chronicle.md";
pub const CODEX_VERSION: u32 = 1;

/// Open quests the board will hold before new ones stop being pinned.
pub const BOARD_CAPACITY: usize = 6;

// ---------------------------------------------------------------------------
// Ranks of the Order
// ---------------------------------------------------------------------------

/// The seven ranks of the Order, earned by character level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderRank {
    NoviceScribe,
    InkApprentice,
    JourneymanScribe,
    AdeptScholar,
    Loremaster,
    MasterArchivist,
    GrandArchivist,
}

impl OrderRank {
    pub const ALL: [OrderRank; 7] = [
        OrderRank::NoviceScribe,
        OrderRank::InkApprentice,
        OrderRank::JourneymanScribe,
        OrderRank::AdeptScholar,
        OrderRank::Loremaster,
        OrderRank::MasterArchivist,
        OrderRank::GrandArchivist,
    ];

    /// Level bands: 1-4, 5-9, 10-14, 15-24, 25-34, 35-44, 45+.
    pub fn from_level(level: u32) -> OrderRank {
        match level {
            0..=4 => OrderRank::NoviceScribe,
            5..=9 => OrderRank::InkApprentice,
            10..=14 => OrderRank::JourneymanScribe,
            15..=24 => OrderRank::AdeptScholar,
            25..=34 => OrderRank::Loremaster,
            35..=44 => OrderRank::MasterArchivist,
            _ => OrderRank::GrandArchivist,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            OrderRank::NoviceScribe => "Novice Scribe",
            OrderRank::InkApprentice => "Ink Apprentice",
            OrderRank::JourneymanScribe => "Journeyman Scribe",
            OrderRank::AdeptScholar => "Adept Scholar",
            OrderRank::Loremaster => "Loremaster",
            OrderRank::MasterArchivist => "Master Archivist",
            OrderRank::GrandArchivist => "Grand Archivist",
        }
    }

    pub fn sigil(&self) -> &'static str {
        match self {
            OrderRank::NoviceScribe => "🪶",
            OrderRank::InkApprentice => "🖋️",
            OrderRank::JourneymanScribe => "📜",
            OrderRank::AdeptScholar => "🕯️",
            OrderRank::Loremaster => "📖",
            OrderRank::MasterArchivist => "🗝️",
            OrderRank::GrandArchivist => "👑",
        }
    }

    /// Minimum level for this rank (used in requirement text).
    pub fn min_level(&self) -> u32 {
        match self {
            OrderRank::NoviceScribe => 1,
            OrderRank::InkApprentice => 5,
            OrderRank::JourneymanScribe => 10,
            OrderRank::AdeptScholar => 15,
            OrderRank::Loremaster => 25,
            OrderRank::MasterArchivist => 35,
            OrderRank::GrandArchivist => 45,
        }
    }

    pub fn index(&self) -> usize {
        OrderRank::ALL.iter().position(|r| r == self).unwrap_or(0)
    }
}

/// Which quest tiers a rank unlocks. Machine-checked by `ensure_board`.
pub fn quest_tier_unlocked(rank: OrderRank, kind: QuestKind) -> bool {
    let idx = rank.index();
    match kind {
        QuestKind::LoreMastery => true,
        QuestKind::EdgeCaseExpedition => idx >= 1,
        QuestKind::ComparativeScholarship => idx >= 2,
        QuestKind::Restoration => idx >= 3,
        QuestKind::TrialByOrdeal => idx >= 4,
    }
}

/// Desk quest-authoring requires a Journeyman's seal.
pub fn desk_authoring_unlocked(rank: OrderRank) -> bool {
    rank.index() >= OrderRank::JourneymanScribe.index()
}

// ---------------------------------------------------------------------------
// Realm lore — per Game-Realm mastery
// ---------------------------------------------------------------------------

/// Mastery tiers of a realm by accumulated Lore Points.
pub const REALM_TIERS: &[(&str, u32)] = &[
    ("Uncharted", 0),
    ("Charted", 10),
    ("Studied", 25),
    ("Examined", 50),
    ("Mastered", 100),
    ("Sanctified", 200),
];

pub fn tier_for(lore: u32) -> (&'static str, u32) {
    // Highest tier whose threshold is met, plus the next threshold (0 = none).
    let mut current = 0;
    let mut next = REALM_TIERS[0].1;
    for (i, (_, th)) in REALM_TIERS.iter().enumerate() {
        if lore >= *th {
            current = i;
            next = REALM_TIERS.get(i + 1).map(|(_, t)| *t).unwrap_or(0);
        }
    }
    (REALM_TIERS[current].0, next)
}

/// Scholarship record for one Game-Realm.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealmLore {
    pub questions_answered: u32,
    pub lore_points: u32,
    /// Category label -> times a curated answer touched it.
    pub categories_touched: BTreeMap<String, u32>,
    /// Entry titles surfaced by answers (most recent last, capped).
    pub entries_seen: Vec<String>,
}

impl RealmLore {
    const SEEN_CAP: usize = 64;

    fn touch_entry(&mut self, title: &str) -> bool {
        self.entries_seen.push(title.to_string());
        if self.entries_seen.len() > Self::SEEN_CAP {
            self.entries_seen.remove(0);
        }
        // "seen" means it appeared in the current cap window.
        true
    }

    fn has_seen(&self, title: &str) -> bool {
        self.entries_seen.iter().any(|t| t == title)
    }
}

// ---------------------------------------------------------------------------
// Quests
// ---------------------------------------------------------------------------

/// The five quest disciplines of the Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestKind {
    /// Answer a sequence of questions in one realm's grimoire.
    LoreMastery,
    /// Deliberately probe rare/never-queried sections of a grimoire.
    EdgeCaseExpedition,
    /// Relate two grimoires through the same discipline.
    ComparativeScholarship,
    /// Identify and fill gaps in a thin grimoire by teaching.
    Restoration,
    /// Constrained trial: a run of grounded answers without faltering.
    TrialByOrdeal,
}

impl QuestKind {
    pub fn label(&self) -> &'static str {
        match self {
            QuestKind::LoreMastery => "📖 Lore Mastery",
            QuestKind::EdgeCaseExpedition => "🧭 Edge-Case Expedition",
            QuestKind::ComparativeScholarship => "⚖️ Comparative Scholarship",
            QuestKind::Restoration => "🔧 Restoration",
            QuestKind::TrialByOrdeal => "⚔️ Trial by Ordeal",
        }
    }

    pub fn id_prefix(&self) -> &'static str {
        match self {
            QuestKind::LoreMastery => "lore",
            QuestKind::EdgeCaseExpedition => "edge",
            QuestKind::ComparativeScholarship => "compare",
            QuestKind::Restoration => "restore",
            QuestKind::TrialByOrdeal => "ordeal",
        }
    }
}

/// One quest pinned to the board. Success criteria are pure counters driven
/// by real events (answers, teaches) — nothing is self-reported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexQuest {
    pub id: String,
    pub kind: QuestKind,
    pub title: String,
    /// Primary realm (display name). None = the Order at large.
    pub game: Option<String>,
    /// Second realm for Comparative Scholarship.
    pub game2: Option<String>,
    /// Grimoire discipline exercised (category label), if any.
    pub category: Option<String>,
    pub target: u32,
    pub progress: u32,
    pub xp_reward: u32,
    pub lore_reward: u32,
    /// Rank index required to be PINNED this quest.
    pub rank_required: usize,
    /// Which grimoire sections this quest exercises (human-readable).
    pub grimoire_sections: Vec<String>,
    pub sealed: bool,
    /// Per-realm progress for Comparative quests.
    #[serde(default)]
    pub realm_progress: BTreeMap<String, u32>,
    pub created: String,
}

impl CodexQuest {
    pub fn progress_line(&self) -> String {
        if self.kind == QuestKind::ComparativeScholarship {
            let g1 = self.game.clone().unwrap_or_default();
            let g2 = self.game2.clone().unwrap_or_default();
            format!(
                "  ⚖️ {}: {}/{} · {}: {}/{}",
                g1,
                self.realm_progress.get(&g1).copied().unwrap_or(0),
                1,
                g2,
                self.realm_progress.get(&g2).copied().unwrap_or(0),
                1
            )
        } else {
            format!("  Progress: {}/{}", self.progress, self.target)
        }
    }
}

// ---------------------------------------------------------------------------
// Chronicle & audit log
// ---------------------------------------------------------------------------

/// A notable scholarly deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleEntry {
    pub date: String,
    pub kind: String,
    pub text: String,
}

/// An auditable state modification. `actor` is "rocket" (desk action) or
/// "order" (system-driven progression).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub detail: String,
}

/// One side-chat message on Rocket's desk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeskMessage {
    pub role: String, // "user" | "rocket"
    pub content: String,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// The Codex (character state)
// ---------------------------------------------------------------------------

/// Outcome of recording one question — what the main loop should apply.
#[derive(Debug, Default)]
pub struct RealmOutcome {
    /// Codex XP bonus to add on top of the standard economy.
    pub xp_bonus: u32,
    /// XP from quests sealed by this event (add on top).
    pub quest_xp: u32,
    /// Toast lines (realm tiers, sealed quests).
    pub lines: Vec<String>,
}

/// The persistent Codex character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codex {
    pub version: u32,
    /// Realm name (display name) -> scholarship record.
    pub realms: BTreeMap<String, RealmLore>,
    pub board: Vec<CodexQuest>,
    pub chronicle: Vec<ChronicleEntry>,
    pub actions: Vec<ActionEntry>,
    /// Consecutive knowledge-grounded answers (Trial by Ordeal).
    pub ordeal_streak: u32,
    pub desk_history: Vec<DeskMessage>,
    /// Permission setting: Rocket may append notes in the main channel.
    pub desk_interjections: bool,
    #[serde(default)]
    pub quest_seq: u64,
    /// 🌳 CREATE TREE — the quest-tree atelier (multi-instance, tabbed).
    #[serde(default)]
    pub trees: Vec<crate::codex_tree::QuestTree>,
    #[serde(default)]
    pub tree_seq: u64,
    /// The tree currently focused in the Scriptorium atelier (if any).
    #[serde(default)]
    pub focused_tree: Option<String>,
    /// 🌳 An in-progress conversational commission (Create Tree button flow).
    #[serde(default)]
    pub commission_draft: Option<crate::codex_tree::CommissionDraft>,
}

impl Default for Codex {
    fn default() -> Self {
        Self {
            version: CODEX_VERSION,
            realms: BTreeMap::new(),
            board: Vec::new(),
            chronicle: Vec::new(),
            actions: Vec::new(),
            ordeal_streak: 0,
            desk_history: Vec::new(),
            desk_interjections: false,
            quest_seq: 0,
            trees: Vec::new(),
            tree_seq: 0,
            focused_tree: None,
            commission_draft: None,
        }
    }
}

const CHRONICLE_CAP: usize = 60;
const ACTIONS_CAP: usize = 200;
const DESK_CAP: usize = 120;

fn base_path(name: &str) -> std::path::PathBuf {
    Codex::base_dir().join(name)
}

impl Codex {
    // -- persistence --------------------------------------------------------

    /// Where the Codex books live. Tests set `ROCKET_CODEX_DIR` so unit tests
    /// never touch the user's real files in the working directory.
    pub fn base_dir() -> std::path::PathBuf {
        std::env::var_os("ROCKET_CODEX_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    pub fn load() -> Self {
        match std::fs::read(base_path(CODEX_FILE)) {
            Ok(bytes) => serde_json::from_slice::<Codex>(&bytes)
                .map(|mut c| {
                    c.version = CODEX_VERSION;
                    c
                })
                .unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Ok(json) = serde_json::to_string_pretty(self) else {
            return;
        };
        let _ = std::fs::write(base_path(CODEX_FILE), json);
    }

    // -- books of record ------------------------------------------------------

    fn chronicle(&mut self, kind: &str, text: String) {
        self.chronicle.push(ChronicleEntry {
            date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            kind: kind.to_string(),
            text,
        });
        if self.chronicle.len() > CHRONICLE_CAP {
            self.chronicle.drain(0..self.chronicle.len() - CHRONICLE_CAP);
        }
    }

    pub fn log_action(&mut self, actor: &str, action: &str, detail: &str) {
        self.actions.push(ActionEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            actor: actor.to_string(),
            action: action.to_string(),
            detail: detail.to_string(),
        });
        if self.actions.len() > ACTIONS_CAP {
            self.actions.drain(0..self.actions.len() - ACTIONS_CAP);
        }
    }

    // -- realm scholarship ------------------------------------------------------

    /// Record one answered question against its realm.
    ///
    /// `tag` is the knowledge-base category/tag the curated answer came from
    /// (None for non-grounded answers). `kb_sourced` marks answers that came
    /// from a grimoire — the only kind that feeds the ordeal streak and lore.
    ///
    /// XP model (exact, mirrored into docs/rules.json by the generator):
    /// * novelty: first time this realm touches a tag  -> +5 XP, +2 lore
    /// * depth:   question of 6+ words                 -> +3 XP
    /// * every grounded answer: +1 lore (included above), +1 ordeal progress
    pub fn record_realm_question(
        &mut self,
        game: &GameType,
        tag: Option<&str>,
        question: &str,
        kb_sourced: bool,
    ) -> RealmOutcome {
        let mut out = RealmOutcome::default();
        if !kb_sourced {
            if self.ordeal_streak != 0 {
                self.ordeal_streak = 0;
                // An ordeal broken — the Order notes it without punishment.
                self.chronicle("ordeal", "A Trial by Ordeal run was broken; the ink waits patiently.".into());
            }
            return out;
        }

        let realm_name = game.get_name().to_string();
        let tag = tag.filter(|t| !t.is_empty());
        let depth = question.split_whitespace().count() >= 6;

        // All realm mutation happens inside this block so the borrow ends
        // before the chronicle/log calls below.
        let (novelty, new_lore, old_tier, new_tier, seen_before) = {
            let realm = self.realms.entry(realm_name.clone()).or_default();
            realm.questions_answered += 1;

            // Novelty: first touch of this tag in this realm.
            let novelty = match tag {
                Some(t) => realm.categories_touched.get(t).copied().unwrap_or(0) == 0,
                None => false,
            };
            if let Some(t) = tag {
                *realm.categories_touched.entry(t.to_string()).or_insert(0) += 1;
            }

            let lore_gain: u32 = if novelty { 2 } else { 1 };
            let before = realm.lore_points;
            realm.lore_points += lore_gain;

            let (old_tier, _) = tier_for(before);
            let (new_tier, _) = tier_for(realm.lore_points);

            // Entry seen-mark (for Edge-Case Expeditions).
            let seen_before = match tag {
                Some(t) => {
                    let seen = realm.has_seen(t);
                    realm.touch_entry(t);
                    seen
                }
                None => true,
            };
            (novelty, realm.lore_points, old_tier, new_tier, seen_before)
        };

        out.xp_bonus += if novelty { 5 } else { 0 } + if depth { 3 } else { 0 };

        if new_tier != old_tier {
            out.lines.push(format!(
                "📖 The {} grimoire rises to **{}** in your Codex.",
                realm_name, new_tier
            ));
            self.chronicle(
                "realm",
                format!("The {} grimoire rose to {} ({} lore).", realm_name, new_tier, new_lore),
            );
        }

        // Ordeal + quest advancement.
        self.ordeal_streak += 1;
        let ordeal = self.ordeal_streak;
        let game_for_quests = realm_name.clone();
        for quest in self.board.iter_mut().filter(|q| !q.sealed) {
            match quest.kind {
                QuestKind::LoreMastery => {
                    let game_ok = quest.game.as_deref() == Some(game_for_quests.as_str());
                    let cat_ok = quest
                        .category
                        .as_deref()
                        .map(|c| tag == Some(c))
                        .unwrap_or(true);
                    if game_ok && cat_ok && quest.progress < quest.target {
                        quest.progress += 1;
                    }
                }
                QuestKind::EdgeCaseExpedition => {
                    let game_ok = quest.game.as_deref() == Some(game_for_quests.as_str());
                    let cat_ok = quest
                        .category
                        .as_deref()
                        .map(|c| tag == Some(c))
                        .unwrap_or(false);
                    // Progress only on entries never surfaced before.
                    if game_ok && cat_ok && !seen_before && quest.progress < quest.target {
                        quest.progress += 1;
                    }
                }
                QuestKind::ComparativeScholarship => {
                    if quest.category.is_some() && quest.category.as_deref() != tag {
                        continue;
                    }
                    if let Some(g) = quest.game.as_deref().filter(|g| *g == game_for_quests) {
                        let e = quest.realm_progress.entry(g.to_string()).or_insert(0);
                        if *e < 1 {
                            *e += 1;
                        }
                    }
                    if let Some(g) = quest.game2.as_deref().filter(|g| *g == game_for_quests) {
                        let e = quest.realm_progress.entry(g.to_string()).or_insert(0);
                        if *e < 1 {
                            *e += 1;
                        }
                    }
                }
                QuestKind::Restoration => {}
                QuestKind::TrialByOrdeal => {
                    quest.progress = ordeal.min(quest.target);
                }
            }
        }
        self.seal_completed(&mut out);
        out
    }

    /// Record a teaching event (Restoration quests).
    pub fn record_teaching(&mut self, game: &GameType) -> Vec<String> {
        let realm_name = game.get_name().to_string();
        for quest in self.board.iter_mut().filter(|q| !q.sealed) {
            if quest.kind != QuestKind::Restoration {
                continue;
            }
            let game_ok = quest
                .game
                .as_deref()
                .map(|g| g == realm_name)
                .unwrap_or(true);
            if game_ok && quest.progress < quest.target {
                quest.progress += 1;
            }
        }
        let mut out = RealmOutcome::default();
        self.seal_completed(&mut out);
        out.lines
    }

    /// Seal any quest whose success criteria are now met.
    fn seal_completed(&mut self, out: &mut RealmOutcome) {
        // Phase 1: seal quests and collect their rewards (board borrow only).
        struct Sealed {
            id: String,
            kind: QuestKind,
            title: String,
            xp: u32,
            lore: u32,
            game: Option<String>,
        }
        let mut sealed: Vec<Sealed> = Vec::new();
        for quest in self.board.iter_mut() {
            if quest.sealed {
                continue;
            }
            let done = match quest.kind {
                QuestKind::ComparativeScholarship => {
                    let g1 = quest.game.clone().unwrap_or_default();
                    let g2 = quest.game2.clone().unwrap_or_default();
                    quest.realm_progress.get(&g1).copied().unwrap_or(0) >= 1
                        && quest.realm_progress.get(&g2).copied().unwrap_or(0) >= 1
                }
                _ => quest.progress >= quest.target,
            };
            if !done {
                continue;
            }
            quest.sealed = true;
            out.quest_xp += quest.xp_reward;
            sealed.push(Sealed {
                id: quest.id.clone(),
                kind: quest.kind,
                title: quest.title.clone(),
                xp: quest.xp_reward,
                lore: quest.lore_reward,
                game: quest.game.clone(),
            });
        }
        // Phase 2: books of record (chronicle, audit log, realm lore).
        for s in sealed {
            if let Some(gname) = s.game.as_deref() {
                if let Some(realm) = self.realms.get_mut(gname) {
                    realm.lore_points += s.lore;
                }
            }
            let line = format!(
                "✒️ Quest sealed: {} — {} (+{} pages, +{} lore)",
                s.kind.label(),
                s.title,
                s.xp,
                s.lore
            );
            out.lines.push(line.clone());
            self.chronicle("quest", line);
            self.log_action("order", "quest_sealed", &s.id);
        }
    }

    // -- the quest board ------------------------------------------------------

    /// Pin fresh quests until the board is full. Quests are derived from the
    /// live knowledge base (never invented) and gated by Order rank.
    /// Returns lines describing newly pinned quests.
    pub fn ensure_board(&mut self, kb: &KnowledgeBase, level: u32) -> Vec<String> {
        let mut lines = Vec::new();
        let rank = OrderRank::from_level(level);
        let mut guard = 0;
        while self.open_count() < BOARD_CAPACITY && guard < 24 {
            guard += 1;
            let Some(quest) = self.next_candidate(kb, rank) else {
                break;
            };
            lines.push(format!(
                "📌 Pinned to the board: {} — {} (+{} pages, +{} lore)",
                quest.kind.label(),
                quest.title,
                quest.xp_reward,
                quest.lore_reward
            ));
            self.log_action("order", "quest_pinned", &quest.id);
            self.board.push(quest);
        }
        lines
    }

    fn open_count(&self) -> usize {
        self.board.iter().filter(|q| !q.sealed).count()
    }

    /// Build one new quest, rotating disciplines and preferring underexplored
    /// material. None when the rank unlocks nothing new or the KB is dry.
    fn next_candidate(&mut self, kb: &KnowledgeBase, rank: OrderRank) -> Option<CodexQuest> {
        let kinds: [QuestKind; 5] = [
            QuestKind::LoreMastery,
            QuestKind::EdgeCaseExpedition,
            QuestKind::ComparativeScholarship,
            QuestKind::Restoration,
            QuestKind::TrialByOrdeal,
        ];
        for kind in kinds {
            if !quest_tier_unlocked(rank, kind) {
                continue;
            }
            // Skip if the board already holds an unsealed quest of this kind.
            if self
                .board
                .iter()
                .any(|q| !q.sealed && q.kind == kind)
            {
                continue;
            }
            if let Some(q) = self.build_quest(kind, kb) {
                self.quest_seq += 1;
                let id = format!("{}-{:04}", kind.id_prefix(), self.quest_seq);
                return Some(CodexQuest {
                    id,
                    sealed: false,
                    progress: 0,
                    realm_progress: BTreeMap::new(),
                    created: Local::now().format("%Y-%m-%d").to_string(),
                    ..q
                });
            }
        }
        None
    }

    /// Construct the shape (not identity) of a quest from live KB data.
    fn build_quest(&self, kind: QuestKind, kb: &KnowledgeBase) -> Option<CodexQuest> {
        let games = crate::all_games();
        // Underexplored first: realms with the fewest recorded questions that
        // still have curated entries.
        let mut ranked: Vec<(GameType, usize)> = games
            .iter()
            .map(|g| {
                let n = kb.entries_for_game(g).len();
                let asked = self
                    .realms
                    .get(g.get_name())
                    .map(|r| r.questions_answered)
                    .unwrap_or(0) as usize;
                (g.clone(), n, asked)
            })
            .filter(|(_, n, _)| *n > 0)
            .map(|(g, n, asked)| (g, n + asked))
            .collect();
        ranked.sort_by_key(|(_, score)| *score);

        match kind {
            QuestKind::LoreMastery => {
                let (g, _) = ranked.first()?;
                let category = common_category(kb, g)?;
                Some(CodexQuest {
                    kind,
                    title: format!("Transcribe the {} lore of {}", category, g.get_name()),
                    game: Some(g.get_name().to_string()),
                    game2: None,
                    category: Some(category.to_string()),
                    target: 3,
                    xp_reward: 60,
                    lore_reward: 5,
                    rank_required: OrderRank::NoviceScribe.index(),
                    grimoire_sections: vec![format!("{} — {} grimoire", category, g.get_name())],
                    ..quest_shell()
                })
            }
            QuestKind::EdgeCaseExpedition => {
                // Pick the game whose category coverage is thinnest but real.
                let (g, _) = ranked.iter().rev().find(|(_, n)| *n >= 4)?;
                let category = rarest_category(kb, g)?;
                Some(CodexQuest {
                    kind,
                    title: format!(
                        "Expedition into the uncharted {} of {}",
                        category,
                        g.get_name()
                    ),
                    game: Some(g.get_name().to_string()),
                    game2: None,
                    category: Some(category.to_string()),
                    target: 2,
                    xp_reward: 80,
                    lore_reward: 8,
                    rank_required: OrderRank::InkApprentice.index(),
                    grimoire_sections: vec![format!("{} — {} grimoire", category, g.get_name())],
                    ..quest_shell()
                })
            }
            QuestKind::ComparativeScholarship => {
                // Two realms that share a discipline.
                let cats = ["🎯 Strategy", "⚙️ Mechanics", "🗡️ Items & Gear", "📈 Progression"];
                for cat in cats {
                    let have: Vec<&GameType> = games
                        .iter()
                        .filter(|g| {
                            kb.entries_for_game(g)
                                .iter()
                                .any(|e| e.category.label() == cat)
                        })
                        .collect();
                    if have.len() >= 2 {
                        let g1 = have[0];
                        let g2 = have[1];
                        return Some(CodexQuest {
                            kind,
                            title: format!(
                                "Compare the {} of {} and {}",
                                cat.trim_start_matches(|c: char| !c.is_ascii_alphabetic()),
                                g1.get_name(),
                                g2.get_name()
                            ),
                            game: Some(g1.get_name().to_string()),
                            game2: Some(g2.get_name().to_string()),
                            category: Some(cat.to_string()),
                            target: 2,
                            xp_reward: 90,
                            lore_reward: 10,
                            rank_required: OrderRank::JourneymanScribe.index(),
                            grimoire_sections: vec![
                                format!("{} — {} grimoire", cat, g1.get_name()),
                                format!("{} — {} grimoire", cat, g2.get_name()),
                            ],
                            ..quest_shell()
                        });
                    }
                }
                None
            }
            QuestKind::Restoration => {
                // Thin grimoires need restoration: <= 2 entries (or none).
                let thin = games
                    .iter()
                    .filter(|g| kb.entries_for_game(g).len() <= 2 && **g != GameType::Generic)
                    .min_by_key(|g| kb.entries_for_game(g).len())?;
                Some(CodexQuest {
                    kind,
                    title: format!("Restore the {} grimoire — teach what is missing", thin.get_name()),
                    game: Some(thin.get_name().to_string()),
                    game2: None,
                    category: None,
                    target: 2,
                    xp_reward: 100,
                    lore_reward: 12,
                    rank_required: OrderRank::AdeptScholar.index(),
                    grimoire_sections: vec![format!("{} grimoire (gaps)", thin.get_name())],
                    ..quest_shell()
                })
            }
            QuestKind::TrialByOrdeal => Some(CodexQuest {
                kind,
                title: "Ordeal of the Grounded Mind — 3 curated answers, no faltering".to_string(),
                game: None,
                game2: None,
                category: None,
                target: 3,
                xp_reward: 120,
                lore_reward: 15,
                rank_required: OrderRank::Loremaster.index(),
                grimoire_sections: vec!["any grimoire (consecutive grounded answers)".to_string()],
                ..quest_shell()
            }),
        }
    }

    // -- Rocket's desk ---------------------------------------------------------

    /// Handle one desk message. Appends both sides to the desk history,
    /// logs state-modifying actions, returns Rocket's reply.
    pub fn desk_reply(&mut self, input: &str, kb: &KnowledgeBase, level: u32) -> String {
        let text = input.trim().to_string();
        if text.is_empty() {
            return String::new();
        }
        self.desk_push("user", &text);
        let lower = text.to_lowercase();

        // 🌳 An open commission captures the conversation until it is done or
        // dropped — the Create Tree button's guided interrogation.
        if self.commission_draft.is_some() {
            const COMMAND_PREFIXES: &[&str] = &[
                "create tree", "trees", "open tree", "restore tree", "minimize tree",
                "rename tree", "duplicate tree", "archive tree", "destroy tree",
                "expand tree", "rebalance tree", "note tree", "validate trees",
                "repair tree", "tree report", "outline tree", "reorder tree",
                "confirm saved", "save the trees", "quest report", "set me a trial",
                "help", "board", "audit", "gaps", "compare", "comparative",
                "where stand i", "character report", "report", "chronicle", "log",
                "interjections", "clear the desk", "skip the rite", "restart the rite",
                "remind me", "show me again",
            ];
            let lower_trim = text.to_lowercase();
            if lower_trim == "cancel"
                || lower_trim == "nevermind"
                || lower_trim == "never mind"
                || lower_trim.starts_with("cancel the commission")
            {
                self.commission_draft = None;
                let reply: String = "🌳 The commission rests unfinished — say `create tree` whenever you wish to begin anew.".to_string();
                self.desk_push("rocket", &reply);
                self.save();
                return reply;
            }
            if COMMAND_PREFIXES.iter().any(|p| lower_trim.starts_with(p)) {
                self.commission_draft = None;
                self.desk_push(
                    "rocket",
                    "🌳 The commission yields to your command — say `create tree` to resume it.",
                );
                // Fall through to the normal dispatch below.
            } else if let Some(reply) = self.commission_step(&text, level, kb) {
                self.desk_push("rocket", &reply);
                self.save();
                return reply;
            }
        }

        let reply = if ["help", "commands", "what can you do"].contains(&lower.as_str()) {
            self.desk_help()
        } else if ["board", "quest report", "summon the board", "quests"].contains(&lower.as_str()) {
            self.desk_board_report()
        } else if lower.starts_with("set me a trial")
            || lower.starts_with("generate challenge")
            || lower.starts_with("generate quest")
            || lower.starts_with("new quest")
        {
            self.desk_generate(kb, level, &text)
        } else if lower.starts_with("audit") {
            self.desk_audit(kb, &text, "audit")
        } else if lower.starts_with("gaps") {
            self.desk_audit(kb, &text, "gaps")
        } else if lower.starts_with("compare") || lower.starts_with("comparative") {
            self.desk_compare(kb, &text)
        } else if ["where stand i", "character report", "report", "status"].contains(&lower.as_str()) {
            self.desk_character_report(level)
        } else if lower.contains("chronicle") {
            self.desk_export_chronicle(level)
        } else if lower == "log" || lower == "action log" {
            self.desk_action_log()
        } else if lower.starts_with("interjections") {
            self.desk_toggle_interjections(&text)
        } else if lower == "clear the desk" || lower == "clear desk" {
            self.desk_history.clear();
            "🪣 The desk is cleared. A clean page awaits.".to_string()
        } else if lower.starts_with("create tree")
            || lower.starts_with("trees")
            || lower.starts_with("open tree")
            || lower.starts_with("restore tree")
            || lower.starts_with("minimize tree")
            || lower.starts_with("rename tree")
            || lower.starts_with("duplicate tree")
            || lower.starts_with("archive tree")
            || lower.starts_with("destroy tree")
            || lower.starts_with("expand tree")
            || lower.starts_with("rebalance tree")
            || lower.starts_with("note tree")
            || lower.starts_with("validate trees")
            || lower.starts_with("repair tree")
            || lower.starts_with("tree report")
            || lower.starts_with("outline tree")
            || lower.starts_with("reorder tree")
            || lower.starts_with("confirm saved")
            || lower.starts_with("save the trees")
        {
            self.desk_tree_command(&text, level, kb)
        } else {
            // Conversational fallback — still the Order's voice.
            format!(
                "🕯️ The desk-lamp flickers in acknowledgement. I am Rocket — your familiar and research assistant of the Order. Name a task: *{}*\n\n\
                 Try: `quest report` · `set me a trial` · `audit <game>` · `gaps <game>` · `compare <a> and <b>` · `where stand I` · `chronicle` · `log` · `interjections on/off` · `help`",
                text
            )
        };

        self.desk_push("rocket", &reply);
        self.save();
        reply
    }

    /// Re-explain any taught topic on demand ("remind me how CREATE TREE
    /// works"). Returns false when the text isn't a reminder, so the desk
    /// falls through to normal handling. The full guidance sheet lives on
    /// the 📖 Info page.
    pub fn re_explain(&mut self, text: &str) -> bool {
        if let Some(lesson) = crate::initiation::re_explain(text) {
            self.desk_push("rocket", &lesson);
            self.save();
            true
        } else {
            false
        }
    }

    fn desk_push(&mut self, role: &str, content: &str) {
        self.desk_history.push(DeskMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Local::now().format("%H:%M").to_string(),
        });
        if self.desk_history.len() > DESK_CAP {
            self.desk_history.drain(0..self.desk_history.len() - DESK_CAP);
        }
    }

    // -- 🌳 CREATE TREE — the quest-tree atelier -----------------------------

    /// Route one grounded answer through the atelier (all living trees).
    pub fn tree_answer_event(
        &mut self,
        game: &GameType,
        tag: Option<&str>,
        grounded: bool,
    ) -> crate::codex_tree::TreeOutcome {
        crate::codex_tree::record_answer(&mut self.trees, game, tag, grounded)
    }

    /// Route one teaching event through the atelier.
    pub fn tree_teach_event(&mut self, game: &GameType) -> crate::codex_tree::TreeOutcome {
        crate::codex_tree::record_teach(&mut self.trees, game)
    }

    /// The CREATE TREE command surface. Every state change is logged and
    /// revision-inscribed; nothing is invented — generation reads the grimoires.
    fn desk_tree_command(&mut self, text: &str, level: u32, kb: &KnowledgeBase) -> String {
        let lower = text.to_lowercase();
        let trimmed = text.trim();

        if lower.starts_with("create tree") {
            let rest = trimmed["create tree".len()..].trim();
            // Bare `create tree` → begin the guided interrogation instead.
            if rest.is_empty() {
                self.commission_draft = Some(crate::codex_tree::CommissionDraft::default());
                let reply: String = "🌳 **What game quest tree do you want me to build?**\nName any realm with a grimoire — `minecraft`, `terraria`, `skyrim`… (Send `cancel` to set this aside.)".to_string();
                self.desk_push("rocket", &reply);
                self.log_action("rocket", "commission_opened", "");
                self.save();
                return reply;
            }
            let spec = match crate::codex_tree::parse_commission(rest) {
                Ok(s) => s,
                Err(e) => return format!("🌳 Commission incomplete: {}. Try `create tree for minecraft`.", e),
            };
            // Resolve realms — trees must map to real grimoires.
            let Some(g1) = Codex::match_game(&spec.game) else {
                return format!(
                    "🌳 No realm called '{}' is inscribed in the registry. Check `what games do you know`.",
                    spec.game
                );
            };
            let g2 = spec.game2.as_deref().and_then(Codex::match_game);
            if let Some(want) = &spec.game2 {
                if g2.is_none() {
                    return format!("🌳 No realm called '{}' is inscribed in the registry.", want);
                }
            }
            // Rank gate.
            let rank = OrderRank::from_level(level);
            if spec.archetype.rank_gate() > rank.index() {
                self.log_action("rocket", "tree_commission_refused", spec.archetype.label());
                return format!(
                    "🗝️ The {} requires the rank of **{}** (level {}). Rise within the Order first.",
                    spec.archetype.label(),
                    OrderRank::ALL[spec.archetype.rank_gate()].name(),
                    OrderRank::ALL[spec.archetype.rank_gate()].min_level()
                );
            }
            self.tree_seq += 1;
            let tree_id = format!("tree-{:03}", self.tree_seq);
            let tree = match crate::codex_tree::generate_tree(kb, &spec, tree_id, &g1, g2.as_ref())
            {
                Ok(t) => t,
                Err(e) => return format!("🌳 {}", e),
            };
            let line = format!(
                "🌳 **Quest Tree commissioned: {}**\n- Realm: {} ({})\n- Nodes: {} across tiers, {} pages total\n- Exercised sections: {}\n{}\n\nIt lives in its own tab — say `minimize tree`, `trees`, or refine it (`expand tree {} depth 2`).",
                tree.name,
                tree.game,
                tree.archetype.label(),
                tree.nodes.len(),
                tree.total_xp(),
                {
                    let secs: std::collections::BTreeSet<String> = tree
                        .nodes
                        .iter()
                        .flat_map(|n| n.grimoire_sections.iter().cloned())
                        .collect();
                    secs.into_iter().collect::<Vec<_>>().join(", ")
                },
                tree.subtitle
                    .as_ref()
                    .map(|f| format!("- Flavor: “{}”", f))
                    .unwrap_or_default(),
                tree.id
            );
            self.focused_tree = Some(tree.id.clone());
            self.log_action("rocket", "tree_commissioned", &tree.id);
            self.trees.push(tree);
            line
        } else if lower == "trees" || lower == "list trees" || lower == "list all trees" {
            if self.trees.is_empty() {
                return "🌳 The atelier is empty — commission one with `create tree for <game>`.".into();
            }
            let mut out = String::from("🌳 **The Atelier** — every living document:\n");
            for t in &self.trees {
                out.push_str(&format!("\n- {}", t.summary_line()));
            }
            let focused = self.focused_tree.as_deref().unwrap_or("-");
            out.push_str(&format!("\n\nFocused tab: `{}`. Minimized trees keep all state.", focused));
            self.log_action("rocket", "trees_listed", "");
            out
        } else {
            // All remaining commands address a specific tree by (fuzzy) name.
            // The literal word "tree" between the verb and the name is optional
            // ("expand tree X depth 2" == "expand X depth 2").
            let (verb, rest) = match lower.split_once(' ') {
                Some((v, r)) => {
                    let raw = trimmed[v.len()..].trim();
                    let stripped = raw
                        .strip_prefix("tree ")
                        .or_else(|| raw.strip_prefix("Tree "))
                        .unwrap_or(raw)
                        .trim();
                    (v, stripped.to_string())
                }
                None => (lower.as_str(), String::new()),
            };
            match verb {
                "open" | "restore" => self.tree_open(&rest),
                "minimize" => self.tree_minimize(&rest),
                "rename" => self.tree_rename(&rest),
                "duplicate" => self.tree_duplicate(&rest),
                "archive" => self.tree_archive(&rest),
                "destroy" => self.tree_destroy(&rest),
                "expand" => self.tree_expand(&rest, kb),
                "rebalance" => self.tree_rebalance(&rest),
                "note" => self.tree_note(&rest),
                "validate" => self.tree_validate(kb),
                "repair" => self.tree_repair(&rest, kb),
                "outline" => self.tree_outline(&rest),
                "reorder" => self.tree_reorder(&rest),
                "confirm" | "save" => self.trees_confirm_saved(),
                "report" => self.tree_report(&rest),
                "tree" => {
                    // `tree report <name>` / `tree <name>`
                    let rest2 = rest
                        .trim()
                        .strip_prefix("report ")
                        .unwrap_or(rest.trim());
                    self.tree_report(rest2)
                }
                _ => "🌳 Unknown tree directive. Try `create tree …`, `trees`, `open tree <name>`, `minimize tree`, `expand tree <name> depth 2`, `rebalance tree <name> to 500`, `validate trees`.".into(),
            }
        }
    }

    fn find_tree_mut(&mut self, name: &str) -> Option<&mut crate::codex_tree::QuestTree> {
        let lower = name.trim().to_lowercase();
        // Deictic references address the focused tree ("minimize tree",
        // "expand this tree", …).
        if lower.is_empty() || matches!(lower.as_str(), "tree" | "this" | "it" | "current") {
            let id = self.focused_tree.clone()?;
            return self.trees.iter_mut().find(|t| t.id == id);
        }
        // Exact id, then fuzzy name.
        if let Some(pos) = self.trees.iter().position(|t| t.id == lower) {
            return self.trees.get_mut(pos);
        }
        let pos = self.trees.iter().position(|t| {
            let n = t.name.to_lowercase();
            n.contains(&lower) || lower.contains(&n)
        })?;
        self.trees.get_mut(pos)
    }

    fn tree_open(&mut self, name: &str) -> String {
        // End the tree borrow before touching focused_tree / the audit log.
        let opened = self.find_tree_mut(name).map(|tree| {
            tree.active = true;
            tree.archived = false;
            (tree.id.clone(), tree.name.clone(), tree.summary_line())
        });
        let Some((id, tree_name, summary)) = opened else {
            return format!("🌳 No tree answers to '{}'. `trees` lists every living document.", name);
        };
        self.focused_tree = Some(id);
        self.log_action("rocket", "tree_restored", name);
        format!("🌳 '{}' restored to the atelier — full state intact ({}).", tree_name, summary)
    }

    /// Close a tree's TAB (✕ on the strip): minimizes it — the tree and all
    /// its progress stay safe — and re-focuses the next open tab.
    pub fn tree_tab_close(&mut self, id: &str) {
        let was_focused = self.focused_tree.as_deref() == Some(id);
        if let Some(tree) = self.trees.iter_mut().find(|t| t.id == id) {
            tree.active = false;
            self.log_action("rocket", "tree_tab_closed", id);
        }
        if was_focused {
            self.focused_tree = self
                .trees
                .iter()
                .find(|t| t.active && !t.archived)
                .map(|t| t.id.clone());
        }
        self.save();
    }

    pub fn tree_minimize(&mut self, name: &str) -> String {
        let minimized = self.find_tree_mut(name).map(|tree| {
            tree.active = false;
            (tree.id.clone(), tree.name.clone())
        });
        let Some((id, tree_name)) = minimized else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        self.log_action("rocket", "tree_minimized", &id);
        format!("🌳 '{}' minimized into its tab — progress, notes and revisions all kept.", tree_name)
    }

    fn tree_rename(&mut self, rest: &str) -> String {
        // `rename tree <name> to <new>`
        let lower_rest = rest.to_lowercase();
        let Some((name, _)) = lower_rest.split_once(" to ") else {
            return "🌳 Usage: `rename tree <name> to <new name>`.".into();
        };
        let new_name = rest[rest.to_lowercase().find(" to ").unwrap() + 4..].trim().to_string();
        let renamed = self.find_tree_mut(name).map(|tree| {
            let old = tree.name.clone();
            tree.name = new_name.clone();
            tree.revisions.push(crate::codex_tree::RevisionEntry {
                date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                directive: "rename".into(),
                summary: format!("'{}' → '{}'", old, new_name),
                nodes_changed: 0,
            });
            (tree.id.clone(), old)
        });
        let Some((id, old)) = renamed else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        self.log_action("rocket", "tree_renamed", &id);
        format!("🌳 Renamed '{}' → '{}'.", old, new_name)
    }

    fn tree_duplicate(&mut self, name: &str) -> String {
        let src = self.find_tree_mut(name).map(|t| (t.clone(), t.id.clone()));
        let Some((mut copy, old_id)) = src else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        self.tree_seq += 1;
        copy.id = format!("tree-{:03}", self.tree_seq);
        copy.name = format!("{} (copy)", copy.name);
        copy.active = true;
        copy.archived = false;
        let line = format!("🌳 Duplicated '{}' → '{}' (independent state, same blueprint).", old_id, copy.id);
        self.log_action("rocket", "tree_duplicated", &copy.id);
        self.trees.push(copy);
        line
    }

    fn tree_archive(&mut self, name: &str) -> String {
        let archived = self.find_tree_mut(name).map(|tree| {
            tree.archived = true;
            tree.active = false;
            (tree.id.clone(), tree.name.clone())
        });
        let Some((id, tree_name)) = archived else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        self.log_action("rocket", "tree_archived", &id);
        format!("🌳 '{}' archived — out of the tab bar, restorable any time (`restore tree`).", tree_name)
    }

    fn tree_destroy(&mut self, name: &str) -> String {
        let Some(pos) = self.trees.iter().position(|t| {
            let lower = name.trim().to_lowercase();
            t.id == lower || t.name.to_lowercase().contains(&lower) || lower.contains(&t.name.to_lowercase())
        }) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let gone = self.trees.remove(pos);
        if self.focused_tree.as_deref() == Some(gone.id.as_str()) {
            self.focused_tree = None;
        }
        self.log_action("rocket", "tree_destroyed", &format!("{} ({})", gone.id, gone.name));
        format!(
            "🔥 '{}' destroyed — {} node(s) and {} pages of state unmade. The deed is in the audit log.",
            gone.name,
            gone.nodes.len(),
            gone.xp_earned
        )
    }

    fn tree_expand(&mut self, rest: &str, kb: &KnowledgeBase) -> String {
        // `expand tree <name> depth N` — grafts N more tiers onto the frontier.
        let lower = rest.to_lowercase();
        let Some(pos) = lower.find("depth") else {
            return "🌳 Usage: `expand tree <name> depth 2`.".into();
        };
        let extra: u32 = lower[pos..]
            .split_whitespace()
            .filter_map(|w| w.parse::<u32>().ok())
            .next()
            .unwrap_or(1);
        let name = rest[..pos].trim();
        // Work on a clone to satisfy borrows; the engine appends surgically.
        let Some(idx) = self.trees.iter().position(|t| {
            let l = name.to_lowercase();
            t.id == l || t.name.to_lowercase().contains(&l) || l.contains(&t.name.to_lowercase())
        }) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let spec = crate::codex_tree::Commission {
            game: self.trees[idx].game.clone(),
            game2: self.trees[idx].game2.clone(),
            archetype: self.trees[idx].archetype,
            depth: extra,
            breadth: 2,
            curve: self.trees[idx].curve,
            name: None,
            flavor: None,
        };
        let Some(game) = Codex::match_game(&spec.game) else { return "🌳 The tree's realm no longer resolves.".into() };
        let g2 = spec.game2.as_deref().and_then(Codex::match_game);
        let tail = crate::codex_tree::generate_tree(kb, &spec, String::new(), &game, g2.as_ref());
        let Ok(tail) = tail else { return "🌳 The grimoire no longer supports this shape — try `validate trees`.".into() };
        // Graft the tail under the tree's current frontier leaves.
        let leaves: Vec<String> = {
            let t = &self.trees[idx];
            let parented: Vec<String> = t.nodes.iter().flat_map(|n| n.parents.clone()).collect();
            t.nodes
                .iter()
                .filter(|n| !parented.contains(&n.id))
                .map(|n| n.id.clone())
                .collect()
        };
        let mut added = 0;
        let start = self.trees[idx].nodes.len();
        let mut id_map: std::collections::BTreeMap<String, String> = Default::default();
        for (i, mut n) in tail.nodes.into_iter().enumerate() {
            // Re-id the tail — node ids must stay unique inside the tree.
            let old = n.id.clone();
            let new = format!("n{:02}", start + i + 1);
            id_map.insert(old, new.clone());
            n.id = new;
            n.tier += leaves.len() as u32; // depth marker only
            if n.parents.is_empty() {
                n.parents = leaves.clone();
            } else {
                // Remap tail-internal parents to the new ids.
                n.parents = n
                    .parents
                    .iter()
                    .map(|p| id_map.get(p).cloned().unwrap_or_else(|| p.clone()))
                    .collect();
            }
            self.trees[idx].nodes.push(n);
            added += 1;
        }
        self.trees[idx].revisions.push(crate::codex_tree::RevisionEntry {
            date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            directive: "expand".into(),
            summary: format!("+{} tier(s) grafted onto the frontier", extra),
            nodes_changed: added,
        });
        let (id, tree_name, total) = (
            self.trees[idx].id.clone(),
            self.trees[idx].name.clone(),
            self.trees[idx].nodes.len(),
        );
        self.log_action("rocket", "tree_expanded", &id);
        format!("🌳 '{}' grows — {} new node(s) grafted. The tree now holds {} nodes.", tree_name, added, total)
    }

    fn tree_rebalance(&mut self, rest: &str) -> String {
        // `rebalance tree <name> [to N]`
        let lower = rest.to_lowercase();
        let target: Option<u32> = lower
            .split_once(" to ")
            .and_then(|(_, n)| n.trim().parse::<u32>().ok());
        let name = match lower.split_once(" to ") {
            Some((n, _)) => rest[..n.len()].trim(),
            None => rest,
        };
        let Some(tree) = self.find_tree_mut(name) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let natural: u32 = tree
            .nodes
            .iter()
            .map(|n| tree.curve.xp_at(n.tier))
            .sum();
        let goal = target.unwrap_or(natural);
        let current: u32 = tree.nodes.iter().map(|n| n.xp_reward).sum();
        if current == 0 {
            return "🌳 Nothing to rebalance — the tree has no nodes.".into();
        }
        let scale = goal as f64 / current as f64;
        for n in tree.nodes.iter_mut() {
            n.xp_reward = ((n.xp_reward as f64) * scale).round() as u32;
        }
        tree.revisions.push(crate::codex_tree::RevisionEntry {
            date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            directive: "rebalance".into(),
            summary: format!("XP economy re-scaled: {} → {} pages", current, goal),
            nodes_changed: tree.nodes.len() as u32,
        });
        let (id, tree_name, count) = (tree.id.clone(), tree.name.clone(), tree.nodes.len());
        self.log_action("rocket", "tree_rebalanced", &id);
        format!(
            "⚖️ '{}' rebalanced — total economy now {} pages across {} nodes.",
            tree_name, goal, count
        )
    }

    fn tree_note(&mut self, rest: &str) -> String {
        // `note tree <name>: text`
        let Some((name, note)) = rest.split_once(':') else {
            return "🌳 Usage: `note tree <name>: your research note`.".into();
        };
        let Some(tree) = self.find_tree_mut(name) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let note_text = note.trim().to_string();
        let noted = self.find_tree_mut(name).map(|tree| {
            tree.notes.push_str(&format!("{}\n", note_text));
            (tree.id.clone(), tree.name.clone())
        });
        let Some((id, tree_name)) = noted else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        self.log_action("rocket", "tree_noted", &id);
        format!("🗒️ Noted on '{}'.", tree_name)
    }

    fn tree_validate(&mut self, kb: &KnowledgeBase) -> String {
        let stale = crate::codex_tree::validate_trees(&mut self.trees, kb);
        self.log_action("rocket", "trees_validated", &format!("{} stale", stale.len()));
        if stale.is_empty() {
            "🕯️ Every tree in the atelier still traces cleanly to its grimoire — no stale nodes.".into()
        } else {
            let mut out = String::from("⚠️ Stale nodes found (their sections no longer exist in the grimoire):\n");
            for (id, n) in &stale {
                if let Some(t) = self.trees.iter().find(|t| &t.id == id) {
                    out.push_str(&format!("\n- {} ({}) — {} stale node(s); `repair tree {}` re-binds them.", t.name, id, n, id));
                }
            }
            out
        }
    }

    fn tree_repair(&mut self, name: &str, kb: &KnowledgeBase) -> String {
        let Some(tree) = self.find_tree_mut(name) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let repaired = self.find_tree_mut(name).map(|tree| {
            let n = crate::codex_tree::repair_tree(tree, kb);
            (n, tree.id.clone(), tree.name.clone(), tree.game.clone())
        });
        let Some((repaired, id, tree_name, game)) = repaired else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        if repaired > 0 {
            self.log_action("rocket", "tree_repaired", &id);
        }
        format!(
            "🔧 '{}' — {} stale node(s) re-bound to living sections of the {} grimoire.",
            tree_name, repaired, game
        )
    }

    fn tree_report(&mut self, name: &str) -> String {
        // Detail view, read-only. Borrow is ended before the audit log write.
        let lower = name.trim().to_lowercase();
        let snapshot = self
            .trees
            .iter()
            .find(|t| {
                !lower.is_empty()
                    && (t.id == lower
                        || t.name.to_lowercase().contains(&lower)
                        || lower.contains(&t.name.to_lowercase()))
            })
            .or_else(|| {
                self.trees
                    .iter()
                    .find(|t| Some(&t.id) == self.focused_tree.as_ref())
            })
            .cloned();
        let Some(tree) = snapshot else {
            return "🌳 No tree is focused — `open tree <name>` or `create tree for <game>`.".into();
        };
        let mut out = format!(
            "🌳 **{}**\n- {} · realm: {}{} · rank gate: {}+\n- {} pages earned · revisions: {}\n",
            tree.name,
            tree.archetype.label(),
            tree.game,
            tree.game2.as_ref().map(|g| format!(" ⚔️ {}", g)).unwrap_or_default(),
            tree.rank_gate + 1,
            tree.xp_earned,
            tree.revisions.len()
        );
        if let Some(f) = &tree.subtitle {
            out.push_str(&format!("- Flavor: “{}”\n", f));
        }
        if !tree.notes.is_empty() {
            out.push_str(&format!("- Notes: {}\n", tree.notes.trim()));
        }
        out.push_str("\n**Nodes** (frontier marked ⚔️):\n");
        let frontier = tree.frontier();
        for n in &tree.nodes {
            let mark = if frontier.contains(&n.id) {
                "⚔️"
            } else if n.completed {
                "✅"
            } else {
                "·"
            };
            out.push_str(&format!(
                "\n{} [T{}] {} — {}\n  {} · +{} pages · sections: {}\n",
                mark,
                n.tier,
                n.title,
                if n.completed { "sealed" } else if n.stale { "⚠️ STALE" } else { "open" },
                n.progress_line(),
                n.xp_reward,
                n.grimoire_sections.join(", ")
            ));
        }
        self.log_action("rocket", "tree_reported", &tree.id);
        out
    }

    // -- 🌳 CREATE TREE — conversational commission + graph surface ----------

    /// The Create Tree button: opens the atelier conversation and asks the
    /// one question that always comes first.
    pub fn begin_commission(&mut self, suggested_game: Option<String>) -> String {
        self.commission_draft = Some(crate::codex_tree::CommissionDraft {
            suggested_game: suggested_game.clone(),
            ..Default::default()
        });
        let opening = match &suggested_game {
            Some(g) => format!(
                "🌳 **What game quest tree do you want me to build?**\nSend `current` to use {g} — or name any realm. One word and I build it instantly; add a shape if you like (`branching depth 4 steep`). (`cancel` sets this aside.)"
            ),
            None => "🌳 **What game quest tree do you want me to build?**\nName any realm with a grimoire — `minecraft`, `skyrim`… — and I build it the moment you answer. (Add a shape if you like: `branching depth 4 steep` · `cancel` sets this aside.)".into(),
        };
        self.desk_push("rocket", &opening);
        self.log_action("rocket", "commission_opened", "");
        self.save();
        opening
    }

    /// One step of the conversational commission. Returns the reply.
    /// One step of the conversational commission — now ONE answer: name the
    /// game and the tree is built instantly with sensible defaults. Shape
    /// words in the same answer (`branching depth 4 steep`) are honored, and
    /// refinement stays available afterwards (`expand`, `rebalance`, …).
    fn commission_step(&mut self, text: &str, level: u32, kb: &KnowledgeBase) -> Option<String> {
        let draft = self.commission_draft.as_ref()?;
        let mut lower = text.to_lowercase().trim().to_string();
        // `current` answers with the game already selected in the Main Hall.
        if lower == "current" {
            match &draft.suggested_game {
                Some(g) => lower = g.to_lowercase(),
                None => {
                    return Some(
                        "🕯️ No game is selected in the Main Hall — name a realm instead (e.g. `minecraft`).".into(),
                    )
                }
            }
        }

        // Resolve the answer into a full commission spec: the whole line may
        // already carry a shape ("skyrim branching depth 4"), or it may be a
        // bare game name (defaults applied).
        let spec = match crate::codex_tree::parse_commission(&lower) {
            Ok(s) => s,
            Err(e) => {
                return Some(format!(
                    "🌳 Hmm — {e}. Name a realm with a grimoire (e.g. `minecraft`), or `cancel` to set this aside."
                ))
            }
        };
        if spec.game.is_empty() {
            return Some("🌳 Name a realm — e.g. `minecraft` — and I shall build.".into());
        }
        let Some(g1) = Codex::match_game(&spec.game) else {
            return Some(format!(
                "🌳 No realm called '{}' is inscribed in the registry. Check `what games do you know`.",
                spec.game
            ));
        };
        let g2 = spec.game2.as_deref().and_then(Codex::match_game);
        if let Some(want) = &spec.game2 {
            if g2.is_none() {
                return Some(format!("🌳 No realm called '{want}' is inscribed in the registry."));
            }
        }
        let rank = OrderRank::from_level(level);
        if spec.archetype.rank_gate() > rank.index() {
            return Some(format!(
                "🗝️ The {} requires the rank of **{}** (level {}). Rise within the Order first — or commission a humbler shape.",
                spec.archetype.label(),
                OrderRank::ALL[spec.archetype.rank_gate()].name(),
                OrderRank::ALL[spec.archetype.rank_gate()].min_level()
            ));
        }
        self.tree_seq += 1;
        let tree_id = format!("tree-{:03}", self.tree_seq);
        let tree = match crate::codex_tree::generate_tree(kb, &spec, tree_id, &g1, g2.as_ref()) {
            Ok(t) => t,
            Err(e) => return Some(format!("🌳 {e}")),
        };
        let summary = format!(
            "🌳 **Quest Tree commissioned: {}**\n- {} nodes across {} tiers · {} pages total\n- Sections exercised: {}\n\nIt is pinned in the atelier as its own tab — click its nodes to read them, and **Mark Done** when their moment comes. Refine anytime: `expand tree …` · `rebalance tree …`.",
            tree.name,
            tree.nodes.len(),
            tree.nodes.iter().map(|n| n.tier).max().unwrap_or(0) + 1,
            tree.total_xp(),
            {
                let secs: std::collections::BTreeSet<String> = tree
                    .nodes
                    .iter()
                    .flat_map(|n| n.grimoire_sections.iter().cloned())
                    .collect();
                secs.into_iter().collect::<Vec<_>>().join(", ")
            }
        );
        self.focused_tree = Some(tree.id.clone());
        self.log_action("rocket", "tree_commissioned", &tree.id);
        self.chronicle(
            "tree",
            format!(
                "Commissioned quest tree '{}' ({}, {})",
                tree.name,
                tree.game,
                tree.archetype.label()
            ),
        );
        self.trees.push(tree);
        self.commission_draft = None;
        Some(summary)
    }

    pub fn tree_complete_node(
        &mut self,
        tree_id: &str,
        node_id: &str,
    ) -> Result<(u32, Vec<String>), String> {
        let Some(tree) = self.trees.iter_mut().find(|t| t.id == tree_id) else {
            return Err("That tree is no longer in the atelier.".into());
        };
        let (xp, mut lines, lit) = crate::codex_tree::complete_node(tree, node_id)?;
        if !lit.is_empty() {
            lines.push(format!("✨ {} new path(s) lit forward.", lit.len()));
        }
        self.log_action("order", "node_sealed_manual", node_id);
        self.save();
        Ok((xp, lines))
    }

    /// Confirmation that every tree's state has been safely written.
    pub fn trees_confirm_saved(&mut self) -> String {
        self.save();
        let count = self.trees.len();
        let done: usize = self.trees.iter().map(|t| t.completed_count()).sum();
        let sealed = self.trees.iter().filter(|t| t.completed_sealed).count();
        self.log_action("order", "trees_persisted", &format!("{} trees", count));
        format!(
            "💾 **All {} quest tree(s) safely written** to `{}` ({} node(s) sealed, {} tree(s) fully sealed). Nothing in the atelier can be lost — every change saves itself the moment it happens.",
            count,
            crate::codex::CODEX_FILE,
            done,
            sealed
        )
    }

    /// Pure textual outline of one tree (the graph stays the primary surface).
    /// Move a tree's tab by `delta` positions (reorder, persisted).
    pub fn tree_tab_move(&mut self, id: &str, delta: i32) {
        let Some(pos) = self.trees.iter().position(|t| t.id == id) else {
            return;
        };
        let new_pos = (pos as i32 + delta).clamp(0, self.trees.len() as i32 - 1) as usize;
        if new_pos == pos {
            return;
        }
        let tree = self.trees.remove(pos);
        self.trees.insert(new_pos, tree);
        let moved = self.trees[new_pos].id.clone();
        self.log_action("rocket", "tree_reordered", &moved);
        self.save();
    }

    pub fn tree_outline(&mut self, name: &str) -> String {
        let lower = name.trim().to_lowercase();
        let Some(tree) = self.trees.iter().find(|t| {
            !lower.is_empty()
                && (t.id == lower
                    || t.name.to_lowercase().contains(&lower)
                    || lower.contains(&t.name.to_lowercase()))
        }) else {
            return format!("🌳 No tree answers to '{}'. `trees` lists every living document.", name);
        };
        crate::codex_tree::outline(tree)
    }

    /// Reorder a tree's tab position ("up"/"down").
    pub fn tree_reorder(&mut self, rest: &str) -> String {
        let lower = rest.to_lowercase();
        let (name, dir) = if lower.ends_with(" up") {
            (rest[..rest.len() - 3].trim(), -1i32)
        } else if lower.ends_with(" down") {
            (rest[..rest.len() - 5].trim(), 1i32)
        } else {
            (rest.trim(), 1i32)
        };
        let Some(pos) = self.trees.iter().position(|t| {
            let l = name.to_lowercase();
            t.id == l || t.name.to_lowercase().contains(&l) || l.contains(&t.name.to_lowercase())
        }) else {
            return format!("🌳 No tree answers to '{}'.", name);
        };
        let new_pos = (pos as i32 + dir).clamp(0, self.trees.len() as i32 - 1) as usize;
        if new_pos == pos {
            return format!("🌳 '{}' is already at that edge of the atelier.", self.trees[pos].name);
        }
        let tree = self.trees.remove(pos);
        self.trees.insert(new_pos, tree);
        let (id, name) = (self.trees[new_pos].id.clone(), self.trees[new_pos].name.clone());
        let order = self
            .trees
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .join(" · ");
        self.log_action("rocket", "tree_reordered", &id);
        format!("🌳 '{}' moved — the atelier now reads: {}.", name, order)
    }

    fn desk_help(&self) -> String {
        "🪶 **Rocket, familiar of the Order, at your service.** My practical arts:\n\n\
         - `quest report` — the state of the quest board\n\
         - `set me a trial` — I pin a fresh quest drawn from the grimoires (Journeyman+)\n\
         - `audit <game>` — completeness audit of a grimoire\n\
         - `gaps <game>` — list a grimoire's missing disciplines\n\
         - `compare <a> and <b>` — scholarship across two grimoires\n\
         - `where stand I` — your rank, pages, and realm tiers\n\
         - `chronicle` — export your deeds to codex_chronicle.md\n\
         - `log` — the auditable record of my modifications\n\
         - `interjections on/off` — may I note things in the main channel?\n\
         - `clear the desk` — a fresh page"
            .to_string()
    }

    fn desk_board_report(&self) -> String {
        let open: Vec<&CodexQuest> = self.board.iter().filter(|q| !q.sealed).collect();
        let sealed = self.board.len() - open.len();
        if open.is_empty() {
            return format!(
                "📜 The board stands empty of open quests ({} sealed in the annals). Ask me to `set me a trial`.",
                sealed
            );
        }
        let mut out = format!("📜 **The Quest Board** — {} open, {} sealed:\n", open.len(), sealed);
        for q in open {
            out.push_str(&format!(
                "\n{} **{}**\n  {}\n{}\n",
                q.kind.label(),
                q.title,
                q.progress_line(),
                q.grimoire_sections.first().map(|s| format!("  📚 {}", s)).unwrap_or_default()
            ));
        }
        out
    }

    fn desk_generate(&mut self, kb: &KnowledgeBase, level: u32, _text: &str) -> String {
        let rank = OrderRank::from_level(level);
        if !desk_authoring_unlocked(rank) {
            self.log_action("rocket", "quest_generate_refused", "rank too low");
            return format!(
                "🗝️ The board's deeper shelves are sealed until you are a **Journeyman Scribe** (level 10). You are a {} ({}).",
                rank.name(),
                OrderRank::from_level(level).min_level()
            );
        }
        // One focused pin, even if the board is nearly full.
        if self.open_count() >= BOARD_CAPACITY + 2 {
            return "📜 The board is full — seal something first (`quest report` shows the way).".to_string();
        }
        let Some(quest) = self.next_candidate(kb, rank) else {
            return "📜 Every shelf of the board is already occupied by a fitting trial. Seal one first.".to_string();
        };
        let line = format!(
            "📌 Pinned by my own wing: {} — **{}** (+{} pages, +{} lore)\n  📚 exercises: {}\n  Start by asking grounded questions in the realm.",
            quest.kind.label(),
            quest.title,
            quest.xp_reward,
            quest.lore_reward,
            quest.grimoire_sections.join(", ")
        );
        self.log_action("rocket", "quest_pinned", &quest.id);
        self.chronicle("quest", format!("Rocket pinned {} — {}", quest.kind.label(), quest.title));
        line
    }

    /// Fuzzy-match a game name from free text against the registry.
    pub fn match_game(query: &str) -> Option<GameType> {
        let q = query.trim().to_lowercase();
        if q.len() < 3 {
            return None;
        }
        crate::CATALOG
            .iter()
            .find(|g| {
                let name = g.name.to_lowercase();
                name.contains(&q) || q.contains(&name)
            })
            .map(|g| g.id.clone())
    }

    fn desk_audit(&mut self, kb: &KnowledgeBase, text: &str, mode: &str) -> String {
        // "audit <game>" / "audit the grimoire" (falls back to current realm name).
        let arg = text
            .trim_start_matches(|c: char| c.is_ascii_punctuation())
            .split_whitespace()
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ")
            .trim_start_matches("the grimoire")
            .trim_start_matches("the")
            .trim()
            .to_string();
        let game = Self::match_game(&arg);
        let Some(game) = game else {
            return "🕯️ Name a realm, scribe — e.g. `audit minecraft` or `gaps terraria`.".to_string();
        };
        let entries = kb.entries_for_game(&game);
        let mut by_cat: BTreeMap<&'static str, usize> = BTreeMap::new();
        for e in &entries {
            *by_cat.entry(e.category.label()).or_insert(0) += 1;
        }
        let all_cats = ["🎯 Strategy", "⚙️ Mechanics", "🗡️ Items & Gear", "👥 Characters & Roles", "📈 Progression", "📖 Glossary"];
        let missing: Vec<&str> = all_cats
            .iter()
            .copied()
            .filter(|c| !by_cat.contains_key(c))
            .collect();
        self.log_action("rocket", mode, game.get_name());

        if mode == "gaps" {
            if missing.is_empty() {
                return format!(
                    "🔎 The {} grimoire lacks nothing — all six disciplines are inscribed ({} entries).",
                    game.get_name(),
                    entries.len()
                );
            }
            return format!(
                "🔎 **Gaps in the {} grimoire** ({} entries inscribed):\n{}",
                game.get_name(),
                entries.len(),
                missing
                    .iter()
                    .map(|c| format!("\n- {} — no entries", c))
                    .collect::<String>()
            );
        }

        let realm = self.realms.get(game.get_name());
        let (tier, _) = tier_for(realm.map(|r| r.lore_points).unwrap_or(0));
        format!(
            "🕯️ **Completeness audit — {} grimoire**\n- Entries: {}\n- Disciplines covered: {} / 6{}\n- Realm standing: {} ({} lore)\n- Verdict: {}",
            game.get_name(),
            entries.len(),
            by_cat.len(),
            if missing.is_empty() {
                String::new()
            } else {
                format!("\n- Missing: {}", missing.join(", "))
            },
            tier,
            realm.map(|r| r.lore_points).unwrap_or(0),
            if entries.len() >= 8 && missing.is_empty() {
                "a complete grimoire by the Order's standard"
            } else if entries.len() >= 4 {
                "serviceable, Restoration worthy"
            } else {
                "thin — Restoration Quest advised"
            }
        )
    }

    fn desk_compare(&mut self, kb: &KnowledgeBase, text: &str) -> String {
        let lower = text.to_lowercase();
        let rest = lower
            .trim_start_matches("comparative")
            .trim_start_matches("compare")
            .trim();
        let (a, b) = if let Some(pos) = rest.find(" and ") {
            (rest[..pos].trim(), rest[pos + 5..].trim())
        } else if let Some(pos) = rest.find(" with ") {
            (rest[..pos].trim(), rest[pos + 6..].trim())
        } else if let Some(pos) = rest.find(" vs ") {
            (rest[..pos].trim(), rest[pos + 4..].trim())
        } else {
            (rest.trim(), "")
        };
        let (Some(g1), Some(g2)) = (Self::match_game(a), Self::match_game(b)) else {
            return "⚖️ Name two realms, scholar — e.g. `compare minecraft and terraria`.".to_string();
        };
        if g1 == g2 {
            return "⚖️ A grimoire cannot be compared against itself — choose two realms.".to_string();
        }
        self.log_action("rocket", "comparative", &format!("{} vs {}", g1.get_name(), g2.get_name()));
        let cats = ["🎯 Strategy", "⚙️ Mechanics", "🗡️ Items & Gear", "👥 Characters & Roles", "📈 Progression", "📖 Glossary"];
        let mut out = format!(
            "⚖️ **Comparative scholarship — {} ⚔️ {}**\n\n| Discipline | {} | {} |\n|---|---|---|\n",
            g1.get_name(),
            g2.get_name(),
            g1.get_name(),
            g2.get_name()
        );
        for cat in cats {
            let n1 = kb
                .entries_for_game(&g1)
                .iter()
                .filter(|e| e.category.label() == cat)
                .count();
            let n2 = kb
                .entries_for_game(&g2)
                .iter()
                .filter(|e| e.category.label() == cat)
                .count();
            out.push_str(&format!("| {} | {} | {} |\n", cat, n1, n2));
        }
        out.push_str("\n_pin a Comparative Scholarship quest by asking me to `set me a trial`._");
        out
    }

    fn desk_character_report(&mut self, level: u32) -> String {
        let rank = OrderRank::from_level(level);
        let mut studied: Vec<(String, u32)> = self
            .realms
            .iter()
            .map(|(name, r)| (name.clone(), r.lore_points))
            .collect();
        studied.sort_by(|a, b| b.1.cmp(&a.1));
        let mut out = format!(
            "🗂️ **Scholar's report**\n- Order standing: {} {} (level {})\n- Realms charted: {}\n- Open quests: {} · sealed: {}\n- Ordeal streak: {} grounded answers\n",
            rank.sigil(),
            rank.name(),
            level,
            self.realms.len(),
            self.open_count(),
            self.board.iter().filter(|q| q.sealed).count(),
            self.ordeal_streak
        );
        for (name, lore) in studied.iter().take(6) {
            let (tier, next) = tier_for(*lore);
            out.push_str(&format!(
                "- 📖 {} — **{}** ({} lore{})\n",
                name,
                tier,
                lore,
                if next > 0 { format!(", {} to rise", next) } else { String::new() }
            ));
        }
        self.log_action("rocket", "character_report", "");
        out
    }

    fn desk_export_chronicle(&mut self, level: u32) -> String {
        let rank = OrderRank::from_level(level);
        let mut md = String::from("# Chronicle of the Order\n\n_Deeds of the scholar, as inscribed by the Order of the Eternal Codex._\n\n");
        md.push_str(&format!("- Standing: {} {} (level {})\n", rank.sigil(), rank.name(), level));
        md.push_str(&format!("- Realms charted: {}\n\n## Deeds\n\n", self.realms.len()));
        for e in self.chronicle.iter().rev() {
            md.push_str(&format!("- **{}** [{}] {}\n", e.date, e.kind, e.text));
        }
        let path = base_path(CHRONICLE_FILE);
        let written = std::fs::write(path, md).is_ok();
        self.log_action(
            "rocket",
            "chronicle_export",
            if written { CHRONICLE_FILE } else { "FAILED" },
        );
        if written {
            format!("📜 Chronicle exported — {} deeds inscribed to `{}`.", self.chronicle.len(), CHRONICLE_FILE)
        } else {
            "📜 The quill snapped — could not write the chronicle file.".to_string()
        }
    }

    fn desk_action_log(&self) -> String {
        if self.actions.is_empty() {
            return "🗂️ The audit log is blank — no modifications yet.".to_string();
        }
        let mut out = String::from("🗂️ **Audit log** (most recent last):\n");
        for a in self.actions.iter().rev().take(12).rev() {
            out.push_str(&format!(
                "\n- `{}` [{}] {}: {}",
                a.timestamp, a.actor, a.action, a.detail
            ));
        }
        out
    }

    fn desk_toggle_interjections(&mut self, text: &str) -> String {
        let lower = text.to_lowercase();
        let new_val = if lower.contains("off") {
            false
        } else if lower.contains("on") {
            true
        } else {
            !self.desk_interjections
        };
        self.desk_interjections = new_val;
        self.log_action("rocket", "interjections_set", if new_val { "on" } else { "off" });
        if new_val {
            "🕯️ So granted — I shall leave brief marginalia in the main channel when a quest is sealed or a realm rises.".to_string()
        } else {
            "🕯️ Withheld — my notes shall stay on this desk alone.".to_string()
        }
    }
}

fn quest_shell() -> CodexQuest {
    CodexQuest {
        id: String::new(),
        kind: QuestKind::LoreMastery,
        title: String::new(),
        game: None,
        game2: None,
        category: None,
        target: 1,
        progress: 0,
        xp_reward: 0,
        lore_reward: 0,
        rank_required: 0,
        grimoire_sections: Vec::new(),
        sealed: false,
        realm_progress: BTreeMap::new(),
        created: String::new(),
    }
}

/// The most-covered discipline of a realm's grimoire.
fn common_category(kb: &KnowledgeBase, g: &GameType) -> Option<&'static str> {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for e in kb.entries_for_game(g) {
        *counts.entry(e.category.label()).or_insert(0) += 1;
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(c, _)| c)
}

/// The least-covered (but non-empty) discipline — the underexplored edge.
fn rarest_category(kb: &KnowledgeBase, g: &GameType) -> Option<&'static str> {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for e in kb.entries_for_game(g) {
        *counts.entry(e.category.label()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n >= 1)
        .min_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
}

// ---------------------------------------------------------------------------
// Narration — the Order's voice
// ---------------------------------------------------------------------------

pub fn narrate_xp(xp: u32) -> String {
    format!("+{} pages transcribed", xp)
}

pub fn proclaim_level_up(level: u32) -> String {
    let rank = OrderRank::from_level(level);
    format!(
        "📜 PROCLAMATION — the Order elevates you: {} {}, Level {}!",
        rank.sigil(),
        rank.name(),
        level
    )
}

pub fn quest_seal_line(kind_label: &str, title: &str, xp: u32, lore: u32) -> String {
    format!("✒️ Quest sealed: {} — {} (+{} pages, +{} lore)", kind_label, title, xp, lore)
}

pub fn realm_rise_line(game: &str, tier: &str) -> String {
    format!("📖 The {} grimoire rises to {} in your Codex.", game, tier)
}

// JSON shape documentation used by src/docs_gen.rs (codex section).
pub fn codex_schema_summary() -> serde_json::Value {
    json!({
        "file": CODEX_FILE,
        "version": CODEX_VERSION,
        "ranks": OrderRank::ALL.iter().map(|r| json!({
            "rank": r.name(),
            "min_level": r.min_level(),
            "sigil": r.sigil(),
        })).collect::<Vec<_>>(),
        "realm_tiers": REALM_TIERS.iter().map(|(t, th)| json!({"tier": t, "lore_points": th})).collect::<Vec<_>>(),
        "quest_types": ["LoreMastery", "EdgeCaseExpedition", "ComparativeScholarship", "Restoration", "TrialByOrdeal"],
        "board_capacity": BOARD_CAPACITY,
        "xp_model": {
            "novelty_first_tag_touch": {"xp": 5, "lore": 2},
            "grounded_answer": {"xp": 0, "lore": 1},
            "depth_6_plus_words": {"xp": 3, "lore": 0},
            "note": "bonuses are added on top of the standard conversation XP (memory.rs)"
        },
        "quest_rewards": {"LoreMastery": [60, 5], "EdgeCaseExpedition": [80, 8], "ComparativeScholarship": [90, 10], "Restoration": [100, 12], "TrialByOrdeal": [120, 15]},
        "desk_commands": ["help", "board", "set me a trial", "audit <game>", "gaps <game>", "compare <a> and <b>", "where stand I", "chronicle", "log", "interjections on/off", "clear the desk"],
        "tree_atelier": {
            "feature": "CREATE TREE — multi-instance quest-tree authoring environment",
            "archetypes": crate::codex_tree::TreeArchetype::ALL
                .iter()
                .map(|a| json!({"archetype": a.label(), "rank_gate": a.rank_gate()}))
                .collect::<Vec<_>>(),
            "desk_commands": [
                "create tree [archetype] for <game> [and <game2>] [depth N] [breadth N] [gentle|steep] [flavor: …] [named …]",
                "trees", "open tree <name>", "restore tree <name>", "minimize tree [name]",
                "rename tree <name> to <new>", "duplicate tree <name>", "archive tree <name>",
                "destroy tree <name>", "expand tree <name> depth N", "rebalance tree <name> [to N]",
                "note tree <name>: <text>", "validate trees", "repair tree <name>", "tree report <name>",
                "outline tree <name>", "reorder tree <name> up|down", "confirm saved",
                "(Create Tree button → guided interrogation: game → shape → generated graph)"
            ],
            "graph_ui": "interactive canvas: circular nodes (❓ available / ✓ sealed / · locked), automatic tier-column layout, edges glow+pulse when their source seals, click-to-select with synchronized detail panel and Mark Done (frontier-only; locked nodes explain prerequisites in character)",
            "node_traceability": "every node's grimoire_sections are validated against the live knowledge base; stale nodes are flagged and repairable",
            "completion": "node criteria are event counters (grounded answers per discipline, teach events, ordeal runs, comparative realm answers); sealing a node pays its XP through the global economy; sealing all required nodes pays a 10% tree bonus"
        },
        "chronicle_file": CHRONICLE_FILE,
        "caps": {"chronicle": CHRONICLE_CAP, "actions": ACTIONS_CAP, "desk_history": DESK_CAP, "entries_seen_per_realm": 64}
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn minecraft() -> GameType {
        Codex::match_game("minecraft").expect("minecraft in registry")
    }

    #[test]
    fn rank_ladder_bands_and_privileges() {
        assert_eq!(OrderRank::from_level(1), OrderRank::NoviceScribe);
        assert_eq!(OrderRank::from_level(4), OrderRank::NoviceScribe);
        assert_eq!(OrderRank::from_level(5), OrderRank::InkApprentice);
        assert_eq!(OrderRank::from_level(10), OrderRank::JourneymanScribe);
        assert_eq!(OrderRank::from_level(15), OrderRank::AdeptScholar);
        assert_eq!(OrderRank::from_level(25), OrderRank::Loremaster);
        assert_eq!(OrderRank::from_level(35), OrderRank::MasterArchivist);
        assert_eq!(OrderRank::from_level(45), OrderRank::GrandArchivist);
        // Privilege gating.
        assert!(quest_tier_unlocked(OrderRank::NoviceScribe, QuestKind::LoreMastery));
        assert!(!quest_tier_unlocked(OrderRank::NoviceScribe, QuestKind::EdgeCaseExpedition));
        assert!(quest_tier_unlocked(OrderRank::InkApprentice, QuestKind::EdgeCaseExpedition));
        assert!(!quest_tier_unlocked(OrderRank::InkApprentice, QuestKind::Restoration));
        assert!(quest_tier_unlocked(OrderRank::Loremaster, QuestKind::TrialByOrdeal));
        assert!(!desk_authoring_unlocked(OrderRank::InkApprentice));
        assert!(desk_authoring_unlocked(OrderRank::JourneymanScribe));
    }

    #[test]
    fn realm_tiers_progress() {
        assert_eq!(tier_for(0).0, "Uncharted");
        assert_eq!(tier_for(9).0, "Uncharted");
        assert_eq!(tier_for(10).0, "Charted");
        assert_eq!(tier_for(25).0, "Studied");
        assert_eq!(tier_for(200).0, "Sanctified");
        // Next-threshold is surfaced for the XP-bar-style display.
        assert_eq!(tier_for(9).1, 10);
        assert_eq!(tier_for(200).1, 0);
    }

    #[test]
    fn grounded_questions_award_lore_and_novelty_xp() {
        let mut codex = Codex::default();
        let g = minecraft();

        // First grounded question in a NEW tag: novelty (+5 XP, +2 lore).
        // Short question (< 6 words) so the depth bonus does not fire.
        let out = codex.record_realm_question(&g, Some("🗡️ Items & Gear"), "craft a diamond sword", true);
        assert_eq!(out.xp_bonus, 5);
        let realm = codex.realms.get(g.get_name()).unwrap();
        assert_eq!(realm.lore_points, 2);
        assert_eq!(realm.questions_answered, 1);

        // Same tag again: no novelty (+0 XP, +1 lore); short question: no depth.
        let out = codex.record_realm_question(&g, Some("🗡️ Items & Gear"), "more", true);
        assert_eq!(out.xp_bonus, 0);

        // Long question: depth bonus (+3).
        let out = codex.record_realm_question(&g, Some("🗡️ Items & Gear"), "what is the absolute best armor set and where do i find every piece", true);
        assert_eq!(out.xp_bonus, 3);

        // Novel tag on a long question: novelty + depth (+8).
        let out = codex.record_realm_question(&g, Some("⚙️ Mechanics"), "explain the full machine mechanics with frames and timings please", true);
        assert_eq!(out.xp_bonus, 8);

        // Non-grounded answer: no lore, ordeal reset, no crash.
        let out = codex.record_realm_question(&g, None, "hello there my good friend", false);
        assert_eq!(out.xp_bonus, 0);
        assert_eq!(codex.ordeal_streak, 0);
    }

    #[test]
    fn lore_mastery_quest_lifecycle() {
        let kb = KnowledgeBase::new();
        let mut codex = Codex::default();
        // Level 25 → all tiers unlocked.
        codex.ensure_board(&kb, 25);
        assert!(codex.open_count() > 0, "board pins quests at high rank");

        // Drive every open quest by its own pinned realm/category until
        // something seals — the engine must reward real scholarship.
        let mut sealed_any = false;
        for _ in 0..30 {
            // Snapshot the open quests, then satisfy each one.
            let open: Vec<(String, Option<String>, Option<String>)> = codex
                .board
                .iter()
                .filter(|q| !q.sealed)
                .map(|q| (q.kind.id_prefix().to_string(), q.game.clone(), q.category.clone()))
                .collect();
            let mut did_work = false;
            for (kind, game, category) in &open {
                match kind.as_str() {
                    "restore" => {
                        // Restoration advances via teaching in the quest realm.
                        if let Some(gname) = game {
                            if let Some(g) = Codex::match_game(gname) {
                                let _ = codex.record_teaching(&g);
                                did_work = true;
                            }
                        }
                    }
                    _ => {
                        // Lore / Edge / Comparative / Ordeal advance via
                        // grounded answers in the quest realm + category.
                        let Some(gname) = game.clone().or_else(|| {
                            // Ordeal quests have no realm — use any realm.
                            codex.board.iter().find(|q| !q.sealed).and_then(|q| q.game.clone())
                        }) else {
                            continue;
                        };
                        let Some(g) = Codex::match_game(&gname) else { continue };
                        let out = codex.record_realm_question(
                            &g,
                            category.as_deref(),
                            "a long and detailed question about this discipline of the realm",
                            true,
                        );
                        if out.quest_xp > 0 {
                            sealed_any = true;
                        }
                        did_work = true;
                    }
                }
            }
            codex.ensure_board(&kb, 25);
            if sealed_any || !did_work {
                break;
            }
        }
        assert!(sealed_any, "some quest must seal from satisfied scholarship");
        assert!(codex.chronicle.iter().any(|c| c.kind == "quest"));
        assert!(codex.actions.iter().any(|a| a.action == "quest_sealed"));
    }

    #[test]
    fn ordeal_streak_progresses_and_resets() {
        let mut codex = Codex::default();
        let g = minecraft();
        // Ordeal quests need Loremaster; test the streak mechanics directly.
        codex.board.push(CodexQuest {
            id: "ordeal-test".into(),
            kind: QuestKind::TrialByOrdeal,
            title: "test ordeal".into(),
            game: None,
            game2: None,
            category: None,
            target: 3,
            progress: 0,
            xp_reward: 120,
            lore_reward: 15,
            rank_required: 4,
            grimoire_sections: vec![],
            sealed: false,
            realm_progress: BTreeMap::new(),
            created: String::new(),
        });
        codex.record_realm_question(&g, Some("⚙️ Mechanics"), "q1", true);
        codex.record_realm_question(&g, Some("🗡️ Items & Gear"), "q2", true);
        assert_eq!(codex.ordeal_streak, 2);
        assert_eq!(codex.board[0].progress, 2);
        codex.record_realm_question(&g, Some("⚙️ Mechanics"), "q3 long enough question here", true);
        assert_eq!(codex.ordeal_streak, 3);
        assert!(codex.board[0].sealed, "ordeal seals at 3 grounded answers");
        // A non-grounded answer resets the streak (for the NEXT ordeal).
        codex.record_realm_question(&g, None, "smalltalk", false);
        assert_eq!(codex.ordeal_streak, 0);
    }

    #[test]
    fn comparative_quest_needs_both_realms() {
        let mut codex = Codex::default();
        let g1 = minecraft();
        let g2 = Codex::match_game("terraria").expect("terraria in registry");
        let g1n = g1.get_name().to_string();
        let g2n = g2.get_name().to_string();
        codex.board.push(CodexQuest {
            id: "compare-test".into(),
            kind: QuestKind::ComparativeScholarship,
            title: "test compare".into(),
            game: Some(g1n.clone()),
            game2: Some(g2n.clone()),
            category: Some("⚙️ Mechanics".into()),
            target: 2,
            progress: 0,
            xp_reward: 90,
            lore_reward: 10,
            rank_required: 2,
            grimoire_sections: vec![],
            sealed: false,
            realm_progress: BTreeMap::new(),
            created: String::new(),
        });
        codex.record_realm_question(&g1, Some("⚙️ Mechanics"), "mechanics question one", true);
        assert!(!codex.board[0].sealed, "one realm is not enough");
        codex.record_realm_question(&g2, Some("⚙️ Mechanics"), "mechanics question two", true);
        assert!(codex.board[0].sealed, "both realms seal the comparative quest");
        // Wrong category does not advance.
        codex.board.push(CodexQuest {
            id: "compare-test-2".into(),
            kind: QuestKind::ComparativeScholarship,
            title: "test compare 2".into(),
            game: Some(g1n.clone()),
            game2: Some(g2n.clone()),
            category: Some("🎯 Strategy".into()),
            target: 2,
            progress: 0,
            xp_reward: 90,
            lore_reward: 10,
            rank_required: 2,
            grimoire_sections: vec![],
            sealed: false,
            realm_progress: BTreeMap::new(),
            created: String::new(),
        });
        codex.record_realm_question(&g1, Some("⚙️ Mechanics"), "mechanics again", true);
        assert!(!codex.board[1].sealed);
    }

    #[test]
    fn restoration_quest_advances_on_teaching() {
        let mut codex = Codex::default();
        let g = minecraft();
        codex.board.push(CodexQuest {
            id: "restore-test".into(),
            kind: QuestKind::Restoration,
            title: "test restore".into(),
            game: Some(g.get_name().to_string()),
            game2: None,
            category: None,
            target: 2,
            progress: 0,
            xp_reward: 100,
            lore_reward: 12,
            rank_required: 3,
            grimoire_sections: vec![],
            sealed: false,
            realm_progress: BTreeMap::new(),
            created: String::new(),
        });
        let lines = codex.record_teaching(&g);
        assert!(lines.is_empty(), "first teach advances but does not seal");
        assert_eq!(codex.board[0].progress, 1);
        let lines = codex.record_teaching(&g);
        assert!(!lines.is_empty(), "second teach seals the restoration");
        assert!(codex.board[0].sealed);
    }

    #[test]
    fn board_generation_is_rank_gated_and_fresh() {
        let kb = KnowledgeBase::new();
        let mut codex = Codex::default();
        // Level 1: only LoreMastery pins.
        codex.ensure_board(&kb, 1);
        assert!(codex.open_count() > 0);
        assert!(codex
            .board
            .iter()
            .all(|q| q.kind == QuestKind::LoreMastery));
        // Quests carry real grimoire references.
        assert!(codex
            .board
            .iter()
            .all(|q| !q.grimoire_sections.is_empty() || q.kind == QuestKind::TrialByOrdeal));
    }

    #[test]
    fn desk_commands_parse_and_log() {
        // Isolate file writes from the user's real working directory.
        let dir = std::env::temp_dir().join(format!("rocket-codex-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("ROCKET_CODEX_DIR", &dir);
        let kb = KnowledgeBase::new();
        let mut codex = Codex::default();

        let help = codex.desk_reply("help", &kb, 1);
        assert!(help.contains("Rocket, familiar of the Order"));

        let report = codex.desk_reply("where stand I", &kb, 1);
        assert!(report.contains("Scholar's report"));
        assert!(report.contains("Novice Scribe"));

        let board = codex.desk_reply("quest report", &kb, 1);
        assert!(board.contains("Quest Board") || board.contains("board stands empty"));

        // Audit a real realm.
        let audit = codex.desk_reply("audit minecraft", &kb, 1);
        assert!(audit.contains("Minecraft"), "audit matches the realm: {audit}");

        // Rank-gated generation is refused with the requirement named.
        let refused = codex.desk_reply("set me a trial", &kb, 3);
        assert!(refused.contains("Journeyman"));

        // Chronicle export writes a file (into the isolated test directory).
        let exported = codex.desk_reply("export my chronicle", &kb, 1);
        assert!(exported.contains("codex_chronicle.md"));
        assert!(base_path(CHRONICLE_FILE).exists(), "chronicle exported to the codex dir");

        // Every state-modifying desk action landed in the audit log.
        assert!(codex.actions.iter().any(|a| a.actor == "rocket" && a.action == "character_report"));
        assert!(codex.actions.iter().any(|a| a.actor == "rocket" && a.action == "chronicle_export"));
        // Desk history holds both sides of the conversation.
        assert!(codex.desk_history.iter().any(|m| m.role == "user"));
        assert!(codex.desk_history.iter().any(|m| m.role == "rocket"));
    }

    #[test]
    fn interjection_toggle_roundtrip() {
        let dir = std::env::temp_dir().join(format!("rocket-codex-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("ROCKET_CODEX_DIR", &dir);
        let kb = KnowledgeBase::new();
        let mut codex = Codex::default();
        assert!(!codex.desk_interjections);
        codex.desk_reply("interjections on", &kb, 1);
        assert!(codex.desk_interjections);
        codex.desk_reply("interjections off", &kb, 1);
        assert!(!codex.desk_interjections);
    }

    #[test]
    fn chronicle_and_logs_respect_caps() {
        let mut codex = Codex::default();
        for i in 0..300 {
            codex.log_action("order", "test", &format!("entry {i}"));
        }
        assert!(codex.actions.len() <= 200);
        assert!(codex.actions.last().unwrap().detail.contains("299"));
    }

    #[test]
    fn codex_state_roundtrips_through_json() {
        let mut codex = Codex::default();
        let g = minecraft();
        codex.record_realm_question(&g, Some("⚙️ Mechanics"), "a question about mechanics", true);
        codex.log_action("rocket", "test", "roundtrip");
        let json = serde_json::to_string(&codex).unwrap();
        let back: Codex = serde_json::from_str(&json).unwrap();
        assert_eq!(back.realms.len(), codex.realms.len());
        assert_eq!(
            back.realms.get(g.get_name()).unwrap().lore_points,
            codex.realms.get(g.get_name()).unwrap().lore_points
        );
        assert_eq!(back.actions.len(), codex.actions.len());
        assert_eq!(back.version, CODEX_VERSION);
    }
}

