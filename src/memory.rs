//! ROCKET's personal memory — the things the user teaches it about
//! themselves ("my name is…", "my favorite game is…", "remember that…").
//!
//! Everything is captured from plain chat text, persisted to
//! `rocket_memory.json`, and injected into AI prompts so answers feel
//! like they come from *your* ROCKET.

use serde::{Deserialize, Serialize};

const MEMORY_FILE: &str = "rocket_memory.json";

/// A single conversation turn — what the user asked and what ROCKET answered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub question: String,
    pub answer: String,
    pub topic: Option<String>,
    pub timestamp: i64,
}

/// Skill levels for personalized difficulty scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillLevel {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
}

impl SkillLevel {
    pub fn label(&self) -> &'static str {
        match self {
            SkillLevel::Beginner => "🟢 Beginner",
            SkillLevel::Intermediate => "🟡 Intermediate",
            SkillLevel::Advanced => "🔴 Advanced",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SkillLevel::Beginner => "Getting started — I provide step-by-step guidance",
            SkillLevel::Intermediate => "Building skills — I balance detail with assumptions",
            SkillLevel::Advanced => "Pro gamer — I give concise, advanced techniques",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub user_name: Option<String>,
    pub favorite_game: Option<String>,
    pub main_character: Option<String>,
    /// Free-form facts the user asked ROCKET to remember.
    pub facts: Vec<String>,
    /// Conversation history for context-aware responses.
    pub conversation_history: Vec<ConversationTurn>,
    /// User's skill level for personalized difficulty.
    pub skill_level: SkillLevel,
    /// Topics the user has mastered.
    pub mastered_topics: Vec<String>,
    /// Total questions answered.
    pub questions_answered: u32,
    /// Learning streak (consecutive days).
    pub learning_streak: u32,
    /// Last activity date (for streak tracking).
    pub last_activity: Option<i64>,
    /// Leveling system data.
    pub leveling: LevelingData,
    /// Day the current daily quests were rolled ("YYYY-MM-DD").
    #[serde(default)]
    pub quest_day: Option<String>,
    /// Today's active daily quests (rolled once per calendar day).
    #[serde(default)]
    pub daily_quests: Vec<DailyQuest>,
    /// Distinct games switched to today (drives the explore quest).
    #[serde(default)]
    pub games_today: Vec<String>,
    /// Profile nickname (set on the Profile page). Empty string clears it.
    #[serde(default)]
    pub nickname: Option<String>,
    /// Profile bio (set on the Profile page).
    #[serde(default)]
    pub bio: Option<String>,
    /// Path of the profile picture copied next to the app.
    #[serde(default)]
    pub profile_picture: Option<String>,
    /// Unlock state for every title in the catalog.
    #[serde(default)]
    pub titles: Vec<TitleState>,
    /// Catalog id of the equipped title, if any.
    #[serde(default)]
    pub equipped_title: Option<String>,
    /// Recent build/loadout context the user shared, newest last
    /// ("mage build", "using a bow") — used to reply like a conversation.
    #[serde(default)]
    pub loadout_context: Vec<String>,
    /// Rocket's current personality ("vibe") — see vibe.rs.
    #[serde(default)]
    pub vibe: String,
    /// Co-op missions ("beat the ender dragon") you and Rocket are on.
    #[serde(default)]
    pub missions: Vec<Mission>,
}

/// One co-op mission you and Rocket are working on together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mission {
    /// What you're beating/finishing ("the ender dragon").
    pub title: String,
    /// Game it belongs to (display name).
    pub game: String,
    #[serde(default)]
    pub done: bool,
}

/// One daily quest with live progress. Quests complete automatically and
/// pay out their XP the moment the target is reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyQuest {
    /// Event kind that drives progress: "ask" | "teach" | "explore" | "item" | "kb".
    pub kind: String,
    pub title: String,
    pub progress: u32,
    pub target: u32,
    pub xp_reward: u32,
    pub done: bool,
}

/// The daily quest pool — 3 distinct quests are rolled per day (public for docs_gen).
pub const QUEST_POOL: &[(&str, &str, u32, u32)] = &[
    // (kind, title, target, xp reward)
    ("ask", "Ask Rocket 5 questions", 5, 50),
    ("ask", "Ask Rocket 3 questions", 3, 30),
    ("chat", "Have 10 conversations with Rocket", 10, 45),
    ("teach", "Teach Rocket something new", 1, 60),
    ("explore", "Switch between 2 different games", 2, 40),
    ("item", "Ask how to get any item or gear", 1, 30),
    ("kb", "Open the Knowledge Base", 1, 25),
];

/// Leveling system data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelingData {
    pub current_level: u32,
    pub experience_points: u32,
    pub total_xp_earned: u32,
    /// Spendable currency — mirrors every XP earned (never decreases level).
    #[serde(default)]
    pub coins: u32,
    pub badges: Vec<Badge>,
    pub achievements: Vec<Achievement>,
    pub profile_decorations: Vec<Decoration>,
}

impl Default for LevelingData {
    fn default() -> Self {
        Self {
            current_level: 1,
            experience_points: 0,
            total_xp_earned: 0,
            coins: 0,
            badges: Vec::new(),
            achievements: Vec::new(),
            profile_decorations: Vec::new(),
        }
    }
}

/// Badge types that can be earned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Badge {
    /// Conversation badges
    FirstConversation,
    ConversationMaster(u32), // For reaching conversation milestones
    KnowledgeSeeker,
    FollowUpExpert,
    
    /// Learning badges
    QuickLearner,
    StreakWarrior(u32), // For streak milestones
    SkillExplorer,
    ExpertGamer,
    
    /// Special badges
    RocketFanatic,
    EarlyAdopter,
    HelpfulAssistant,
    CreativeBuilder,
    
    /// Game-specific badges
    MinecraftChampion,
    RobloxExpert,
    GameMaster,
}

impl Badge {
    /// Short human-readable label for profile display.
    pub fn label(&self) -> String {
        match self {
            Badge::FirstConversation => "💬 First Chat".to_string(),
            Badge::ConversationMaster(n) => format!("🗣️ Chatter x{}", n),
            Badge::KnowledgeSeeker => "🔍 Knowledge Seeker".to_string(),
            Badge::FollowUpExpert => "🔁 Follow-up Pro".to_string(),
            Badge::QuickLearner => "⚡ Quick Learner".to_string(),
            Badge::StreakWarrior(n) => format!("🔥 Streak x{}", n),
            Badge::SkillExplorer => "🧭 Skill Explorer".to_string(),
            Badge::ExpertGamer => "🎮 Expert Gamer".to_string(),
            Badge::RocketFanatic => "🚀 ROCKET Fanatic".to_string(),
            Badge::EarlyAdopter => "⭐ Early Adopter".to_string(),
            Badge::HelpfulAssistant => "🤝 Helper".to_string(),
            Badge::CreativeBuilder => "🏗️ Builder".to_string(),
            Badge::MinecraftChampion => "⛏️ MC Champion".to_string(),
            Badge::RobloxExpert => "🧱 Roblox Expert".to_string(),
            Badge::GameMaster => "👑 Game Master".to_string(),
        }
    }
}

/// Achievement types with descriptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub unlocked_date: Option<i64>,
    pub rarity: AchievementRarity,
}

/// Achievement rarity levels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

/// Profile decorations that can be equipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decoration {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: DecorationCategory,
    pub unlocked: bool,
    pub equipped: bool,
    pub rarity: DecorationRarity,
}

/// Decoration categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecorationCategory {
    Frame,
    Background,
    Icon,
    Animation,
    Title,
}

/// Decoration rarity levels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecorationRarity {
    Basic,
    Common,
    Rare,
    Epic,
    Legendary,
}

impl DecorationRarity {
    pub fn label(&self) -> &'static str {
        match self {
            DecorationRarity::Basic => "Basic",
            DecorationRarity::Common => "Common",
            DecorationRarity::Rare => "Rare",
            DecorationRarity::Epic => "Epic",
            DecorationRarity::Legendary => "Legendary",
        }
    }
}

/// Coin price for unlocking a decoration early in the XP shop.
pub fn decoration_price(rarity: &DecorationRarity) -> u32 {
    match rarity {
        DecorationRarity::Basic => 0,
        DecorationRarity::Common => 300,
        DecorationRarity::Rare => 800,
        DecorationRarity::Epic => 1500,
        DecorationRarity::Legendary => 3000,
    }
}

/// Cost of one mystery box.
pub const MYSTERY_BOX_PRICE: u32 = 500;

/// Level progression configuration.
pub const LEVEL_XP_REQUIREMENTS: [u32; 50] = [
    0, 100, 250, 500, 750,  // Levels 1-5
    1000, 1500, 2000, 3000, 4000,  // Levels 6-10
    5000, 6000, 7500, 9000, 11000,  // Levels 11-15
    13000, 15000, 17500, 20000, 23000,  // Levels 16-20
    26000, 29000, 32000, 35000, 38000,  // Levels 21-25
    41000, 44000, 47000, 50000, 53000,  // Levels 26-30
    56000, 59000, 62000, 65000, 68000,  // Levels 31-35
    71000, 74000, 77000, 80000, 83000,  // Levels 36-40
    86000, 89000, 92000, 95000, 98000,  // Levels 41-45
    100000, 120000, 140000, 160000, 180000,  // Levels 46-50
];

/// Rank title shown on the profile for the current level.
pub fn level_title(level: u32) -> &'static str {
    match level {
        0..=4 => "🌱 Rookie",
        5..=9 => "🛰️ Cadet",
        10..=14 => "🚀 Pilot",
        15..=19 => "✈️ Ace Pilot",
        20..=24 => "🛡️ Guardian",
        25..=29 => "⚔️ Veteran",
        30..=34 => "🏆 Elite",
        35..=39 => "👑 Champion",
        40..=44 => "🌟 Legend",
        _ => "💫 Mythic",
    }
}

// ---------------------------------------------------------------------------
// Profile titles — 104 collectible titles unlocked by playing.
// ---------------------------------------------------------------------------

/// Which stat unlocks a profile title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleCondition {
    Level(u32),
    Streak(u32),
    Questions(u32),
    Facts(u32),
    Topics(u32),
    Badges(u32),
    TotalXp(u32),
}

/// One title definition from the static catalog.
#[derive(Debug, Clone)]
pub struct TitleDef {
    pub id: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub rarity: DecorationRarity,
    pub condition: TitleCondition,
}

impl TitleDef {
    /// Human-readable unlock requirement (shown on locked titles).
    pub fn requirement(&self) -> String {
        match self.condition {
            TitleCondition::Level(n) => format!("Reach level {}", n),
            TitleCondition::Streak(n) => format!("{}-day learning streak", n),
            TitleCondition::Questions(n) => format!("Ask {} questions", n),
            TitleCondition::Facts(n) => format!("Teach Rocket {} facts", n),
            TitleCondition::Topics(n) => format!("Master {} topics", n),
            TitleCondition::Badges(n) => format!("Earn {} badges", n),
            TitleCondition::TotalXp(n) => format!("Earn {} total XP", n),
        }
    }

    pub const fn t(id: &'static str, icon: &'static str, name: &'static str, rarity: DecorationRarity, condition: TitleCondition) -> TitleDef {
        TitleDef { id, icon, name, rarity, condition }
    }
}

/// Persisted unlock state for one catalog title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleState {
    pub id: String,
    #[serde(default)]
    pub unlocked: bool,
}

/// The full title catalog — every launch deserves a rank.
pub const PROFILE_TITLES: &[TitleDef] = &[
    // --- levels 1-50: the launch ladder ---
    TitleDef::t("lvl_01", "🌱", "Fresh Launch", DecorationRarity::Basic, TitleCondition::Level(1)),
    TitleDef::t("lvl_02", "🧑‍🚀", "Ground Crew", DecorationRarity::Basic, TitleCondition::Level(2)),
    TitleDef::t("lvl_03", "⚡", "Spark Plug", DecorationRarity::Basic, TitleCondition::Level(3)),
    TitleDef::t("lvl_04", "🔌", "Fuel Line", DecorationRarity::Basic, TitleCondition::Level(4)),
    TitleDef::t("lvl_05", "🛰️", "Cadet Grad", DecorationRarity::Common, TitleCondition::Level(5)),
    TitleDef::t("lvl_06", "🧨", "Booster Stage", DecorationRarity::Common, TitleCondition::Level(6)),
    TitleDef::t("lvl_07", "☁️", "Sky Walker", DecorationRarity::Common, TitleCondition::Level(7)),
    TitleDef::t("lvl_08", "🌤️", "Cloud Piercer", DecorationRarity::Common, TitleCondition::Level(8)),
    TitleDef::t("lvl_09", "🍃", "Stratosphere", DecorationRarity::Common, TitleCondition::Level(9)),
    TitleDef::t("lvl_10", "🚀", "Orbit Cadet", DecorationRarity::Common, TitleCondition::Level(10)),
    TitleDef::t("lvl_11", "🌍", "Orbital", DecorationRarity::Common, TitleCondition::Level(11)),
    TitleDef::t("lvl_12", "🪐", "Gravity Breaker", DecorationRarity::Common, TitleCondition::Level(12)),
    TitleDef::t("lvl_13", "🌙", "Moon Walker", DecorationRarity::Common, TitleCondition::Level(13)),
    TitleDef::t("lvl_14", "🌑", "Eclipse", DecorationRarity::Rare, TitleCondition::Level(14)),
    TitleDef::t("lvl_15", "☀️", "Solar Flare", DecorationRarity::Rare, TitleCondition::Level(15)),
    TitleDef::t("lvl_16", "☄️", "Comet Chaser", DecorationRarity::Rare, TitleCondition::Level(16)),
    TitleDef::t("lvl_17", "🪨", "Asteroid Belt", DecorationRarity::Rare, TitleCondition::Level(17)),
    TitleDef::t("lvl_18", "💫", "Ring Bearer", DecorationRarity::Rare, TitleCondition::Level(18)),
    TitleDef::t("lvl_19", "🌌", "Deep Space", DecorationRarity::Rare, TitleCondition::Level(19)),
    TitleDef::t("lvl_20", "🌠", "Nebula", DecorationRarity::Rare, TitleCondition::Level(20)),
    TitleDef::t("lvl_21", "⭐", "Star Forge", DecorationRarity::Rare, TitleCondition::Level(21)),
    TitleDef::t("lvl_22", "💥", "Supernova", DecorationRarity::Rare, TitleCondition::Level(22)),
    TitleDef::t("lvl_23", "📡", "Pulsar", DecorationRarity::Rare, TitleCondition::Level(23)),
    TitleDef::t("lvl_24", "🔆", "Quasar", DecorationRarity::Rare, TitleCondition::Level(24)),
    TitleDef::t("lvl_25", "🥋", "Void Black Belt", DecorationRarity::Rare, TitleCondition::Level(25)),
    TitleDef::t("lvl_26", "🕳️", "Event Horizon", DecorationRarity::Epic, TitleCondition::Level(26)),
    TitleDef::t("lvl_27", "🛸", "Warp Pioneer", DecorationRarity::Epic, TitleCondition::Level(27)),
    TitleDef::t("lvl_28", "🔗", "Hyperlane", DecorationRarity::Epic, TitleCondition::Level(28)),
    TitleDef::t("lvl_29", "🏃", "Void Runner", DecorationRarity::Epic, TitleCondition::Level(29)),
    TitleDef::t("lvl_30", "⚓", "Starship Captain", DecorationRarity::Epic, TitleCondition::Level(30)),
    TitleDef::t("lvl_31", "🎖️", "Fleet Commander", DecorationRarity::Epic, TitleCondition::Level(31)),
    TitleDef::t("lvl_32", "🛎️", "Station Master", DecorationRarity::Epic, TitleCondition::Level(32)),
    TitleDef::t("lvl_33", "🏗️", "Planet Crafter", DecorationRarity::Epic, TitleCondition::Level(33)),
    TitleDef::t("lvl_34", "🌎", "Terraformer", DecorationRarity::Epic, TitleCondition::Level(34)),
    TitleDef::t("lvl_35", "🏛️", "Galactic Senator", DecorationRarity::Epic, TitleCondition::Level(35)),
    TitleDef::t("lvl_36", "🛡️", "Cosmic Warden", DecorationRarity::Epic, TitleCondition::Level(36)),
    TitleDef::t("lvl_37", "⚔️", "Astral Knight", DecorationRarity::Epic, TitleCondition::Level(37)),
    TitleDef::t("lvl_38", "🐉", "Comet Lord", DecorationRarity::Epic, TitleCondition::Level(38)),
    TitleDef::t("lvl_39", "🔥", "Nova Commander", DecorationRarity::Epic, TitleCondition::Level(39)),
    TitleDef::t("lvl_40", "📜", "Mythmaker", DecorationRarity::Legendary, TitleCondition::Level(40)),
    TitleDef::t("lvl_41", "🌅", "Dawnbringer", DecorationRarity::Legendary, TitleCondition::Level(41)),
    TitleDef::t("lvl_42", "🌟", "Legend Ascendant", DecorationRarity::Legendary, TitleCondition::Level(42)),
    TitleDef::t("lvl_43", "👑", "Star Sovereign", DecorationRarity::Legendary, TitleCondition::Level(43)),
    TitleDef::t("lvl_44", "🎆", "Galactic Legend", DecorationRarity::Legendary, TitleCondition::Level(44)),
    TitleDef::t("lvl_45", "🌌", "Nebula Overlord", DecorationRarity::Legendary, TitleCondition::Level(45)),
    TitleDef::t("lvl_46", "🗿", "Cosmic Titan", DecorationRarity::Legendary, TitleCondition::Level(46)),
    TitleDef::t("lvl_47", "✨", "Universe Shaper", DecorationRarity::Legendary, TitleCondition::Level(47)),
    TitleDef::t("lvl_48", "⚡", "Singularity", DecorationRarity::Legendary, TitleCondition::Level(48)),
    TitleDef::t("lvl_49", "🛩️", "Omega Pilot", DecorationRarity::Legendary, TitleCondition::Level(49)),
    TitleDef::t("lvl_50", "💫", "Beyond the Stars", DecorationRarity::Legendary, TitleCondition::Level(50)),
    // --- streaks: consistency pays ---
    TitleDef::t("str_02", "🔥", "Warming Up", DecorationRarity::Basic, TitleCondition::Streak(2)),
    TitleDef::t("str_03", "🌶️", "On Fire", DecorationRarity::Basic, TitleCondition::Streak(3)),
    TitleDef::t("str_04", "🧯", "Unburnable", DecorationRarity::Common, TitleCondition::Streak(4)),
    TitleDef::t("str_05", "🏅", "Streaker", DecorationRarity::Common, TitleCondition::Streak(5)),
    TitleDef::t("str_07", "🗓️", "Full Week Flame", DecorationRarity::Common, TitleCondition::Streak(7)),
    TitleDef::t("str_10", "🌋", "Blazing", DecorationRarity::Rare, TitleCondition::Streak(10)),
    TitleDef::t("str_14", "🧨", "Fortnight Fire", DecorationRarity::Rare, TitleCondition::Streak(14)),
    TitleDef::t("str_21", "⚙️", "Habit Engine", DecorationRarity::Rare, TitleCondition::Streak(21)),
    TitleDef::t("str_30", "🌙", "Moonlong Burn", DecorationRarity::Epic, TitleCondition::Streak(30)),
    TitleDef::t("str_50", "🗿", "Immovable", DecorationRarity::Epic, TitleCondition::Streak(50)),
    TitleDef::t("str_100", "💯", "Century Flame", DecorationRarity::Epic, TitleCondition::Streak(100)),
    TitleDef::t("str_365", "🎆", "Year of Fire", DecorationRarity::Legendary, TitleCondition::Streak(365)),
    // --- questions: curiosity rewarded ---
    TitleDef::t("q_001", "❓", "Curious Mind", DecorationRarity::Basic, TitleCondition::Questions(1)),
    TitleDef::t("q_003", "👋", "Warm Welcome", DecorationRarity::Basic, TitleCondition::Questions(3)),
    TitleDef::t("q_005", "🙋", "Question Asker", DecorationRarity::Basic, TitleCondition::Questions(5)),
    TitleDef::t("q_010", "🎯", "Quiz Cadet", DecorationRarity::Common, TitleCondition::Questions(10)),
    TitleDef::t("q_025", "🕵️", "Answer Hunter", DecorationRarity::Common, TitleCondition::Questions(25)),
    TitleDef::t("q_050", "🧠", "Curiosity Engine", DecorationRarity::Common, TitleCondition::Questions(50)),
    TitleDef::t("q_100", "💯", "Hundred Club", DecorationRarity::Rare, TitleCondition::Questions(100)),
    TitleDef::t("q_150", "🎓", "Question Scholar", DecorationRarity::Rare, TitleCondition::Questions(150)),
    TitleDef::t("q_200", "🤿", "Deep Diver", DecorationRarity::Rare, TitleCondition::Questions(200)),
    TitleDef::t("q_500", "🧐", "Grand Inquisitor", DecorationRarity::Epic, TitleCondition::Questions(500)),
    TitleDef::t("q_1000", "🔎", "Millennium Mind", DecorationRarity::Epic, TitleCondition::Questions(1000)),
    TitleDef::t("q_2500", "🌠", "Curiosity Incarnate", DecorationRarity::Legendary, TitleCondition::Questions(2500)),
    // --- facts taught: teacher's titles ---
    TitleDef::t("f_001", "🧠", "First Lesson", DecorationRarity::Basic, TitleCondition::Facts(1)),
    TitleDef::t("f_003", "🍎", "Teacher's Pet", DecorationRarity::Basic, TitleCondition::Facts(3)),
    TitleDef::t("f_005", "📖", "Mentor", DecorationRarity::Common, TitleCondition::Facts(5)),
    TitleDef::t("f_010", "🎓", "Professor", DecorationRarity::Common, TitleCondition::Facts(10)),
    TitleDef::t("f_020", "📚", "Lore Keeper", DecorationRarity::Rare, TitleCondition::Facts(20)),
    TitleDef::t("f_030", "🧬", "Mind Architect", DecorationRarity::Rare, TitleCondition::Facts(30)),
    TitleDef::t("f_040", "🗂️", "Archive Keeper", DecorationRarity::Rare, TitleCondition::Facts(40)),
    TitleDef::t("f_050", "🏛️", "Grand Archivist", DecorationRarity::Epic, TitleCondition::Facts(50)),
    TitleDef::t("f_060", "🧿", "Oracle", DecorationRarity::Epic, TitleCondition::Facts(60)),
    TitleDef::t("f_075", "📕", "Living Encyclopedia", DecorationRarity::Legendary, TitleCondition::Facts(75)),
    // --- topics mastered: polymath path ---
    TitleDef::t("t_001", "🥉", "Specialist", DecorationRarity::Common, TitleCondition::Topics(1)),
    TitleDef::t("t_003", "🥈", "Polymath", DecorationRarity::Common, TitleCondition::Topics(3)),
    TitleDef::t("t_005", "🥇", "Mastermind", DecorationRarity::Rare, TitleCondition::Topics(5)),
    TitleDef::t("t_010", "💎", "Omniscient", DecorationRarity::Rare, TitleCondition::Topics(10)),
    TitleDef::t("t_015", "🔮", "Oracle of Topics", DecorationRarity::Epic, TitleCondition::Topics(15)),
    TitleDef::t("t_020", "🌐", "Renaissance Gamer", DecorationRarity::Epic, TitleCondition::Topics(20)),
    TitleDef::t("t_030", "🧿", "Polymath Prime", DecorationRarity::Legendary, TitleCondition::Topics(30)),
    // --- badges: decoration of dedication ---
    TitleDef::t("b_001", "🎖️", "Badge Earner", DecorationRarity::Common, TitleCondition::Badges(1)),
    TitleDef::t("b_003", "🧷", "Collector", DecorationRarity::Common, TitleCondition::Badges(3)),
    TitleDef::t("b_005", "🏹", "Badge Hunter", DecorationRarity::Rare, TitleCondition::Badges(5)),
    TitleDef::t("b_008", "🎩", "Badge Baron", DecorationRarity::Epic, TitleCondition::Badges(8)),
    // --- total XP: the grind ---
    TitleDef::t("xp_0250", "🪙", "Early Energy", DecorationRarity::Basic, TitleCondition::TotalXp(250)),
    TitleDef::t("xp_1000", "🌾", "One Grind", DecorationRarity::Basic, TitleCondition::TotalXp(1000)),
    TitleDef::t("xp_5000", "💰", "XP Farmer", DecorationRarity::Common, TitleCondition::TotalXp(5000)),
    TitleDef::t("xp_10000", "🔟", "Ten Thousand", DecorationRarity::Common, TitleCondition::TotalXp(10000)),
    TitleDef::t("xp_25000", "🏦", "XP Tycoon", DecorationRarity::Rare, TitleCondition::TotalXp(25000)),
    TitleDef::t("xp_50000", "💎", "XP Baron", DecorationRarity::Rare, TitleCondition::TotalXp(50000)),
    TitleDef::t("xp_75000", "👑", "XP Magnate", DecorationRarity::Epic, TitleCondition::TotalXp(75000)),
    TitleDef::t("xp_100000", "🏆", "XP Sovereign", DecorationRarity::Epic, TitleCondition::TotalXp(100000)),
    TitleDef::t("xp_150000", "🌠", "XP Deity", DecorationRarity::Legendary, TitleCondition::TotalXp(150000)),
    TitleDef::t("xp_180000", "💫", "Max Grind", DecorationRarity::Legendary, TitleCondition::TotalXp(180000)),
];

/// Ensure every catalog title exists in the saved state, then unlock any
/// whose condition is now met. Returns one toast line per new unlock.
impl Memory {
    pub fn sync_titles(&mut self) -> Vec<String> {
        // Backfill catalog entries added after this memory was saved.
        for def in PROFILE_TITLES {
            if !self.titles.iter().any(|t| t.id == def.id) {
                self.titles.push(TitleState {
                    id: def.id.to_string(),
                    unlocked: false,
                });
            }
        }

        let level = self.leveling.current_level;
        let streak = self.learning_streak;
        let questions = self.questions_answered;
        let facts = self.facts.len() as u32;
        let topics = self.mastered_topics.len() as u32;
        let badges = self.leveling.badges.len() as u32;
        let xp = self.leveling.total_xp_earned;
        let met = |c: TitleCondition| match c {
            TitleCondition::Level(n) => level >= n,
            TitleCondition::Streak(n) => streak >= n,
            TitleCondition::Questions(n) => questions >= n,
            TitleCondition::Facts(n) => facts >= n,
            TitleCondition::Topics(n) => topics >= n,
            TitleCondition::Badges(n) => badges >= n,
            TitleCondition::TotalXp(n) => xp >= n,
        };

        let mut lines = Vec::new();
        for def in PROFILE_TITLES {
            if let Some(state) = self.titles.iter_mut().find(|t| t.id == def.id) {
                if !state.unlocked && met(def.condition) {
                    state.unlocked = true;
                    lines.push(format!("🎖️ Title unlocked: {} {}", def.icon, def.name));
                }
            }
        }
        lines
    }

    /// Equip an unlocked title. Equipping the already-equipped one clears it.
    pub fn equip_title(&mut self, id: &str) {
        let unlocked = self
            .titles
            .iter()
            .any(|t| t.id == id && t.unlocked);
        if unlocked {
            self.equipped_title = if self.equipped_title.as_deref() == Some(id) {
                None
            } else {
                Some(id.to_string())
            };
        }
    }

    /// (icon, name) of the equipped title, if any is set.
    pub fn equipped_title_display(&self) -> Option<(String, String)> {
        let id = self.equipped_title.as_deref()?;
        PROFILE_TITLES
            .iter()
            .find(|d| d.id == id)
            .map(|d| (d.icon.to_string(), d.name.to_string()))
    }

    /// (unlocked count, total catalog size).
    pub fn title_progress(&self) -> (usize, usize) {
        (
            self.titles.iter().filter(|t| t.unlocked).count(),
            PROFILE_TITLES.len(),
        )
    }

    /// Set/clear the profile nickname (empty string clears it).
    pub fn set_nickname(&mut self, value: &str) {
        let trimmed = value.trim();
        self.nickname = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.chars().take(24).collect())
        };
    }

    /// Set/clear the profile bio (empty string clears it).
    pub fn set_bio(&mut self, value: &str) {
        let trimmed = value.trim();
        self.bio = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.chars().take(160).collect())
        };
    }

    /// Copy a picked image next to the app and remember it as the profile
    /// picture. Returns an IO error when the file can't be read/copied.
    pub fn set_profile_picture_from_path(&mut self, source: &std::path::Path) -> std::io::Result<()> {
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_else(|| "png".to_string());
        let dest = std::path::PathBuf::from(format!("rocket-profile-picture.{}", ext));
        std::fs::copy(source, &dest)?;
        self.profile_picture = Some(dest.to_string_lossy().to_string());
        Ok(())
    }

    /// Remove the profile picture (both the copied file and the setting).
    pub fn clear_profile_picture(&mut self) {
        if let Some(p) = &self.profile_picture {
            let _ = std::fs::remove_file(p);
        }
        self.profile_picture = None;
    }

    /// Remember build/loadout self-statements ("i'm going for a mage build",
    /// "i'm using a bow") and return a short conversational acknowledgment.
    /// Returns None when the message isn't self-context talk.
    pub fn capture_build_talk(&mut self, message: &str) -> Option<String> {
        let lower = message.trim().to_lowercase();
        const LEADS: &[&str] = &[
            "im going for a ", "i am going for a ", "im going for ", "im going ",
            "im using ", "i am using ", "i use ", "i main ", "im maining ",
            "im running ", "im playing ", "i play ", "my build is ", "im doing a ",
            "im doing ", "im a ", "i am a ", "im building ",
        ];
        let mut ctx: Option<String> = None;
        for lead in LEADS {
            if let Some(rest) = lower.strip_prefix(lead) {
                // The context is just the first clause — stop at commas,
                // "and", "then", or where the question part begins.
                let clause = rest
                    .split([',', ';', '?', '!'])
                    .next()
                    .unwrap_or(rest);
                let clause = clause
                    .split(" and ")
                    .next()
                    .unwrap_or(clause);
                let clause = clause
                    .split(" then ")
                    .next()
                    .unwrap_or(clause);
                let clause = match clause.find(" what").or_else(|| clause.find(" how ")) {
                    Some(i) => &clause[..i],
                    None => clause,
                };
                let clause = clause.trim().trim_end_matches(['.', '!', '?']).trim();
                if !clause.is_empty() && clause.len() <= 48 {
                    ctx = Some(clause.to_string());
                    break;
                }
            }
        }
        let ctx = ctx?;

        // Remember it — most recent last, deduped, capped.
        self.loadout_context.retain(|c| c != &ctx);
        self.loadout_context.push(ctx.clone());
        if self.loadout_context.len() > 5 {
            self.loadout_context.remove(0);
        }

        // Acknowledge it with a little personality.
        let emoji = if ctx.contains("mage") || ctx.contains("wizard") || ctx.contains("sorcer") {
            "🧙"
        } else if ctx.contains("stealth") || ctx.contains("archer") || ctx.contains("bow")
            || ctx.contains("sniper") || ctx.contains("rogue")
        {
            "🏹"
        } else if ctx.contains("tank") || ctx.contains("warrior") || ctx.contains("knight")
            || ctx.contains("paladin")
        {
            "🛡️"
        } else if ctx.contains("heal") || ctx.contains("support") || ctx.contains("cleric") {
            "💚"
        } else if ctx.contains("melee") || ctx.contains("berserk") {
            "⚔️"
        } else if ctx.contains("necro") || ctx.contains("summon") {
            "💀"
        } else {
            "🎯"
        };
        Some(format!(
            "{} {} — noted, I'll keep that in mind for you.",
            emoji,
            title_case(&ctx)
        ))
    }

    /// The most recent build/loadout context the user shared, if any.
    pub fn context_hint(&self) -> Option<&str> {
        self.loadout_context.last().map(|c| c.as_str())
    }

    // -- co-op missions ------------------------------------------------------

    /// Add a mission to the co-op board (deduped against open missions).
    /// Returns false when it's already on the board or the board is full.
    pub fn create_mission(&mut self, title: &str, game: &str) -> bool {
        let title = title.trim().trim_end_matches(['.', '!', '?']).trim();
        if title.is_empty() {
            return false;
        }
        if self.missions.iter().any(|m| {
            m.done == false && m.title.eq_ignore_ascii_case(title) && m.game == game
        }) {
            return false;
        }
        let open: usize = self.missions.iter().filter(|m| !m.done).count();
        if open >= 5 {
            return false;
        }
        self.missions.push(Mission {
            title: title.to_string(),
            game: game.to_string(),
            done: false,
        });
        true
    }

    /// Mark the open mission matching `text` ("i finally beat the ender
    /// dragon") as done. Returns the completed mission, if any.
    pub fn complete_mission_by_text(&mut self, text: &str) -> Option<Mission> {
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| w.len() > 2)
            .collect();
        let index = self.missions.iter().position(|m| {
            !m.done
                && m.title
                    .to_lowercase()
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|w| w.len() > 2)
                    .any(|w| tokens.contains(&w))
        })?;
        self.missions[index].done = true;
        Some(self.missions[index].clone())
    }

    /// Open (not yet completed) missions.
    pub fn active_missions(&self) -> Vec<&Mission> {
        self.missions.iter().filter(|m| !m.done).collect()
    }

    // -- vibe ----------------------------------------------------------------

    /// Switch Rocket's personality. Returns the vibe's intro line.
    pub fn set_vibe(&mut self, id: &str) -> String {
        self.vibe = id.to_string();
        crate::vibe::vibe_by_id(id).intro.to_string()
    }

    /// The currently active vibe.
    pub fn current_vibe(&self) -> &'static crate::vibe::Vibe {
        crate::vibe::vibe_by_id(&self.vibe)
    }
}

/// Parse "let's beat X" / "help me beat X" style mission requests.
/// Returns the mission target ("the ender dragon").
pub fn parse_mission_creation(message: &str) -> Option<String> {
    let lower = message.trim().to_lowercase();
    const TRIGGERS: &[&str] = &[
        "lets 100%", "let's 100%", "lets beat", "let's beat", "help me beat",
        "i want to beat", "we should beat", "going to beat", "tryna beat",
        "trying to beat", "we are gonna beat", "im gonna beat", "help me finish",
        "lets finish", "let's finish", "i want to finish", "help me complete",
        "lets complete", "let's complete", "i want to complete", "help me get through",
        "help me with the", "lets do the", "let's do the", "lets take down",
        "let's take down", "i want to 100%",
    ];
    let trigger = TRIGGERS.iter().find(|t| lower.contains(*t))?;
    let after = &lower[lower.find(trigger)? + trigger.len()..];
    let mut target: String = after
        .split([',', ';', '?', '!', '.'])
        .next()
        .unwrap_or(after)
        .trim()
        .to_string();
    // Drop leading articles: "the ender dragon" → "ender dragon".
    for prefix in ["the ", "a ", "my first "] {
        if let Some(stripped) = target.strip_prefix(prefix) {
            target = stripped.trim().to_string();
            break;
        }
    }
    let target = target.trim().to_string();
    if target.len() < 3 || target.len() > 60 {
        return None;
    }
    // Questions about beating something are advice requests, not missions.
    if lower.contains("how do") || lower.contains("how can") || lower.contains("how to") {
        return None;
    }
    Some(target)
}

/// Parse "i beat X" / "we did it" style completion reports.
/// Returns Some(()) when the message reports a genuine win — not a request.
pub fn parse_mission_completion(message: &str) -> Option<()> {
    let lower = message.trim().to_lowercase();
    if message_is_question(&lower) {
        return None;
    }
    // Requests ("let's beat the dragon") are NOT wins.
    if lower.starts_with("let")
        || lower.starts_with("help")
        || lower.starts_with("how")
        || lower.starts_with("we should")
        || parse_mission_creation(message).is_some()
    {
        return None;
    }
    const WINS: &[&str] = &[
        "i beat", "we beat", "i just beat", "we just beat", "i finally beat",
        "we finally beat", "i finished", "we finished", "i just finished",
        "i completed", "we completed", "i did it", "we did it", "we won", "i won",
    ];
    WINS.iter().any(|w| lower.contains(w)).then_some(())
}

/// True when the message is asking something (vs just sharing info).
pub fn message_is_question(message: &str) -> bool {
    let lower = message.trim().to_lowercase();
    if lower.contains('?') {
        return true;
    }
    const QUESTION_STARTS: &[&str] = &[
        "what", "whats", "what s", "how", "why", "when", "where", "which", "who",
        "should", "can", "could", "is", "are", "do", "does", "did", "any", "will",
    ];
    if let Some(first) = lower.split_whitespace().next() {
        if QUESTION_STARTS.contains(&first) {
            return true;
        }
    }
    [" what s ", " whats ", " how ", " why "].iter().any(|p| {
        let padded = format!(" {}", lower);
        padded.contains(p)
    })
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            user_name: None,
            favorite_game: None,
            main_character: None,
            facts: Vec::new(),
            conversation_history: Vec::new(),
            skill_level: SkillLevel::Beginner,
            mastered_topics: Vec::new(),
            questions_answered: 0,
            learning_streak: 0,
            last_activity: None,
            leveling: LevelingData::default(),
            quest_day: None,
            daily_quests: Vec::new(),
            games_today: Vec::new(),
            nickname: None,
            bio: None,
            profile_picture: None,
            titles: Vec::new(),
            equipped_title: None,
            loadout_context: Vec::new(),
            vibe: String::new(),
            missions: Vec::new(),
        }
    }
}

impl Memory {
    pub fn load() -> Self {
        let mut memory = std::fs::read(MEMORY_FILE)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Memory>(&bytes).ok())
            .unwrap_or_default();
        // Backfill title catalog entries added after this memory was saved.
        memory.sync_titles();
        memory
    }

    pub fn save(&self) {
        if let Ok(bytes) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(MEMORY_FILE, bytes);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.user_name.is_none()
            && self.favorite_game.is_none()
            && self.main_character.is_none()
            && self.facts.is_empty()
            && self.conversation_history.is_empty()
    }

    pub fn forget_everything(&mut self) {
        *self = Memory::default();
    }

    /// Try to learn a personal fact from a chat message.
    /// Returns a confirmation reply when something was learned.
    pub fn capture(&mut self, text: &str) -> Option<String> {
        let lower = text.trim().to_lowercase();
        let lower = lower.trim_end_matches(['.', '!', '?']).trim().to_string();

        // Forgetting wins first so it can't be captured as a fact.
        if lower == "forget everything" || lower == "forget all" {
            self.forget_everything();
            return Some("🧹 Done — I forgot everything you taught me. Fresh start!".to_string());
        }

        if let Some(name) = strip_prefix(&lower, &["my name is", "call me", "i am called"]) {
            let name = title_case(&name);
            if valid_name(&name) {
                self.user_name = Some(name.clone());
                return Some(format!(
                    "Nice to meet you, {}! 🧠 I'll remember that.",
                    name
                ));
            }
        }

        if let Some(game) = strip_prefix(&lower, &["my favorite game is", "my favourite game is"]) {
            let game = title_case(&game);
            if valid_name(&game) {
                self.favorite_game = Some(game.clone());
                return Some(format!(
                    "🧠 Noted — {} is your favorite game. I'll keep that in mind!",
                    game
                ));
            }
        }

        if let Some(main) = strip_prefix(&lower, &["my main is", "i main"]) {
            let main = title_case(&main);
            if valid_name(&main) {
                self.main_character = Some(main.clone());
                return Some(format!(
                    "🧠 Got it — {} is your main. Expect main-specific advice!",
                    main
                ));
            }
        }

        // Capture skill level — the phrase may be the entire message, so a
        // plain starts_with check (strip_prefix requires a non-empty rest).
        if ["i'm a beginner", "i'm a beginner at", "i'm new", "i'm a beginner gamer"]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            self.skill_level = SkillLevel::Beginner;
            return Some("🧠 Got it — you're a beginner. I'll give you step-by-step guidance!".to_string());
        }

        if ["i'm intermediate", "i'm pretty good", "i know what i'm doing", "i'm intermediate at"]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            self.skill_level = SkillLevel::Intermediate;
            return Some("🧠 Got it — you're intermediate level. I'll balance detail with some assumptions!".to_string());
        }

        if ["i'm advanced", "i'm a pro", "i'm advanced at", "i'm an expert"]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            self.skill_level = SkillLevel::Advanced;
            return Some("🧠 Got it — you're advanced! I'll give you concise, pro-level tips.".to_string());
        }

        if let Some(fact) = strip_prefix(&lower, &["remember that", "remember:"]) {
            let fact = fact.trim();
            if fact.len() >= 3 && fact.len() <= 120 {
                let fact = title_case_sentence(fact);
                if !self.facts.iter().any(|f| f.eq_ignore_ascii_case(&fact)) {
                    self.facts.push(fact.clone());
                    return Some(format!("🧠 Remembered: {}", fact));
                }
                return Some("🧠 I already knew that!".to_string());
            }
        }

        None
    }

    /// Add a conversation turn to history.
    pub fn add_conversation(&mut self, question: String, answer: String, topic: Option<String>) -> (bool, u32) {
        let turn = ConversationTurn {
            question,
            answer,
            topic,
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.conversation_history.push(turn);

        // Keep only last 50 conversations to avoid unbounded growth
        if self.conversation_history.len() > 50 {
            self.conversation_history.remove(0);
        }

        // Increment questions answered counter
        self.questions_answered += 1;

        // Award experience points for conversation
        let (leveled_up, levels_gained) = self.leveling.add_experience(10);
        
        // Check for achievements and badges
        if self.questions_answered == 1 {
            self.leveling.award_achievement(Achievement {
                id: "first_conversation".to_string(),
                title: "First Conversation".to_string(),
                description: "Have your first conversation with ROCKET".to_string(),
                icon: "💬".to_string(),
                unlocked: false,
                unlocked_date: None,
                rarity: AchievementRarity::Common,
            });
            self.leveling.award_badge(Badge::FirstConversation);
        }
        
        if self.questions_answered == 10 {
            self.leveling.award_achievement(Achievement {
                id: "quick_learner".to_string(),
                title: "Quick Learner".to_string(),
                description: "Answer 10 questions correctly".to_string(),
                icon: "🚀".to_string(),
                unlocked: false,
                unlocked_date: None,
                rarity: AchievementRarity::Common,
            });
            self.leveling.award_badge(Badge::QuickLearner);
        }
        
        if self.conversation_history.len() == 50 {
            self.leveling.award_badge(Badge::ConversationMaster(50));
        }

        // Update activity date for streak tracking
        self.last_activity = Some(chrono::Utc::now().timestamp());
        
        // Return leveling status for UI feedback
        (leveled_up, levels_gained)
    }
    
    /// Award experience for teaching ROCKET something new
    pub fn award_teaching_xp(&mut self) -> (bool, u32) {
        let (leveled_up, levels_gained) = self.leveling.add_experience(25);
        
        // Award knowledge seeker badge
        if !self.leveling.has_badge(&Badge::KnowledgeSeeker) {
            self.leveling.award_badge(Badge::KnowledgeSeeker);
        }
        
        (leveled_up, levels_gained)
    }
    
    /// Award experience for following up on topics
    pub fn award_followup_xp(&mut self) -> (bool, u32) {
        let (leveled_up, levels_gained) = self.leveling.add_experience(15);
        
        // Award follow-up expert badge on 5th follow-up
        let followup_count = self.conversation_history.iter()
            .filter(|turn| turn.question.to_lowercase().contains("more") || 
                         turn.question.to_lowercase().contains("additional") ||
                         turn.question.to_lowercase().contains("another"))
            .count();
        
        if followup_count >= 5 && !self.leveling.has_badge(&Badge::FollowUpExpert) {
            self.leveling.award_badge(Badge::FollowUpExpert);
        }
        
        (leveled_up, levels_gained)
    }
    
    /// Award experience for maintaining learning streaks
    pub fn award_streak_xp(&mut self, streak_days: u32) -> (bool, u32) {
        let xp_amount = match streak_days {
            1..=3 => 5,
            4..=7 => 15,
            8..=14 => 30,
            15..=30 => 50,
            _ => 75,
        };
        
        let (leveled_up, levels_gained) = self.leveling.add_experience(xp_amount);
        
        // Award streak warrior badges for milestones
        if streak_days == 7 {
            self.leveling.award_badge(Badge::StreakWarrior(7));
            self.leveling.award_achievement(Achievement {
                id: "streak_warrior".to_string(),
                title: "Streak Warrior".to_string(),
                description: "Maintain a 7-day learning streak".to_string(),
                icon: "🔥".to_string(),
                unlocked: false,
                unlocked_date: None,
                rarity: AchievementRarity::Rare,
            });
        }
        
        if streak_days == 30 {
            self.leveling.award_badge(Badge::StreakWarrior(30));
        }
        
        (leveled_up, levels_gained)
    }
    
    /// Award experience for mastering topics
    pub fn award_mastery_xp(&mut self, topic: &str) -> (bool, u32) {
        // Add topic to mastered topics if not already there
        if !self.mastered_topics.contains(&topic.to_string()) {
            self.mastered_topics.push(topic.to_string());
            
            let xp_amount = 20;
            let (leveled_up, levels_gained) = self.leveling.add_experience(xp_amount);
            
            // Award skill explorer badge for first mastery
            if self.mastered_topics.len() == 1 {
                self.leveling.award_badge(Badge::SkillExplorer);
            }
            
            // Award achievement for mastering 5 topics
            if self.mastered_topics.len() == 5 {
                self.leveling.award_achievement(Achievement {
                    id: "knowledge_seeker".to_string(),
                    title: "Knowledge Seeker".to_string(),
                    description: "Ask about 5 different topics".to_string(),
                    icon: "🔍".to_string(),
                    unlocked: false,
                    unlocked_date: None,
                    rarity: AchievementRarity::Common,
                });
            }
            
            (leveled_up, levels_gained)
        } else {
            (false, 0)
        }
    }
    
    /// Award experience for switching games (encourages exploration)
    pub fn award_game_exploration_xp(&mut self) -> (bool, u32) {
        let (leveled_up, levels_gained) = self.leveling.add_experience(5);
        
        // Award game master badge for exploring multiple games
        if self.games_played() >= 3 {
            self.leveling.award_badge(Badge::GameMaster);
        }
        
        (leveled_up, levels_gained)
    }
    
    /// Award special badges for milestones
    pub fn check_milestone_badges(&mut self) {
        let level = self.leveling.current_level;
        
        // Level-based badges
        if level == 5 {
            self.leveling.award_badge(Badge::ExpertGamer);
        }
        
        if level == 10 {
            self.leveling.award_achievement(Achievement {
                id: "rocket_fanatic".to_string(),
                title: "ROCKET Fanatic".to_string(),
                description: "Reach level 10".to_string(),
                icon: "🚀".to_string(),
                unlocked: false,
                unlocked_date: None,
                rarity: AchievementRarity::Epic,
            });
        }
        
        if level == 25 {
            self.leveling.award_badge(Badge::CreativeBuilder);
        }
        
        // Conversation milestone badges
        if self.questions_answered == 50 {
            self.leveling.award_achievement(Achievement {
                id: "conversation_master".to_string(),
                title: "Conversation Master".to_string(),
                description: "Have 50 conversations with ROCKET".to_string(),
                icon: "🗣️".to_string(),
                unlocked: false,
                unlocked_date: None,
                rarity: AchievementRarity::Rare,
            });
        }
    }
    
    /// Get games played count (based on game switching)
    fn games_played(&self) -> u32 {
        // This would track how many different games the user has interacted with
        // For now, we'll use a simple approximation based on mastered topics
        (self.mastered_topics.len() / 3) as u32 + 1
    }

    /// Get the last N conversation turns with topic context.
    pub fn get_recent_context(&self, count: usize) -> Vec<&ConversationTurn> {
        self.conversation_history
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get a summary of recent conversations.
    pub fn get_conversation_summary(&self) -> String {
        if self.conversation_history.is_empty() {
            return String::new();
        }

        let recent = self.get_recent_context(5);
        let mut lines = vec![String::from("\n\nRecent conversations:")];

        for turn in recent.iter().rev() {
            let topic_label = turn.topic.as_deref().unwrap_or("General");
            let time_ago = format_ago(turn.timestamp);
            lines.push(format!("- Asked about {}: {} ({})", topic_label, turn.question, time_ago));
        }

        lines.join("\n")
    }

    /// A prompt block describing everything ROCKET knows about the user.
    /// Empty string when nothing has been learned yet.
    pub fn recall_context(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut lines = vec![String::from("\n\nPersonal facts you know about the user:")];
        if let Some(name) = &self.user_name {
            lines.push(format!("- Their name is {} — greet them by name when it fits.", name));
        }
        if let Some(game) = &self.favorite_game {
            lines.push(format!("- Their favorite game is {}.", game));
        }
        if let Some(main) = &self.main_character {
            lines.push(format!("- Their main character is {}.", main));
        }
        for fact in &self.facts {
            lines.push(format!("- {}", fact));
        }
        lines.push(format!("- Their skill level is: {}", self.skill_level.label()));
        lines.push(format!("\n{} — {}", self.skill_level.label(), self.skill_level.description()));
        lines.join("\n")
    }
}

/// Match any of the prefixes and return the remainder, if non-empty.
fn strip_prefix(text: &str, prefixes: &[&str]) -> Option<String> {
    for p in prefixes {
        if let Some(rest) = text.strip_prefix(p) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// Plausible name/topic: short, no newlines, not obviously a sentence.
fn valid_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 40 && !s.contains('\n')
}

/// Capitalize each word's first letter (for names/games/characters).
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Capitalize just the first letter (for full sentences).
fn title_case_sentence(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Format a timestamp as "X ago" (e.g., "2 minutes ago").
fn format_ago(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else {
        format!("{} weeks ago", diff / 604800)
    }
}

/// Export memory to JSON file.
pub fn export_to_json(memory: &Memory, filepath: &str) -> Result<(), String> {
    if let Ok(bytes) = serde_json::to_vec_pretty(memory) {
        std::fs::write(filepath, bytes)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    } else {
        Err("Failed to serialize memory data".to_string())
    }
}

/// Import memory from JSON file.
pub fn import_from_json(filepath: &str) -> Result<Memory, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Export only custom games and facts (without personal data).
pub fn export_custom_games(memory: &Memory) -> Result<String, String> {
    let custom_data = serde_json::json!({
        "facts": memory.facts,
        "conversation_history": memory.conversation_history,
        "mastered_topics": memory.mastered_topics,
        "export_date": chrono::Utc::now().to_rfc3339(),
        "version": "1.0"
    });
    
    serde_json::to_string_pretty(&custom_data)
        .map_err(|e| format!("Failed to serialize custom data: {}", e))
}

/// Import custom games and facts.
pub fn import_custom_games(json_str: &str) -> Result<(), String> {
    let data: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    if let Some(facts) = data.get("facts").and_then(|f| f.as_array()) {
        // This would require modifying the Memory struct to have a separate import method
        // For now, we'll just validate the format
        if facts.len() > 100 {
            return Err("Too many facts to import (max 100)".to_string());
        }
    }
    
    Ok(())
}

/// Create a timestamped backup of the memory file.
pub fn create_backup() -> Result<String, String> {
    let memory = Memory::load();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("rocket_memory_backup_{}.json", timestamp);
    
    export_to_json(&memory, &backup_path)?;
    Ok(backup_path)
}

/// Check if the user has been active today (for streak tracking).
pub fn is_active_today(memory: &Memory) -> bool {
    if let Some(last_activity) = memory.last_activity {
        let last_date = chrono::DateTime::from_timestamp(last_activity, 0)
            .map(|dt| dt.date_naive())
            .unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        
        let today = chrono::Local::now().date_naive();
        last_date == today
    } else {
        false
    }
}

/// Update learning streak based on activity.
pub fn update_streak(memory: &mut Memory) -> u32 {
    let today = chrono::Local::now().date_naive();
    let yesterday = today.pred_opt().unwrap_or(today);
    
    if let Some(last_activity) = memory.last_activity {
        let last_date = chrono::DateTime::from_timestamp(last_activity, 0)
            .map(|dt| dt.date_naive())
            .unwrap_or(yesterday);
        
        if last_date == today {
            // Already active today, no change
            return memory.learning_streak;
        } else if last_date == yesterday {
            // Active yesterday, increment streak
            memory.learning_streak += 1;
        } else if last_date < yesterday {
            // Missed a day, reset streak
            memory.learning_streak = 1;
        }
    } else {
        // First activity
        memory.learning_streak = 1;
    }
    
    memory.last_activity = Some(chrono::Utc::now().timestamp());
    memory.learning_streak
}

impl Memory {
    /// Populate the default achievement/decoration catalogs if missing
    /// (fresh profiles and old saves start with empty lists).
    pub fn ensure_defaults(&mut self) {
        if self.leveling.achievements.is_empty() {
            self.leveling.achievements = get_default_achievements();
        }
        if self.leveling.profile_decorations.is_empty() {
            self.leveling.profile_decorations = get_default_decorations();
        }
    }

    /// Roll 3 fresh daily quests when the calendar day changed.
    /// Returns true when a new set was rolled.
    pub fn ensure_daily_quests(&mut self) -> bool {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if self.quest_day.as_deref() == Some(today.as_str()) && !self.daily_quests.is_empty() {
            return false;
        }

        let mut pool: Vec<(String, String, u32, u32)> = QUEST_POOL
            .iter()
            .map(|(kind, title, target, xp)| (kind.to_string(), title.to_string(), *target, *xp))
            .collect();
        // Fisher–Yates shuffle, then take 3 distinct quests.
        for i in (1..pool.len()).rev() {
            let j = rand::random::<usize>() % (i + 1);
            pool.swap(i, j);
        }
        self.daily_quests = pool
            .into_iter()
            .take(3)
            .map(|(kind, title, target, xp_reward)| DailyQuest {
                kind,
                title,
                target,
                xp_reward,
                progress: 0,
                done: false,
            })
            .collect();
        self.quest_day = Some(today);
        self.games_today.clear();
        true
    }

    /// Advance every active quest of `kind` by `amount`. Newly completed
    /// quests pay out their XP immediately; returns one line per completion
    /// for a toast.
    pub fn quest_progress(&mut self, kind: &str, amount: u32) -> Vec<String> {
        let mut completed = Vec::new();
        for quest in &mut self.daily_quests {
            if quest.kind == kind && !quest.done {
                quest.progress = (quest.progress + amount).min(quest.target);
                if quest.progress >= quest.target {
                    quest.done = true;
                    let _ = self.leveling.add_experience(quest.xp_reward);
                    completed.push(format!(
                        "✅ Quest complete: {} (+{} XP)",
                        quest.title, quest.xp_reward
                    ));
                }
            }
        }
        completed
    }

    /// True when every daily quest is done (for the profile header).
    pub fn all_quests_done(&self) -> bool {
        !self.daily_quests.is_empty() && self.daily_quests.iter().all(|q| q.done)
    }

    /// Award every badge / achievement / decoration the user now qualifies
    /// for. Returns one human-readable line per newly unlocked reward, so
    /// the UI can flash a toast for each.
    pub fn check_progress_rewards(&mut self) -> Vec<String> {
        let mut unlocked: Vec<String> = Vec::new();
        let level = self.leveling.current_level;
        let questions = self.questions_answered;
        let streak = self.learning_streak;

        // --- badges ---
        if questions >= 1 && self.leveling.award_badge(Badge::FirstConversation) {
            unlocked.push("🏅 Badge unlocked: 💬 First Chat".to_string());
        }
        if questions >= 10 && self.leveling.award_badge(Badge::QuickLearner) {
            unlocked.push("🏅 Badge unlocked: ⚡ Quick Learner".to_string());
        }
        if questions >= 50 && self.leveling.award_badge(Badge::ConversationMaster(questions)) {
            unlocked.push("🏅 Badge unlocked: 🗣️ Chatterbox".to_string());
        }
        if streak >= 3 && self.leveling.award_badge(Badge::StreakWarrior(streak)) {
            unlocked.push(format!("🏅 Badge unlocked: 🔥 {}-day streak", streak));
        }
        if level >= 10 && self.leveling.award_badge(Badge::RocketFanatic) {
            unlocked.push("🏅 Badge unlocked: 🚀 ROCKET Fanatic".to_string());
        }
        if self.mastered_topics.len() >= 5 && self.leveling.award_badge(Badge::SkillExplorer) {
            unlocked.push("🏅 Badge unlocked: 🧭 Skill Explorer".to_string());
        }

        // --- achievements (from the default catalog) ---
        let catalog = get_default_achievements();
        let mut try_award = |slf: &mut Self, id: &str, unlocked: &mut Vec<String>| {
            if let Some(a) = catalog.iter().find(|c| c.id == id) {
                if slf.leveling.award_achievement(a.clone()) {
                    unlocked.push(format!("🏆 Achievement unlocked: {} {}", a.icon, a.title));
                }
            }
        };
        if questions >= 1 {
            try_award(self, "first_conversation", &mut unlocked);
        }
        if questions >= 10 {
            try_award(self, "quick_learner", &mut unlocked);
        }
        if questions >= 50 {
            try_award(self, "conversation_master", &mut unlocked);
        }
        if streak >= 7 {
            try_award(self, "streak_warrior", &mut unlocked);
        }
        if self.mastered_topics.len() >= 5 {
            try_award(self, "knowledge_seeker", &mut unlocked);
        }
        if level >= 10 {
            try_award(self, "rocket_fanatic", &mut unlocked);
        }

        // --- level-based decoration unlocks ---
        let decorations = get_default_decorations();
        let unlock_at: &[(&str, u32)] = &[
            ("blue_frame", 5),
            ("green_background", 5),
            ("star_icon", 8),
            ("gold_frame", 12),
            ("galaxy_background", 12),
            ("rocket_icon", 15),
            ("rainbow_frame", 20),
            ("neon_background", 25),
            ("legendary_frame", 30),
            ("cosmic_background", 40),
        ];
        for (id, at) in unlock_at {
            if level >= *at {
                if let Some(d) = decorations.iter().find(|c| c.id == *id) {
                    if self.leveling.unlock_decoration(d.clone()) {
                        unlocked.push(format!("🎨 Decoration unlocked: {} {}", d.icon, d.name));
                    }
                }
            }
        }

        // --- title unlocks (also backfills new catalog entries) ---
        unlocked.extend(self.sync_titles());

        unlocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_name_and_greets() {
        let mut mem = Memory::default();
        let reply = mem.capture("my name is wyatt").unwrap();
        assert!(reply.contains("Wyatt"), "{}", reply);
        assert_eq!(mem.user_name.as_deref(), Some("Wyatt"));
        assert!(mem.recall_context().contains("Wyatt"));
    }

    #[test]
    fn captures_favorite_game_and_main() {
        let mut mem = Memory::default();
        mem.capture("my favorite game is elden ring");
        mem.capture("my main is jett");
        assert_eq!(mem.favorite_game.as_deref(), Some("Elden Ring"));
        assert_eq!(mem.main_character.as_deref(), Some("Jett"));
    }

    #[test]
    fn captures_skill_level() {
        let mut mem = Memory::default();
        mem.capture("i'm a beginner");
        assert_eq!(mem.skill_level, SkillLevel::Beginner);

        mem.capture("i'm intermediate");
        assert_eq!(mem.skill_level, SkillLevel::Intermediate);

        mem.capture("i'm advanced");
        assert_eq!(mem.skill_level, SkillLevel::Advanced);
    }

    #[test]
    fn captures_freeform_facts_without_duplicates() {
        let mut mem = Memory::default();
        mem.capture("remember that i play on controller");
        let dup = mem.capture("remember that i play on controller");
        assert!(dup.unwrap().contains("already knew"));
        assert_eq!(mem.facts.len(), 1);
    }

    #[test]
    fn forget_everything_clears_and_confirms() {
        let mut mem = Memory::default();
        mem.capture("my name is wyatt");
        let reply = mem.capture("forget everything").unwrap();
        assert!(reply.contains("forgot"), "{}", reply);
        assert!(mem.is_empty());
    }

    #[test]
    fn plain_questions_are_not_captured() {
        let mut mem = Memory::default();
        assert!(mem.capture("how do i get netherite").is_none());
        assert!(mem.capture("best jett loadout").is_none());
        assert!(mem.is_empty());
    }

    #[test]
    fn recall_context_empty_when_nothing_known() {
        let mem = Memory::default();
        assert_eq!(mem.recall_context(), "");
    }

    #[test]
    fn add_conversation_and_get_summary() {
        let mut mem = Memory::default();
        mem.add_conversation(
            "how do i get diamonds".to_string(),
            "Diamonds are at Y -58".to_string(),
            Some("Minecraft".to_string()),
        );
        mem.add_conversation(
            "how do i get netherite".to_string(),
            "Mine Ancient Debris at Y 15".to_string(),
            Some("Minecraft".to_string()),
        );

        let recent = mem.get_recent_context(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].question, "how do i get netherite");
        assert_eq!(recent[1].question, "how do i get diamonds");

        let summary = mem.get_conversation_summary();
        assert!(summary.contains("Recent conversations"));
        assert!(summary.contains("Minecraft"));
    }

    #[test]
    fn conversation_history_limit() {
        let mut mem = Memory::default();
        for i in 0..60 {
            mem.add_conversation(
                format!("Question {}", i),
                format!("Answer {}", i),
                Some("Test".to_string()),
            );
        }
        assert_eq!(mem.conversation_history.len(), 50);
    }

    #[test]
    fn progress_rewards_unlock_by_thresholds() {
        let mut mem = Memory::default();
        mem.ensure_defaults();
        mem.questions_answered = 1;
        let rewards = mem.check_progress_rewards();
        assert!(rewards.iter().any(|r| r.contains("First Chat")));
        assert!(mem.leveling.has_badge(&Badge::FirstConversation));
        assert!(mem.leveling.has_achievement("first_conversation"));

        // Already-earned rewards don't re-unlock.
        assert!(mem.check_progress_rewards().is_empty());

        // Level 5 unlocks the blue frame decoration.
        mem.leveling.current_level = 5;
        let rewards = mem.check_progress_rewards();
        assert!(rewards.iter().any(|r| r.contains("Blue Frame")));
        assert!(mem
            .leveling
            .profile_decorations
            .iter()
            .any(|d| d.id == "blue_frame" && d.unlocked));
    }

    #[test]
    fn equip_decoration_only_when_unlocked() {
        let mut mem = Memory::default();
        mem.ensure_defaults();
        assert!(!mem.leveling.equip_decoration("gold_frame"));
        mem.leveling.current_level = 12;
        mem.check_progress_rewards();
        assert!(mem.leveling.equip_decoration("gold_frame"));
        let equipped: Vec<_> = mem.leveling.equipped_decorations().iter().map(|d| d.id.as_str()).collect();
        assert!(equipped.contains(&"gold_frame"));
    }

    #[test]
    fn level_titles_progress() {
        assert_eq!(level_title(1), "🌱 Rookie");
        assert_eq!(level_title(10), "🚀 Pilot");
        assert_eq!(level_title(42), "🌟 Legend");
        assert_eq!(level_title(50), "💫 Mythic");
    }

    #[test]
    fn daily_quests_roll_once_per_day_and_complete() {
        let mut mem = Memory::default();
        assert!(mem.ensure_daily_quests(), "first roll should create quests");
        assert_eq!(mem.daily_quests.len(), 3);
        assert!(!mem.ensure_daily_quests(), "same day must not re-roll");

        // Advance the "ask" quest(s) to completion and collect the payout.
        // (The pool can roll BOTH ask quests in one day, so advance by the
        // largest target — that completes every ask quest in one call.)
        let targets: Vec<u32> = mem
            .daily_quests
            .iter()
            .filter(|q| q.kind == "ask")
            .map(|q| q.target)
            .collect();
        if let Some(max_target) = targets.iter().max() {
            let completed = mem.quest_progress("ask", *max_target);
            assert!(!completed.is_empty(), "at least one ask quest should complete");
            assert!(completed.iter().all(|line| line.contains("Quest complete")));
            assert!(mem.daily_quests.iter().filter(|q| q.kind == "ask").all(|q| q.done));
            // XP was paid out.
            assert!(mem.leveling.total_xp_earned >= 25);
        }

        // Coins mirror all XP earned.
        assert_eq!(mem.leveling.coins, mem.leveling.total_xp_earned);
    }

    #[test]
    fn shop_unlocks_decorations_with_coins() {
        let mut mem = Memory::default();
        mem.ensure_defaults();
        mem.leveling.coins = 10_000;

        let before = mem.leveling.coins;
        let result = mem.leveling.buy_decoration("legendary_frame");
        assert!(result.is_ok(), "{}", result.unwrap_err());
        assert!(mem
            .leveling
            .profile_decorations
            .iter()
            .any(|d| d.id == "legendary_frame" && d.unlocked));
        assert_eq!(mem.leveling.coins, before - 3000);

        // Already-unlocked decorations can't be bought again.
        assert!(mem.leveling.buy_decoration("legendary_frame").is_err());

        // Mystery box consumes coins and unlocks something.
        let before = mem.leveling.coins;
        let result = mem.leveling.buy_mystery_box();
        assert!(result.is_ok(), "{}", result.unwrap_err());
        assert_eq!(mem.leveling.coins, before - MYSTERY_BOX_PRICE);

        // Broke shoppers are rejected.
        mem.leveling.coins = 5;
        assert!(mem.leveling.buy_decoration("rainbow_frame").is_err());
        assert!(mem.leveling.buy_mystery_box().is_err());
    }

    #[test]
    fn title_catalog_has_100_plus_unique_titles() {
        assert!(
            PROFILE_TITLES.len() >= 104,
            "expected 104+ titles, found {}",
            PROFILE_TITLES.len()
        );
        let mut ids: Vec<_> = PROFILE_TITLES.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PROFILE_TITLES.len(), "title ids must be unique");
        // Every title must state its requirement in words.
        for t in PROFILE_TITLES {
            assert!(!t.requirement().is_empty(), "{} has no requirement text", t.id);
        }
    }

    #[test]
    fn titles_unlock_by_progress_and_equip() {
        let mut mem = Memory::default();
        // Sync backfills the catalog and unlocks the level-1 title.
        let lines = mem.sync_titles();
        assert!(lines.iter().any(|l| l.contains("Fresh Launch")));
        let (unlocked, total) = mem.title_progress();
        assert_eq!(unlocked, 1);
        assert!(total >= 104);

        // Reach level 5 → the level-5 title unlocks with a toast line.
        // (Levels cost 100 + 250 + 500 + 750 = 1600 XP from level 1.)
        mem.leveling.add_experience(1600);
        let lines = mem.check_progress_rewards();
        assert!(
            lines.iter().any(|l| l.contains("Title unlocked") && l.contains("Cadet Grad")),
            "expected the level-5 title unlock toast, got {:?}",
            lines
        );
        // 5 level titles (1-5) + XP titles at 250 and 1000 = 7 unlocked.
        assert_eq!(mem.title_progress().0, 7);

        // Locked titles can't be equipped; unlocked ones toggle.
        mem.equip_title("lvl_50");
        assert!(mem.equipped_title_display().is_none(), "must not equip locked titles");
        mem.equip_title("lvl_05");
        assert_eq!(mem.equipped_title_display().unwrap().1, "Cadet Grad");
        mem.equip_title("lvl_05"); // equipping again clears it
        assert!(mem.equipped_title_display().is_none());
        mem.equip_title("lvl_01");
        assert!(mem.equipped_title_display().is_some());
    }

    #[test]
    fn nickname_and_bio_roundtrip() {
        let mut mem = Memory::default();
        mem.set_nickname("  Wyatt  ");
        assert_eq!(mem.nickname.as_deref(), Some("Wyatt"));
        mem.set_bio("Rocket main since day one.");
        assert_eq!(mem.bio.as_deref(), Some("Rocket main since day one."));

        // Length caps protect the profile layout.
        let long: String = "x".repeat(200);
        mem.set_nickname(&long);
        assert_eq!(mem.nickname.as_ref().map(|s| s.chars().count()), Some(24));
        mem.set_bio(&long);
        assert_eq!(mem.bio.as_ref().map(|s| s.chars().count()), Some(160));

        // Empty strings clear the fields.
        mem.set_nickname("   ");
        mem.set_bio("");
        assert!(mem.nickname.is_none());
        assert!(mem.bio.is_none());
    }

    #[test]
    fn profile_picture_copy_and_clear() {
        let mut mem = Memory::default();
        let src = std::env::temp_dir().join("rocket-profile-src-test.png");
        std::fs::write(&src, b"fakepng").unwrap();

        mem.set_profile_picture_from_path(&src).unwrap();
        let stored = mem.profile_picture.clone().unwrap();
        assert!(stored.starts_with("rocket-profile-picture."));
        assert!(std::path::Path::new(&stored).exists());

        // Clearing removes the copied file and the setting.
        mem.clear_profile_picture();
        assert!(mem.profile_picture.is_none());
        assert!(!std::path::Path::new(&stored).exists());
        let _ = std::fs::remove_file(&src);
    }

    #[test]
    fn build_talk_is_captured_and_acknowledged() {
        let mut mem = Memory::default();
        let ack = mem.capture_build_talk("im going for a mage build").unwrap();
        assert!(
            ack.to_lowercase().contains("mage"),
            "ack should reflect the context: {}",
            ack
        );
        assert_eq!(mem.context_hint(), Some("mage build"));

        // Dedup + reorder: re-stating moves it to the end, no duplicates.
        mem.capture_build_talk("im using a bow");
        mem.capture_build_talk("im going for a mage build");
        assert_eq!(mem.loadout_context.len(), 2);
        assert_eq!(mem.context_hint(), Some("mage build"));

        // Non build-talk returns None and stores nothing.
        assert!(mem.capture_build_talk("how do i find diamonds").is_none());
        assert_eq!(mem.loadout_context.len(), 2);
    }

    #[test]
    fn questions_and_statements_are_told_apart() {
        assert!(message_is_question("Im going for a mage build, and Im using clairvoince, whats it for?"));
        assert!(message_is_question("im using clairvoince whats it for"));
        assert!(message_is_question("what is clairvoyance for"));
        assert!(message_is_question("how does this work"));
        // Self-statements are NOT questions — Rocket should chat back.
        assert!(!message_is_question("im going for a mage build"));
        assert!(!message_is_question("im using a bow"));
        assert!(!message_is_question("im playing roblox today"));
    }

    #[test]
    fn missions_are_created_tracked_and_completed() {
        assert_eq!(
            parse_mission_creation("let's beat the ender dragon").as_deref(),
            Some("ender dragon")
        );
        assert_eq!(
            parse_mission_creation("help me beat Malenia").as_deref(),
            Some("malenia")
        );
        // Advice requests are not missions.
        assert_eq!(parse_mission_creation("how do i beat the ender dragon"), None);

        let mut mem = Memory::default();
        assert!(mem.create_mission("ender dragon", "Minecraft"));
        assert!(!mem.create_mission("ender dragon", "Minecraft"), "dup open mission rejected");
        assert_eq!(mem.active_missions().len(), 1);

        // A win report completes the matching mission.
        assert!(parse_mission_completion("i finally beat the ender dragon!").is_some());
        let mission = mem.complete_mission_by_text("i finally beat the ender dragon!").unwrap();
        assert_eq!(mission.title, "ender dragon");
        assert!(mission.done);
        assert_eq!(mem.active_missions().len(), 0);

        // Requests and questions are never treated as wins.
        assert_eq!(parse_mission_completion("let's beat the ender dragon"), None);
        assert_eq!(parse_mission_completion("how do i beat the ender dragon"), None);
    }

    #[test]
    fn mission_board_caps_at_five_open() {
        let mut mem = Memory::default();
        for i in 1..=5 {
            assert!(mem.create_mission(&format!("mission {}", i), "Test"));
        }
        assert!(!mem.create_mission("mission 6", "Test"), "board full at 5");
        // Completing one frees a slot.
        mem.complete_mission_by_text("mission 3 done");
        assert!(mem.create_mission("mission 6", "Test"));
    }

    #[test]
    fn vibe_switches_and_persists_in_memory() {
        let mut mem = Memory::default();
        assert_eq!(mem.current_vibe().id, "buddy", "default vibe is buddy");
        let intro = mem.set_vibe("funny");
        assert_eq!(mem.current_vibe().id, "funny");
        assert!(!intro.is_empty());
        mem.set_vibe("chill");
        assert_eq!(mem.current_vibe().id, "chill");
    }
}

impl LevelingData {
    /// Add experience points and check for level up.
    /// Every XP earned also mints a spendable coin.
    pub fn add_experience(&mut self, xp: u32) -> (bool, u32) {
        self.total_xp_earned += xp;
        self.experience_points += xp;
        self.coins += xp;
        
        let mut levels_gained = 0;
        let mut current_xp = self.experience_points;
        let mut current_level = self.current_level;
        
        while current_level < LEVEL_XP_REQUIREMENTS.len() as u32 
            && current_xp >= LEVEL_XP_REQUIREMENTS[current_level as usize] {
            current_xp -= LEVEL_XP_REQUIREMENTS[current_level as usize];
            current_level += 1;
            levels_gained += 1;
            
            if current_level >= LEVEL_XP_REQUIREMENTS.len() as u32 {
                break;
            }
        }
        
        if levels_gained > 0 {
            self.current_level = current_level;
            self.experience_points = current_xp;
        }
        
        (levels_gained > 0, levels_gained)
    }
    
    /// Award experience points for conversations
    pub fn award_conversation_xp(&mut self) -> (bool, u32) {
        let base_xp = 10;
        let mut xp = base_xp;
        
        // Bonus XP based on current level
        if self.current_level >= 10 {
            xp += 5;
        }
        if self.current_level >= 25 {
            xp += 10;
        }
        if self.current_level >= 40 {
            xp += 15;
        }
        
        self.add_experience(xp)
    }
    
    /// Award experience points for teaching ROCKET
    pub fn award_teaching_xp(&mut self) -> (bool, u32) {
        let base_xp = 25;
        let mut xp = base_xp;
        
        // Bonus XP for teaching advanced topics
        if self.current_level >= 20 {
            xp += 15;
        }
        if self.current_level >= 40 {
            xp += 25;
        }
        
        self.add_experience(xp)
    }
    
    /// Award experience points for corrections
    pub fn award_correction_xp(&mut self) -> (bool, u32) {
        let base_xp = 15;
        let mut xp = base_xp;
        
        // Bonus XP for high-level corrections
        if self.current_level >= 30 {
            xp += 10;
        }
        
        self.add_experience(xp)
    }
    
    /// Award experience points for following up on topics
    pub fn award_followup_xp(&mut self) -> (bool, u32) {
        let base_xp = 15;
        let mut xp = base_xp;
        
        // Bonus XP for high-level follow-ups
        if self.current_level >= 20 {
            xp += 10;
        }
        
        self.add_experience(xp)
    }
    
    /// Award experience points for learning streaks (wrapper for Memory method)
    pub fn award_streak_xp(&mut self, streak_days: u32) -> (bool, u32) {
        // This is a simple wrapper - the actual streak logic is in Memory
        let xp_amount = match streak_days {
            1..=3 => 5,
            4..=7 => 15,
            8..=14 => 30,
            15..=30 => 50,
            _ => 75,
        };
        
        self.add_experience(xp_amount)
    }
    
    /// Award a badge
    pub fn award_badge(&mut self, badge: Badge) -> bool {
        if !self.badges.contains(&badge) {
            self.badges.push(badge);
            return true;
        }
        false
    }
    
    /// Award an achievement
    pub fn award_achievement(&mut self, achievement: Achievement) -> bool {
        if let Some(existing) = self.achievements.iter_mut().find(|a| a.id == achievement.id) {
            if !existing.unlocked {
                existing.unlocked = true;
                existing.unlocked_date = Some(chrono::Utc::now().timestamp());
                return true;
            }
        } else {
            let mut new_achievement = achievement;
            new_achievement.unlocked = true;
            new_achievement.unlocked_date = Some(chrono::Utc::now().timestamp());
            self.achievements.push(new_achievement);
            return true;
        }
        false
    }
    
    /// Unlock a decoration
    pub fn unlock_decoration(&mut self, decoration: Decoration) -> bool {
        if let Some(existing) = self.profile_decorations.iter_mut().find(|d| d.id == decoration.id) {
            if !existing.unlocked {
                existing.unlocked = true;
                return true;
            }
        } else {
            let mut new_decoration = decoration;
            new_decoration.unlocked = true;
            self.profile_decorations.push(new_decoration);
            return true;
        }
        false
    }
    
    /// Equip a decoration
    pub fn equip_decoration(&mut self, decoration_id: &str) -> bool {
// Find the decoration to equip and its category
        let mut found_decoration = None;
        let category_to_equip = 'outer: loop {
            for (i, dec) in self.profile_decorations.iter_mut().enumerate() {
                if dec.id == decoration_id && dec.unlocked {
                    found_decoration = Some(i);
                    break 'outer dec.category.clone();
                }
            }
            return false;
        };
        
        // Unequip all decorations in the same category
        for dec in self.profile_decorations.iter_mut() {
            if dec.category == category_to_equip {
                dec.equipped = false;
            }
        }
        
        // Equip the selected decoration
        if let Some(index) = found_decoration {
            self.profile_decorations[index].equipped = true;
            return true;
        }
        false
    }
    
    /// Get XP required for next level
    pub fn xp_for_next_level(&self) -> Option<u32> {
        if self.current_level < LEVEL_XP_REQUIREMENTS.len() as u32 {
            Some(LEVEL_XP_REQUIREMENTS[self.current_level as usize] - self.experience_points)
        } else {
            None
        }
    }
    
    /// Get progress percentage to next level
    pub fn progress_to_next_level(&self) -> f32 {
        if self.current_level >= LEVEL_XP_REQUIREMENTS.len() as u32 {
            return 100.0;
        }
        
        let current_req = LEVEL_XP_REQUIREMENTS[self.current_level as usize];
        let prev_req = if self.current_level == 0 { 0 } else { LEVEL_XP_REQUIREMENTS[(self.current_level - 1) as usize] };
        
        let range = current_req - prev_req;
        let progress = self.experience_points - prev_req;
        
        (progress as f32 / range as f32) * 100.0
    }
    
    /// Get available decorations by category
    pub fn decorations_by_category(&self, category: DecorationCategory) -> Vec<&Decoration> {
        self.profile_decorations
            .iter()
            .filter(|d| d.category == category && d.unlocked)
            .collect()
    }
    
    /// Get equipped decorations
    pub fn equipped_decorations(&self) -> Vec<&Decoration> {
        self.profile_decorations
            .iter()
            .filter(|d| d.equipped)
            .collect()
    }
    
    /// Check if user has a specific badge
    pub fn has_badge(&self, badge: &Badge) -> bool {
        self.badges.contains(badge)
    }
    
    /// Check if user has unlocked an achievement
    pub fn has_achievement(&self, achievement_id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == achievement_id && a.unlocked)
    }
    
    /// Get user's total score (level + XP + achievements)
    pub fn total_score(&self) -> u64 {        let level_score = self.current_level as u64 * 1000;
        let xp_score = self.total_xp_earned as u64;
        let badge_score = self.badges.len() as u64 * 100;
        let achievement_score = self.achievements.iter().filter(|a| a.unlocked).count() as u64 * 50;
        let decoration_score = self.profile_decorations.iter().filter(|d| d.unlocked).count() as u64 * 25;
        
        level_score + xp_score + badge_score + achievement_score + decoration_score
    }

    /// Spend coins to unlock a locked decoration early.
    /// Returns a success line or an explanation of what went wrong.
    pub fn buy_decoration(&mut self, id: &str) -> Result<String, String> {
        let deco = self
            .profile_decorations
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| "Unknown decoration.".to_string())?;
        if deco.unlocked {
            return Err("Already unlocked — equip it instead!".to_string());
        }
        let price = decoration_price(&deco.rarity);
        if self.coins < price {
            return Err(format!(
                "Not enough coins — {} costs {}🪙 and you have {}🪙.",
                deco.name, price, self.coins
            ));
        }
        self.coins -= price;
        let deco = self
            .profile_decorations
            .iter_mut()
            .find(|d| d.id == id)
            .unwrap();
        deco.unlocked = true;
        Ok(format!(
            "🛒 Unlocked {} {} ({} rarity) for {}🪙 — equip it in your profile!",
            deco.icon, deco.name, deco.rarity.label(), price
        ))
    }

    /// Spend coins on a mystery box: unlocks one random locked decoration.
    pub fn buy_mystery_box(&mut self) -> Result<String, String> {
        let locked: Vec<usize> = self
            .profile_decorations
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.unlocked)
            .map(|(i, _)| i)
            .collect();
        if locked.is_empty() {
            return Err("You already own every decoration — nothing left in the boxes!".to_string());
        }
        if self.coins < MYSTERY_BOX_PRICE {
            return Err(format!(
                "Not enough coins — a box costs {}🪙 and you have {}🪙.",
                MYSTERY_BOX_PRICE, self.coins
            ));
        }
        self.coins -= MYSTERY_BOX_PRICE;
        let pick = locked[rand::random::<usize>() % locked.len()];
        let deco = &mut self.profile_decorations[pick];
        deco.unlocked = true;
        Ok(format!(
            "🎁 Mystery box opened — you got {} {} ({} rarity)! Equip it in your profile.",
            deco.icon,
            deco.name,
            deco.rarity.label()
        ))
    }
}

/// Predefined achievements
pub fn get_default_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: "first_conversation".to_string(),
            title: "First Conversation".to_string(),
            description: "Have your first conversation with ROCKET".to_string(),
            icon: "💬".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Common,
        },
        Achievement {
            id: "quick_learner".to_string(),
            title: "Quick Learner".to_string(),
            description: "Answer 10 questions correctly".to_string(),
            icon: "🚀".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Common,
        },
        Achievement {
            id: "conversation_master".to_string(),
            title: "Conversation Master".to_string(),
            description: "Have 50 conversations with ROCKET".to_string(),
            icon: "🗣️".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Rare,
        },
        Achievement {
            id: "streak_warrior".to_string(),
            title: "Streak Warrior".to_string(),
            description: "Maintain a 7-day learning streak".to_string(),
            icon: "🔥".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Rare,
        },
        Achievement {
            id: "knowledge_seeker".to_string(),
            title: "Knowledge Seeker".to_string(),
            description: "Ask about 5 different topics".to_string(),
            icon: "🔍".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Common,
        },
        Achievement {
            id: "rocket_fanatic".to_string(),
            title: "ROCKET Fanatic".to_string(),
            description: "Reach level 10".to_string(),
            icon: "🚀".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Epic,
        },
        Achievement {
            id: "early_adopter".to_string(),
            title: "Early Adopter".to_string(),
            description: "Use ROCKET within the first week".to_string(),
            icon: "⭐".to_string(),
            unlocked: false,
            unlocked_date: None,
            rarity: AchievementRarity::Legendary,
        },
    ]
}

/// Predefined decorations
pub fn get_default_decorations() -> Vec<Decoration> {
    vec![
        // Basic decorations (always available)
        Decoration {
            id: "basic_frame".to_string(),
            name: "Basic Frame".to_string(),
            description: "A simple profile frame".to_string(),
            icon: "⬜".to_string(),
            category: DecorationCategory::Frame,
            unlocked: true,
            equipped: true,
            rarity: DecorationRarity::Basic,
        },
        Decoration {
            id: "default_background".to_string(),
            name: "Default Background".to_string(),
            description: "The standard profile background".to_string(),
            icon: "🔵".to_string(),
            category: DecorationCategory::Background,
            unlocked: true,
            equipped: true,
            rarity: DecorationRarity::Basic,
        },
        
        // Common decorations
        Decoration {
            id: "blue_frame".to_string(),
            name: "Blue Frame".to_string(),
            description: "A stylish blue profile frame".to_string(),
            icon: "🔷".to_string(),
            category: DecorationCategory::Frame,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Common,
        },
        Decoration {
            id: "green_background".to_string(),
            name: "Green Background".to_string(),
            description: "A nature-inspired green background".to_string(),
            icon: "🟢".to_string(),
            category: DecorationCategory::Background,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Common,
        },
        Decoration {
            id: "star_icon".to_string(),
            name: "Star Icon".to_string(),
            description: "A shining star profile icon".to_string(),
            icon: "⭐".to_string(),
            category: DecorationCategory::Icon,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Common,
        },
        
        // Rare decorations
        Decoration {
            id: "gold_frame".to_string(),
            name: "Gold Frame".to_string(),
            description: "A luxurious gold profile frame".to_string(),
            icon: "🟨".to_string(),
            category: DecorationCategory::Frame,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Rare,
        },
        Decoration {
            id: "galaxy_background".to_string(),
            name: "Galaxy Background".to_string(),
            description: "A beautiful space galaxy background".to_string(),
            icon: "🌌".to_string(),
            category: DecorationCategory::Background,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Rare,
        },
        Decoration {
            id: "rocket_icon".to_string(),
            name: "Rocket Icon".to_string(),
            description: "A ROCKET-themed profile icon".to_string(),
            icon: "🚀".to_string(),
            category: DecorationCategory::Icon,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Rare,
        },
        
        // Epic decorations
        Decoration {
            id: "rainbow_frame".to_string(),
            name: "Rainbow Frame".to_string(),
            description: "A colorful rainbow profile frame".to_string(),
            icon: "🌈".to_string(),
            category: DecorationCategory::Frame,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Epic,
        },
        Decoration {
            id: "neon_background".to_string(),
            name: "Neon Background".to_string(),
            description: "A futuristic neon background".to_string(),
            icon: "💫".to_string(),
            category: DecorationCategory::Background,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Epic,
        },
        
        // Legendary decorations
        Decoration {
            id: "legendary_frame".to_string(),
            name: "Legendary Frame".to_string(),
            description: "An epic legendary profile frame".to_string(),
            icon: "💎".to_string(),
            category: DecorationCategory::Frame,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Legendary,
        },
        Decoration {
            id: "cosmic_background".to_string(),
            name: "Cosmic Background".to_string(),
            description: "A mystical cosmic background".to_string(),
            icon: "🌠".to_string(),
            category: DecorationCategory::Background,
            unlocked: false,
            equipped: false,
            rarity: DecorationRarity::Legendary,
        },
    ]
}
