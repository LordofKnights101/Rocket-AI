//! ROCKET — a personal gaming AI companion.
//! Desktop application using Rust + Iced 0.14 GUI framework.

use iced::{
    widget::{
        column, text, button, container, row, text_input, scrollable, progress_bar, image, Space, canvas,
    },
    Element, Task, Theme, Alignment, Length, Color,
};

mod detect;
mod knowledge;
mod memory;
mod rocket;
mod ai;
mod codex;
mod codex_tree;
mod initiation;
mod tree_graph;
mod llm;
mod theme;
mod vibe;
#[cfg(test)]
mod game_qa;
#[cfg(test)]
mod docs_gen;
use rocket::Rocket;

// Simple Brain struct for system prompt generation
struct Brain {
    learned_facts: std::collections::HashMap<String, String>,
    user_name: Option<String>,
    user_favorite: Option<String>,
}

impl Brain {
    fn new() -> Self {
        Self {
            learned_facts: std::collections::HashMap::new(),
            user_name: None,
            user_favorite: None,
        }
    }

    fn learned_facts(&self, _limit: usize) -> Vec<(String, String)> {
        self.learned_facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }

    fn user_favorite(&self) -> Option<&str> {
        self.user_favorite.as_deref()
    }

    fn teach(&mut self, question: &str, answer: String) {
        self.learned_facts.insert(question.to_string(), answer);
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}
use std::time::Instant;
use std::fs;

#[derive(Debug, Clone)]
enum Message {
    Send,
    InputChanged(String),
    CopyMessage(String),
    TypingIndicator,
    ClearHistory,
    ChangeGame(GameType),
    ShowCheatSheet,
    ShowKnowledge,
    KbSearchChanged(String),
    NextTheme,
    SelectCategory(GameFilter),
    OpenSettings,
    SetProvider(llm::Provider),
    SetApiKey(String),
    SetModel(String),
    SetCustomUrl(String),
    ForgetMemory,
    DetectGameTick,
    DetectedGame(Option<GameType>),
    ToggleAutoDetect,
    ToggleConversationMode,
    ApiResponse(String),
    RocketTick,
    ToggleAiApi,
    ToggleSettings,
    UpdateLlmSettings(llm::Settings),
    ShowApiHelp,
    UpdateStats,
    ShowLevelUp,
    ShowAchievement,
    EquipDecoration(String),
    ShowDecorations,
    ToggleProfile,
    ToggleGamePicker,
    DismissXpGain,
    GameSearchChanged(String),
    SelectPlatform(PlatformFilter),
    SelectEra(EraFilter),
    ClearDetectionBanner,
    AttachPicture,
    PictureLoaded(String, Vec<u8>),
    ClearAttachment,
    ToggleCompact,
    BuyDecoration(String),
    BuyMysteryBox,
    Nav(Page),
    Suggest(String),
    ProfileNicknameChanged(String),
    SaveNickname,
    ProfileBioChanged(String),
    SaveBio,
    PickProfilePicture,
    ProfilePicturePicked(Option<std::path::PathBuf>),
    ClearProfilePicture,
    EquipTitle(String),
    SetVibe(String),
    TestAiApi,
    AiTestResult(Result<String, String>),
    // --- Order of the Eternal Codex: Rocket's desk ---
    DeskInputChanged(String),
    DeskSend,
    DeskGenerate,
    DeskToggleInterject,
    TreeSelect(String),
    TreeNodeClicked(String),
    TreeNodeMarkDone(String, String),
    TreeTabMove(String, i32),
    TreeTabClose(String),
    CreateTreeButton,
    ScribeSection(ScribeSection),
}

/// 📜 Sub-categories of the Scriptorium — one visible at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ScribeSection {
    /// 🌳 The quest-tree atelier.
    #[default]
    Atelier,
    /// 🕯️ Rocket's desk.
    Desk,
    /// 📋 The pinned quest board.
    Board,
}

impl ScribeSection {
    fn label(&self) -> &'static str {
        match self {
            ScribeSection::Atelier => "🌳 Atelier",
            ScribeSection::Desk => "🕯️ Desk",
            ScribeSection::Board => "📋 Quest Board",
        }
    }
}

/// Which page of the app is currently shown in the main area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Chat,
    Profile,
    Games,
    Shop,
    Knowledge,
    CheatSheet,
    Scriptorium,
    Info,
    Settings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: String,
}

/// A picture the user attached for ROCKET to analyze.
#[derive(Debug, Clone)]
struct AttachedImage {
    file_name: String,
    /// `data:<mime>;base64,…` ready for vision-capable chat APIs.
    data_url: String,
}

/// User statistics for tracking progress and activity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct UserStats {
    pub games_played: u32,
    pub questions_answered: u32,
    pub topics_mastered: u32,
    pub skill_level: memory::SkillLevel,
    pub learning_streak: u32,
    pub total_facts_learned: u32,
    pub custom_games_added: u32,
    pub last_activity: Option<i64>,
    pub favorite_topics: Vec<String>,
    pub current_level: u32,
    pub experience_points: u32,
    pub total_xp_earned: u32,
    pub badges_earned: usize,
    pub achievements_unlocked: usize,
}

impl UserStats {
    fn new() -> Self {
        Self {
            games_played: 0,
            questions_answered: 0,
            topics_mastered: 0,
            skill_level: memory::SkillLevel::Beginner,
            learning_streak: 0,
            total_facts_learned: 0,
            custom_games_added: 0,
            last_activity: None,
            favorite_topics: Vec::new(),
            current_level: 1,
            experience_points: 0,
            total_xp_earned: 0,
            badges_earned: 0,
            achievements_unlocked: 0,
        }
    }

    fn update_from_memory(&mut self, memory: &memory::Memory) {
        self.questions_answered = memory.questions_answered;
        self.learning_streak = memory.learning_streak;
        self.skill_level = memory.skill_level;
        self.last_activity = memory.last_activity;
        self.total_facts_learned = memory.facts.len() as u32 + memory.mastered_topics.len() as u32;
        
        // Update leveling information
        self.current_level = memory.leveling.current_level;
        self.experience_points = memory.leveling.experience_points;
        self.total_xp_earned = memory.leveling.total_xp_earned;
        self.badges_earned = memory.leveling.badges.len();
        self.achievements_unlocked = memory.leveling.achievements.iter().filter(|a| a.unlocked).count();
        
        // Update favorite topics based on conversation history
        let mut topic_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for turn in &memory.conversation_history {
            if let Some(topic) = &turn.topic {
                *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }
        
        // Get top 3 topics
        let mut sorted_topics: Vec<_> = topic_counts.into_iter().collect();
        sorted_topics.sort_by(|a, b| b.1.cmp(&a.1));
        self.favorite_topics = sorted_topics
            .into_iter()
            .take(3)
            .map(|(topic, _)| topic)
            .collect();
    }
}

/// Platform availability for a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Platform {
    Pc,
    PlayStation,
    Xbox,
    Nintendo,
    Mobile,
}

/// Device filter chips for the game picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFilter {
    Any,
    Pc,
    PlayStation,
    Xbox,
    Nintendo,
    Mobile,
}

impl PlatformFilter {
    fn label(&self) -> &'static str {
        match self {
            PlatformFilter::Any => "🌐 All",
            PlatformFilter::Pc => "🖥️ PC",
            PlatformFilter::PlayStation => "🎮 PS",
            PlatformFilter::Xbox => "🎯 Xbox",
            PlatformFilter::Nintendo => "🍄 Nintendo",
            PlatformFilter::Mobile => "📱 Mobile",
        }
    }

    fn platform(&self) -> Option<Platform> {
        match self {
            PlatformFilter::Any => None,
            PlatformFilter::Pc => Some(Platform::Pc),
            PlatformFilter::PlayStation => Some(Platform::PlayStation),
            PlatformFilter::Xbox => Some(Platform::Xbox),
            PlatformFilter::Nintendo => Some(Platform::Nintendo),
            PlatformFilter::Mobile => Some(Platform::Mobile),
        }
    }

    fn matches(&self, game: &GameType) -> bool {
        match self.platform() {
            None => true,
            Some(p) => game.platforms().contains(&p),
        }
    }
}

const ALL_PLATFORMS: [PlatformFilter; 6] = [
    PlatformFilter::Any,
    PlatformFilter::Pc,
    PlatformFilter::PlayStation,
    PlatformFilter::Xbox,
    PlatformFilter::Nintendo,
    PlatformFilter::Mobile,
];

/// Release-era buckets for the game picker filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraFilter {
    Any,
    /// 2009 and earlier.
    Retro,
    /// 2010–2019.
    Y2010s,
    /// 2020 and later.
    Y2020s,
}

impl EraFilter {
    fn label(&self) -> &'static str {
        match self {
            EraFilter::Any => "📅 Any year",
            EraFilter::Retro => "📼 Retro (≤2009)",
            EraFilter::Y2010s => "💿 2010s",
            EraFilter::Y2020s => "✨ 2020s",
        }
    }

    fn matches(&self, year: u16) -> bool {
        match self {
            EraFilter::Any => true,
            EraFilter::Retro => year <= 2009,
            EraFilter::Y2010s => (2010..=2019).contains(&year),
            EraFilter::Y2020s => year >= 2020,
        }
    }
}

const ALL_ERAS: [EraFilter; 4] = [
    EraFilter::Any,
    EraFilter::Retro,
    EraFilter::Y2010s,
    EraFilter::Y2020s,
];

/// One game in the catalog: display name, emoji, genre, platforms, year.
struct GameInfo {
    id: GameType,
    name: &'static str,
    emoji: &'static str,
    genre: GameFilter,
    platforms: &'static [Platform],
    year: u16,
}

/// Genre filter tabs for the game picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameFilter {
    All,
    Shooter,
    Moba,
    Sandbox,
    Rpg,
    OpenWorld,
    Sports,
    Racing,
    Simulation,
    Party,
    Fighting,
    Horror,
    Strategy,
    Platformer,
    Mmo,
    Other,
}

const ALL_FILTERS: [GameFilter; 16] = [
    GameFilter::All,
    GameFilter::Shooter,
    GameFilter::Moba,
    GameFilter::Sandbox,
    GameFilter::Rpg,
    GameFilter::OpenWorld,
    GameFilter::Sports,
    GameFilter::Racing,
    GameFilter::Simulation,
    GameFilter::Party,
    GameFilter::Fighting,
    GameFilter::Horror,
    GameFilter::Strategy,
    GameFilter::Platformer,
    GameFilter::Mmo,
    GameFilter::Other,
];

impl GameFilter {
    fn label(&self) -> &'static str {
        match self {
            GameFilter::All => "🎮 All",
            GameFilter::Shooter => "🔫",
            GameFilter::Moba => "⚔️",
            GameFilter::Sandbox => "🧱",
            GameFilter::Rpg => "🐉",
            GameFilter::OpenWorld => "🚗",
            GameFilter::Sports => "⚽",
            GameFilter::Racing => "🏁",
            GameFilter::Simulation => "🌾",
            GameFilter::Party => "🎉",
            GameFilter::Fighting => "🥊",
            GameFilter::Horror => "👻",
            GameFilter::Strategy => "♟️",
            GameFilter::Platformer => "🏃",
            GameFilter::Mmo => "🌐",
            GameFilter::Other => "🎲",
        }
    }

    fn matches(&self, game: &GameType) -> bool {
        *self == GameFilter::All || game.get_category() == *self
    }
}

/// Generates the `GameType` enum and the static `CATALOG` table from one
/// list, so the picker, filters and display names never drift apart.
macro_rules! games_table {
    ($( $variant:ident => ($name:literal, $emoji:literal, $genre:expr,
         [$($plat:expr),* $(,)?], $year:literal) ),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        enum GameType { $( $variant, )* Generic }

        /// Every supported game with its metadata — single source of truth.
        const CATALOG: &[GameInfo] = &[
            $( GameInfo {
                id: GameType::$variant,
                name: $name,
                emoji: $emoji,
                genre: $genre,
                platforms: &[$($plat),*],
                year: $year,
            }, )*
        ];
    };
}

games_table! {
    // ---- originals ----
    Minecraft => ("Minecraft", "⛏️", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2011),
    Roblox => ("Roblox", "🧱", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Mobile], 2006),
    Valorant => ("Valorant", "🎯", GameFilter::Shooter, [Platform::Pc], 2020),
    CS2 => ("Counter-Strike 2", "💣", GameFilter::Shooter, [Platform::Pc], 2023),
    Fortnite => ("Fortnite", "🪂", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2017),
    CallOfDuty => ("Call of Duty", "🎖️", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2003),
    ApexLegends => ("Apex Legends", "🏹", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    PUBG => ("PUBG", "🍗", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Mobile], 2017),
    LeagueOfLegends => ("League of Legends", "⚔️", GameFilter::Moba, [Platform::Pc], 2009),
    Dota2 => ("Dota 2", "🪓", GameFilter::Moba, [Platform::Pc], 2013),
    WorldOfWarcraft => ("World of Warcraft", "🧝", GameFilter::Mmo, [Platform::Pc], 2004),
    Skyrim => ("Skyrim", "🐉", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2011),
    Zelda => ("Zelda Series", "🛡️", GameFilter::Rpg, [Platform::Nintendo], 1986),
    GTA => ("GTA Series", "🚗", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 1997),
    RocketLeague => ("Rocket League", "⚽", GameFilter::Sports, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2015),
    StardewValley => ("Stardew Valley", "🌾", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2016),
    TheSims => ("The Sims", "💎", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2000),
    AmongUs => ("Among Us", "👽", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2018),
    FallGuys => ("Fall Guys", "🎉", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2020),
    // ---- shooters ----
    Overwatch => ("Overwatch 2", "🦸", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2022),
    RainbowSix => ("Rainbow Six Siege", "🚪", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2015),
    Destiny2 => ("Destiny 2", "🌌", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2017),
    Halo => ("Halo Infinite", "👽", GameFilter::Shooter, [Platform::Pc, Platform::Xbox], 2021),
    GearsOfWar => ("Gears of War", "⚙️", GameFilter::Shooter, [Platform::Pc, Platform::Xbox], 2006),
    Battlefield => ("Battlefield 2042", "💥", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    EscapeFromTarkov => ("Escape from Tarkov", "🎒", GameFilter::Shooter, [Platform::Pc], 2017),
    HuntShowdown => ("Hunt: Showdown", "🌕", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    Warframe => ("Warframe", "🤖", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2013),
    TeamFortress2 => ("Team Fortress 2", "🎩", GameFilter::Shooter, [Platform::Pc], 2007),
    MarvelRivals => ("Marvel Rivals", "🕷️", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2024),
    Helldivers2 => ("Helldivers 2", "🪖", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation], 2024),
    Doom => ("Doom Eternal", "😈", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2020),
    Borderlands => ("Borderlands 3", "🔫", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    FarCry => ("Far Cry 5", "🏞️", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    TheFinals => ("The Finals", "🏟️", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    Paladins => ("Paladins", "🔮", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2018),
    Splitgate => ("Splitgate", "🌀", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    Titanfall2 => ("Titanfall 2", "🦾", GameFilter::Shooter, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2016),
    Splatoon => ("Splatoon 3", "🦑", GameFilter::Shooter, [Platform::Nintendo], 2022),
    // ---- MOBA / strategy ----
    Smite => ("Smite", "⚡", GameFilter::Moba, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2014),
    HeroesOfTheStorm => ("Heroes of the Storm", "🌪️", GameFilter::Moba, [Platform::Pc], 2015),
    MobileLegends => ("Mobile Legends", "📱", GameFilter::Moba, [Platform::Mobile], 2016),
    BrawlStars => ("Brawl Stars", "⭐", GameFilter::Moba, [Platform::Mobile], 2018),
    StarCraft2 => ("StarCraft II", "🚀", GameFilter::Strategy, [Platform::Pc], 2010),
    AgeOfEmpires => ("Age of Empires IV", "🏰", GameFilter::Strategy, [Platform::Pc, Platform::Xbox], 2021),
    Civilization6 => ("Civilization VI", "🌍", GameFilter::Strategy, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2016),
    TotalWar => ("Total War: WARHAMMER III", "🔨", GameFilter::Strategy, [Platform::Pc], 2022),
    ClashOfClans => ("Clash of Clans", "🛡️", GameFilter::Strategy, [Platform::Mobile], 2012),
    ClashRoyale => ("Clash Royale", "👑", GameFilter::Strategy, [Platform::Mobile], 2016),
    Warcraft3 => ("Warcraft III", "🐲", GameFilter::Strategy, [Platform::Pc], 2002),
    // ---- sandbox / crafting / survival ----
    Terraria => ("Terraria", "🌳", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2011),
    Rust => ("Rust", "🪓", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    ARK => ("ARK: Survival Evolved", "🦖", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2017),
    Valheim => ("Valheim", "🛶", GameFilter::Sandbox, [Platform::Pc, Platform::Xbox], 2021),
    SevenDaysToDie => ("7 Days to Die", "🧟", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2013),
    Subnautica => ("Subnautica", "🐠", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2018),
    NoMansSky => ("No Man's Sky", "🌠", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2016),
    DontStarve => ("Don't Starve", "🌑", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2013),
    Raft => ("Raft", "🛟", GameFilter::Sandbox, [Platform::Pc], 2022),
    TheForest => ("The Forest", "🌲", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation], 2018),
    SonsOfTheForest => ("Sons of the Forest", "🪵", GameFilter::Sandbox, [Platform::Pc], 2024),
    Grounded => ("Grounded", "🐜", GameFilter::Sandbox, [Platform::Pc, Platform::Xbox, Platform::PlayStation], 2022),
    Astroneer => ("Astroneer", "🪐", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2019),
    Palworld => ("Palworld", "🐑", GameFilter::Sandbox, [Platform::Pc, Platform::Xbox, Platform::PlayStation], 2024),
    LegoFortnite => ("LEGO Fortnite", "🧱", GameFilter::Sandbox, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    // ---- RPG / action ----
    EldenRing => ("Elden Ring", "💍", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2022),
    DarkSouls3 => ("Dark Souls III", "🔥", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2016),
    BaldursGate3 => ("Baldur's Gate 3", "🐲", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    Cyberpunk => ("Cyberpunk 2077", "🌃", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2020),
    Witcher3 => ("The Witcher 3", "🐺", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2015),
    Diablo4 => ("Diablo IV", "😈", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    PathOfExile => ("Path of Exile", "💀", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2013),
    Persona5 => ("Persona 5 Royal", "🎭", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2019),
    MonsterHunter => ("Monster Hunter World", "🐗", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    MonsterHunterWilds => ("Monster Hunter Wilds", "🌩️", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2025),
    Genshin => ("Genshin Impact", "🌫️", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Mobile], 2020),
    HonkaiStarRail => ("Honkai: Star Rail", "🚄", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Mobile], 2023),
    Bloodborne => ("Bloodborne", "🌙", GameFilter::Rpg, [Platform::PlayStation], 2015),
    Sekiro => ("Sekiro", "🥷", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    Fallout4 => ("Fallout 4", "☢️", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2015),
    Fallout76 => ("Fallout 76", "🏚️", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    Oblivion => ("Oblivion", "🚪", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2006),
    KingdomCome => ("Kingdom Come: Deliverance", "⚔️", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    Divinity2 => ("Divinity: Original Sin 2", "✨", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2017),
    MassEffect => ("Mass Effect", "🚀", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2007),
    Pokemon => ("Pokémon Scarlet/Violet", "🔴", GameFilter::Rpg, [Platform::Nintendo], 2022),
    Hogwarts => ("Hogwarts Legacy", "🪄", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2023),
    Wukong => ("Black Myth: Wukong", "🐒", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2024),
    Starfield => ("Starfield", "🌟", GameFilter::Rpg, [Platform::Pc, Platform::Xbox], 2023),
    Hades => ("Hades", "🔥", GameFilter::Rpg, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2020),
    // ---- open world / action ----
    RDR2 => ("Red Dead Redemption 2", "🤠", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2018),
    AssassinsCreed => ("Assassin's Creed Valhalla", "🪓", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2020),
    GhostOfTsushima => ("Ghost of Tsushima", "⛩️", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation], 2020),
    HorizonZeroDawn => ("Horizon Zero Dawn", "🦖", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation], 2017),
    DeathStranding => ("Death Stranding", "🌧️", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    WatchDogs => ("Watch Dogs 2", "💻", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2016),
    SeaOfThieves => ("Sea of Thieves", "🏴", GameFilter::OpenWorld, [Platform::Pc, Platform::Xbox, Platform::PlayStation], 2018),
    DaysGone => ("Days Gone", "🏍️", GameFilter::OpenWorld, [Platform::Pc, Platform::PlayStation], 2019),
    // ---- sports / racing ----
    EAFC => ("EA Sports FC 24", "⚽", GameFilter::Sports, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2023),
    NBA2K => ("NBA 2K24", "🏀", GameFilter::Sports, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2023),
    Madden => ("Madden NFL 24", "🏈", GameFilter::Sports, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    MLBTheShow => ("MLB The Show", "⚾", GameFilter::Sports, [Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2021),
    UFC => ("UFC 5", "🥊", GameFilter::Sports, [Platform::PlayStation, Platform::Xbox], 2023),
    Forza => ("Forza Horizon 5", "🏎️", GameFilter::Racing, [Platform::Pc, Platform::Xbox], 2021),
    GranTurismo => ("Gran Turismo 7", "🏁", GameFilter::Racing, [Platform::PlayStation], 2022),
    MarioKart => ("Mario Kart 8 Deluxe", "🍄", GameFilter::Racing, [Platform::Nintendo], 2017),
    F1 => ("F1 23", "🏎", GameFilter::Racing, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    NeedForSpeed => ("Need for Speed Heat", "🚙", GameFilter::Racing, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2019),
    Trackmania => ("Trackmania", "🛣️", GameFilter::Racing, [Platform::Pc], 2020),
    // ---- simulation ----
    FlightSim => ("Microsoft Flight Simulator", "✈️", GameFilter::Simulation, [Platform::Pc, Platform::Xbox], 2020),
    EuroTruck => ("Euro Truck Simulator 2", "🚚", GameFilter::Simulation, [Platform::Pc], 2012),
    CitiesSkylines => ("Cities: Skylines", "🏙️", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2015),
    PlanetCoaster => ("Planet Coaster", "🎢", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2016),
    FootballManager => ("Football Manager", "📋", GameFilter::Simulation, [Platform::Pc, Platform::Mobile], 2024),
    FarmingSim => ("Farming Simulator 22", "🚜", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    PowerWash => ("PowerWash Simulator", "🚿", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2022),
    Kerbal => ("Kerbal Space Program", "🛰️", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2015),
    HouseFlipper => ("House Flipper", "🔨", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2018),
    Factorio => ("Factorio", "⚙️", GameFilter::Simulation, [Platform::Pc, Platform::Nintendo], 2020),
    Satisfactory => ("Satisfactory", "🏭", GameFilter::Simulation, [Platform::Pc], 2024),
    AnimalCrossing => ("Animal Crossing", "🍃", GameFilter::Simulation, [Platform::Nintendo], 2020),
    DaveTheDiver => ("Dave the Diver", "🤿", GameFilter::Simulation, [Platform::Pc, Platform::PlayStation, Platform::Nintendo], 2023),
    // ---- party / co-op ----
    Jackbox => ("Jackbox Party Pack", "🎉", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2014),
    GarticPhone => ("Gartic Phone", "📞", GameFilter::Party, [Platform::Pc], 2021),
    PummelParty => ("Pummel Party", "🎲", GameFilter::Party, [Platform::Pc], 2018),
    GolfWithFriends => ("Golf With Your Friends", "⛳", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2020),
    TabletopSim => ("Tabletop Simulator", "♠️", GameFilter::Party, [Platform::Pc], 2015),
    HumanFallFlat => ("Human: Fall Flat", "🧍", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2016),
    GangBeasts => ("Gang Beasts", "🟨", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2017),
    StumbleGuys => ("Stumble Guys", "🏃", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Mobile], 2021),
    MarioParty => ("Mario Party Superstars", "⭐", GameFilter::Party, [Platform::Nintendo], 2021),
    ItTakesTwo => ("It Takes Two", "👫", GameFilter::Party, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    PokemonGO => ("Pokémon GO", "📍", GameFilter::Party, [Platform::Mobile], 2016),
    // ---- fighting ----
    StreetFighter6 => ("Street Fighter 6", "👊", GameFilter::Fighting, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    MortalKombat => ("Mortal Kombat 1", "🥋", GameFilter::Fighting, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2023),
    Tekken => ("Tekken 8", "🥊", GameFilter::Fighting, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2024),
    SmashBros => ("Super Smash Bros. Ultimate", "💥", GameFilter::Fighting, [Platform::Nintendo], 2018),
    GuiltyGear => ("Guilty Gear Strive", "🎸", GameFilter::Fighting, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    DBFighterZ => ("Dragon Ball FighterZ", "🐉", GameFilter::Fighting, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2018),
    // ---- horror ----
    Phasmophobia => ("Phasmophobia", "👻", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2020),
    RE4 => ("Resident Evil 4", "🧟", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2023),
    Outlast => ("Outlast", "🔦", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2013),
    FNAF => ("Five Nights at Freddy's", "🐻", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2014),
    LethalCompany => ("Lethal Company", "🏢", GameFilter::Horror, [Platform::Pc], 2023),
    PoppyPlaytime => ("Poppy Playtime", "🧸", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Mobile], 2021),
    DeadSpace => ("Dead Space", "🩸", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2008),
    AlienIsolation => ("Alien: Isolation", "👾", GameFilter::Horror, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2014),
    // ---- platformer / indie ----
    HollowKnight => ("Hollow Knight", "🪲", GameFilter::Platformer, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2017),
    Celeste => ("Celeste", "⛰️", GameFilter::Platformer, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2018),
    Ori => ("Ori and the Will of the Wisps", "🦉", GameFilter::Platformer, [Platform::Pc, Platform::Xbox, Platform::Nintendo], 2020),
    Cuphead => ("Cuphead", "☕", GameFilter::Platformer, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2017),
    DeadCells => ("Dead Cells", "🗡️", GameFilter::Platformer, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2018),
    MinecraftDungeons => ("Minecraft Dungeons", "🗺️", GameFilter::Platformer, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo], 2020),
    Portal2 => ("Portal 2", "🌀", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2011),
    Undertale => ("Undertale", "💛", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Nintendo], 2015),
    SlayTheSpire => ("Slay the Spire", "🗼", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2019),
    VampireSurvivors => ("Vampire Survivors", "🦇", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2022),
    Balatro => ("Balatro", "🃏", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Xbox, Platform::Nintendo, Platform::Mobile], 2024),
    Journey => ("Journey", "🏜️", GameFilter::Other, [Platform::Pc, Platform::PlayStation], 2012),
    Naraka => ("Naraka: Bladepoint", "🌪️", GameFilter::Other, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2021),
    // ---- MMO ----
    FinalFantasy14 => ("Final Fantasy XIV", "🗡️", GameFilter::Mmo, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2013),
    ESO => ("Elder Scrolls Online", "📜", GameFilter::Mmo, [Platform::Pc, Platform::PlayStation, Platform::Xbox], 2014),
    NewWorld => ("New World", "🗺️", GameFilter::Mmo, [Platform::Pc], 2021),
    GuildWars2 => ("Guild Wars 2", "🦁", GameFilter::Mmo, [Platform::Pc], 2012),
    LostArk => ("Lost Ark", "⚓", GameFilter::Mmo, [Platform::Pc], 2022),
    RuneScape => ("Old School RuneScape", "🪙", GameFilter::Mmo, [Platform::Pc, Platform::Mobile], 2013),
}

impl GameType {
    fn get_name(&self) -> &'static str {
        CATALOG
            .iter()
            .find(|g| g.id == *self)
            .map(|g| g.name)
            .unwrap_or("General Chat")
    }

    fn get_emoji(&self) -> &'static str {
        CATALOG
            .iter()
            .find(|g| g.id == *self)
            .map(|g| g.emoji)
            .unwrap_or("🎮")
    }

    fn get_category(&self) -> GameFilter {
        CATALOG
            .iter()
            .find(|g| g.id == *self)
            .map(|g| g.genre)
            .unwrap_or(GameFilter::Other)
    }

    fn year(&self) -> u16 {
        CATALOG
            .iter()
            .find(|g| g.id == *self)
            .map(|g| g.year)
            .unwrap_or(2024)
    }

    fn platforms(&self) -> &'static [Platform] {
        CATALOG
            .iter()
            .find(|g| g.id == *self)
            .map(|g| g.platforms)
            .unwrap_or(&[])
    }
}

/// Every catalog game plus the general-chat pseudo-game, for the picker —
/// sorted alphabetically by display name.
fn all_games() -> Vec<GameType> {
    let mut games: Vec<GameType> = CATALOG.iter().map(|g| g.id.clone()).collect();
    games.push(GameType::Generic);
    games.sort_by_key(|g| g.get_name().to_lowercase());
    games
}

/// What the knowledge pipeline answered, and where the answer came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KnowledgeAnswer {
    /// Curated guide from the currently selected game's pack (plus its first tag).
    Game(String, &'static str),
    /// Curated guide from another game's pack — ONLY ever returned in
    /// General Chat mode, where no single game is selected.
    CrossGame(String),
    /// Universal "how to get X" answer for any item in any game.
    UniversalItem(String),
}

/// The shared question-answering pipeline used by the chat. A selected game
/// answers ONLY from its own curated pack (then the universal item brain) —
/// knowledge from other games is reserved for General Chat mode, where no
/// game is selected. Kept as a pure function so the QA suite tests the exact
/// chat path.
fn knowledge_answer(
    kb: &knowledge::KnowledgeBase,
    current_game: &GameType,
    user_message: &str,
) -> Option<KnowledgeAnswer> {
    let kb_hits = kb.search(current_game, user_message, 2);
    if !kb_hits.is_empty() {
        let first_tag = kb_hits
            .first()
            .and_then(|hit| hit.tags.first())
            .copied()
            .unwrap_or("");
        return Some(KnowledgeAnswer::Game(
            knowledge::format_chat_answer(&kb_hits),
            first_tag,
        ));
    }

    // A specific game is selected — its answers come ONLY from its own pack.
    // Never leak another game's knowledge into a selected game.
    if *current_game != GameType::Generic {
        return knowledge::universal_item_answer(current_game, user_message)
            .map(KnowledgeAnswer::UniversalItem);
    }

    // General Chat mode (no game selected): knowledge from any game is fair
    // game — the question itself names the game it's about.
    let cross_hits = kb.search_any(user_message, 2);
    if !cross_hits.is_empty() {
        return Some(KnowledgeAnswer::CrossGame(knowledge::format_cross_answer(
            &cross_hits,
        )));
    }

    // Universal item knowledge — any item, any game, nothing curated needed.
    knowledge::universal_item_answer(current_game, user_message)
        .map(KnowledgeAnswer::UniversalItem)
}
#[derive(Debug, Clone)]
struct App {
    input_text: String,
    messages: Vec<ChatMessage>,
    is_typing: bool,
    current_game: GameType,
    use_ai_api: bool,
    rocket: Rocket,
    llm_settings: llm::Settings,
    kb: knowledge::KnowledgeBase,
    kb_search: String,
    theme_idx: usize,
    game_filter: GameFilter,
    memory: memory::Memory,
    stats: UserStats,
    /// Which page is shown in the main area.
    page: Page,
    auto_detect: bool,
    detected: Option<GameType>,
    last_answered_topic: Option<String>,
    show_level_up: bool,
    level_up_details: String,
    show_xp_gain: bool,
    xp_gain_details: String,
    show_achievement: bool,
    achievement_details: String,
    /// User manually picked a game — auto-detect won't override it.
    manual_pick: bool,
    /// Free-text name search for the game picker.
    game_search: String,
    /// Device filter for the game picker.
    platform_filter: PlatformFilter,
    /// Release-era filter for the game picker.
    era_filter: EraFilter,
    /// Transient "game detected" banner (auto-vanishes after a few seconds).
    detection_banner: Option<String>,
    /// Picture attached for ROCKET to analyze (sent with the next message).
    attached_image: Option<AttachedImage>,
    /// Compact overlay mode — a small always-handy chat bar.
    compact: bool,
    /// Set by `get_ai_response` when the universal item answer fired.
    last_was_item_answer: bool,
    /// Draft text for the Profile page's nickname editor.
    profile_nickname_input: String,
    /// Draft text for the Profile page's bio editor.
    profile_bio_input: String,
    /// The Order of the Eternal Codex — ranks, lore realms, quest board,
    /// chronicle, audit log, and Rocket's desk history.
    codex: codex::Codex,
    /// Draft text for the Scriptorium desk input.
    desk_input: String,
    /// 🌳 CREATE TREE — which tree tab is shown in the atelier (None = focused).
    tree_view: Option<String>,
    /// 🌳 The node currently selected in the CREATE TREE graph.
    tree_selected_node: Option<String>,
    /// 📜 Which Scriptorium sub-category is showing.
    scribe_section: ScribeSection,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            input_text: String::new(),
            messages: Vec::new(),
            is_typing: false,
            current_game: GameType::Generic,
            use_ai_api: false,
            kb: knowledge::KnowledgeBase::new(),
            kb_search: String::new(),
            theme_idx: theme::load_index(),
            game_filter: GameFilter::All,
            memory: memory::Memory::load(),
            stats: UserStats::new(),
            page: Page::Chat,
            auto_detect: detect::load_enabled(),
            detected: None,
            rocket: Rocket {
                started: Instant::now(),
                tick: 0,
                warming: false,
                last_launch: None,
            },
            llm_settings: llm::Settings::load(),
            last_answered_topic: None,
            profile_nickname_input: String::new(),
            profile_bio_input: String::new(),
            codex: codex::Codex::load(),
            desk_input: String::new(),
            tree_view: None,
            tree_selected_node: None,
            scribe_section: ScribeSection::Atelier,
            show_level_up: false,
            level_up_details: String::new(),
            show_xp_gain: false,
            xp_gain_details: String::new(),
            show_achievement: false,
            achievement_details: String::new(),
            manual_pick: false,
            game_search: String::new(),
            platform_filter: PlatformFilter::Any,
            era_filter: EraFilter::Any,
            detection_banner: None,
            attached_image: None,
            compact: false,
            last_was_item_answer: false,
        };
        // Populate default achievements/decorations for fresh or legacy saves.
        app.memory.ensure_defaults();
        app.memory.ensure_daily_quests();
        app.memory.save();
        app.stats.update_from_memory(&app.memory);
        app.load_message_history();
        app
    }

    // Helper method to get brain data for system prompt
    fn get_brain(&mut self) -> Brain {
        let mut brain = Brain::new();
        
        // Add learned facts from conversation history
        for msg in self.messages.iter() {
            if msg.role == "user" && msg.content.starts_with("wrong:") {
                // Handle corrections
                let parts: Vec<&str> = msg.content.splitn(2, ":").collect();
                if parts.len() == 2 {
                    let question = parts[1].trim();
                    // Find the previous bot message to correct
                    if let Some(prev_msg) = self.messages.iter().rev().skip(1).find(|m| m.role == "bot") {
                        brain.teach(question, prev_msg.content.clone());
                        // Award correction XP
                        let (leveled_up, xp_earned) = self.memory.leveling.award_correction_xp();
                        if leveled_up {
                            self.show_level_up = true;
                            self.level_up_details = format!("🎉 LEVEL UP! You reached level {}!", self.memory.leveling.current_level);
                        }
                        if xp_earned > 0 {
                            self.show_xp_gain = true;
                            self.xp_gain_details = "✨ +15 XP for the correction!".to_string();
                        }
                    }
                }
            }
        }
        
        brain
    }
    fn add_message(&mut self, role: String, content: String) {
        let timestamp = self.get_timestamp();
        let role_clone = role.clone();
        self.messages.push(ChatMessage {
            role,
            content,
            timestamp,
        });
        
        // Award XP for user messages
        if role_clone == "user" {
            // Update learning streak and award streak XP
            let streak_days = memory::update_streak(&mut self.memory);
            if streak_days > 1 {
                let (streak_leveled_up, streak_xp) = self.memory.leveling.award_streak_xp(streak_days);
if streak_leveled_up {
                            self.show_level_up = true;
                            self.level_up_details = format!("🎉 LEVEL UP! You reached level {}!", self.memory.leveling.current_level);
                            self.show_achievement = true;
                            self.achievement_details = format!("🔥 {}-day learning streak achieved!", streak_days);
                        }
                if streak_xp > 0 {
                    self.show_xp_gain = true;
                    self.xp_gain_details = format!("🔥 Streak bonus — {} days in a row!", streak_days);
                }
            }
            
            // Award conversation XP
            let (leveled_up, xp_earned) = self.memory.leveling.award_conversation_xp();
            if leveled_up {
                self.show_level_up = true;
                self.level_up_details = format!("🎉 LEVEL UP! You reached level {}!", self.memory.leveling.current_level);
            }
            if xp_earned > 0 {
                self.show_xp_gain = true;
                self.xp_gain_details = "✨ +10 XP for chatting!".to_string();
            }
        }
        
        self.save_message_history();
    }
    
    /// Check for newly earned badges / achievements / decorations and flash
    /// a reward toast when something new unlocked.
    fn flash_progress_rewards(&mut self) {
        let rewards = self.memory.check_progress_rewards();
        if !rewards.is_empty() {
            self.show_achievement = true;
            self.achievement_details = rewards.join("\n");
        }
        self.stats.update_from_memory(&self.memory);
    }

    /// Flash completed daily quests in the reward toast.
    fn flash_quests(&mut self, completed: Vec<String>) {
        if completed.is_empty() {
            return;
        }
        self.show_achievement = true;
        if !self.achievement_details.is_empty() {
            self.achievement_details.push('\n');
        }
        self.achievement_details.push_str(&completed.join("\n"));
    }


    /// Codex scholarship hook — call for every answered question.
    ///
    /// Awards lore/novelty/depth bonuses on top of the standard economy,
    /// advances the Codex quest board, and returns Rocket's marginalia note
    /// to append to the answer (only when the user granted interjection).
    fn codex_answer_hook(&mut self, tag: Option<&str>, question: &str, kb_sourced: bool) -> String {
        let game = self.current_game.clone();
        let mut out = self.codex.record_realm_question(&game, tag, question, kb_sourced);
        // Keep the board alive — pin fresh quests drawn from the grimoires.
        let mut pinned = self.codex.ensure_board(&self.kb, self.memory.leveling.current_level);
        out.lines.append(&mut pinned);
        // 🌳 CREATE TREE — route the event through every living quest tree.
        let tree_out = self.codex.tree_answer_event(&game, tag, kb_sourced);
        out.xp_bonus += tree_out.xp;
        out.lines.extend(tree_out.lines);
        if out.xp_bonus + out.quest_xp > 0 {
            let (leveled, _) = self
                .memory
                .leveling
                .add_experience(out.xp_bonus + out.quest_xp);
            if leveled {
                self.show_level_up = true;
                self.level_up_details =
                    codex::proclaim_level_up(self.memory.leveling.current_level);
            }
            if out.xp_bonus > 0 {
                self.show_xp_gain = true;
                self.xp_gain_details = codex::narrate_xp(out.xp_bonus + out.quest_xp);
            }
        }
        let note = if self.codex.desk_interjections && !out.lines.is_empty() {
            format!("\n\n🕯️ *Rocket's marginalia:* {}", out.lines.join(" — "))
        } else {
            self.flash_quests(out.lines);
            String::new()
        };
        self.codex.save();
        note
    }

    /// Recent chat history for the online API — the current question is
    /// appended separately by the caller.
    fn build_llm_history(&self) -> Vec<llm::Turn> {
        let mut msgs: Vec<&ChatMessage> = self.messages.iter().collect();
        // The last entry is the message being answered right now.
        if msgs.last().map(|m| m.role.as_str()) == Some("user") {
            msgs.pop();
        }
        msgs.iter()
            .rev()
            .filter(|m| m.role == "user" || m.role == "bot")
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| llm::Turn {
                from_user: m.role == "user",
                text: m.content.clone(),
                image_data_url: None,
            })
            .collect()
    }

    fn save_message_history(&self) {
        if let Ok(json) = serde_json::to_string(&self.messages) {
            if let Err(e) = fs::write("rocket_chat_history.json", json) {
                println!("Failed to save message history: {}", e);
            }
        }
    }
    
    fn load_message_history(&mut self) {
        if let Ok(content) = fs::read_to_string("rocket_chat_history.json") {
            if let Ok(messages) = serde_json::from_str::<Vec<ChatMessage>>(&content) {
                self.messages = messages;
            }
        }
    }
    
    fn clear_message_history(&mut self) {
        self.messages.clear();
        if let Err(e) = fs::remove_file("rocket_chat_history.json") {
            println!("Failed to clear message history: {}", e);
        }
    }

    fn get_timestamp(&self) -> String {
        chrono::Local::now().format("%H:%M").to_string()
    }

    fn get_cheat_sheet(&self) -> String {
        match self.current_game {
            GameType::Minecraft => {
                r#"🎮 MINECRAFT CHEAT SHEET 🎮

💎 DIAMOND FARMING:
• Best depth: y=-58 to -53 (near bedrock)
• Branch mining most effective
• Need iron pickaxe or better
• Always bring torches and food
• Diamond gear: best protection/tools

🏗️ BUILDING TIPS:
• Start with wood, then stone tools
• Use different block types for texture variation
• Build high for safety from mobs
• Create farms for automatic resource generation

⚔️ SURVIVAL GUIDE:
• Always carry food and build shelter before night
• Mining: Dig straight down at bedrock, branch mine at y=11
• Combat: Use a bow for distance, sword for close combat
• Armor: Iron minimum, diamond for endgame

🔧 REDSTONE BASICS:
• Redstone dust carries signals up to 15 blocks
• Repeaters boost signal and add delay
• Pistons move blocks when powered
• Learn simple circuits first (doors, traps)

🌟 PRO TIPS:
• Use maps to navigate large areas
• Create enchantment tables for better gear
• Build nether portals for fast travel
• Always keep backup tools and armor"#.to_string()
            },
            GameType::Fortnite => {
                r#"🎮 FORTNITE CHEAT SHEET 🎮

🏗️ BUILDING STRATEGIES:
• Edit structures quickly (90s, double edits)
• Ramp-rush for high ground advantage
• Use 1x1 for quick protection
• Practice editing in playground mode

🎯 WEAPON GUIDE:
• Assault Rifle: Mid-range, high damage
• Shotgun: Close range, one-shot potential
• Sniper Rifle: Long range, single shot
• SMG: Close range, high fire rate

🗺️ GAMEPLAY TIPS:
• Land at named locations for better loot
• Always carry shields and healing items
• Use sound to locate enemies
• Build for height advantage in fights

⚡ ADVANCED TECHNIQUES:
• Box fighting: Edit and shoot simultaneously
• Double editing: Fast structure replacement
• Ramp rushing: Quick vertical building
• Editing while moving for better positioning"#.to_string()
            },
            GameType::LeagueOfLegends => {
                r#"🎮 LEAGUE OF LEGENDS CHEAT SHEET 🎮

👥 ROLES & RESPONSIBILITIES:
• TOP: Tank/bruiser, split push
• JUNGLE: Farm, gank, objectives
• MID: Mage/assassin, roam
• ADC: Carry, farm, teamfight
• SUPPORT: Protect, engage, peel

🛡️ CORE MECHANICS:
• Last hitting for gold
• Vision control (wards, sweepers)
• Objective timing (dragons, baron)
• Wave management (freeze, push, slow)

🎯 CHAMPION TYPES:
• MAGES: Burst damage, ability reliant
• MARKSMEN: Auto-attack carry
• ASSASSINS: Burst squishy targets
• TANKS: Frontline, peel for team
• FIGHTERS: Balanced damage/tank

🏆 WIN CONDITIONS:
• Take objectives after teamfights
• Split push when enemy grouped
• Baron for siege pressure
• Dragon for team stats
• Inhibitor for super minions"#.to_string()
            },
            GameType::Valorant => {
                r#"🎮 VALORANT CHEAT SHEET 🎮

🔫 WEAPON GUIDE:
• Vandal: High damage, burst fire
• Phantom: Full auto, spray control
• Operator: One-shot sniper
• Marshal: Quickscope sniper
• Classic: Sidearm, low damage

🎯 AGENT ABILITIES:
• DUELIST: Fight and get kills (Jett, Reyna)
• INITIATOR: Start fights (Sova, Breach)
• CONTROLLER: Block areas (Viper, Omen)
• SENTINEL: Hold site (Sage, Cypher)

🎮 MAP CALLS:
• Enemy positions and numbers
• Ultimate usage and cooldowns
• Spike location and defuse status
• Ultimate economy and timing

⚡ AIM TIPS:
• Crosshair placement at head level
• Pre-aim common angles
• Use movement to make shots harder
• Tap firing for accuracy
• Spray control for close range

🎯 SPIKE STRATEGY:
• Plant in hidden corners
• Use utility to clear areas first
• Watch defuse timer
• Fake plants to catch defenders"#.to_string()
            },
            GameType::Skyrim => {
                r#"🏔️ SKYRIM CHEAT SHEET 🏔️

⚔️ COMBAT CLASSES:
• WARRIOR: Two-handed, one-handed, shield
• MAGE: Destruction, illusion, conjuration
• THIEF: Sneak, archery, one-handed
• CLASSIC: Balanced approach

🎒 SKILL PROGRESSION:
• Skills level up by using them
• Focus on 3-5 main skills
• Smithing for gear upgrades
• Enchanting for better items
• Alchemy for potions

🗺️ QUEST STRATEGIES:
• Main quest: Dragon souls essential
• Guild quests: Skill boosts and gear
• Side quests: Gold and experience
• Daedric quests: Unique artifacts

🏰 KEY LOCATIONS:
• Whiterun: Early game hub
• Riften: Thieves Guild
• Winterhold: College of Winterhold
• Darkwater Crossing: Early quest hub

🔮 DRAGON SHOUTS:
• Find Word Walls across Skyrim
• Need dragon souls to unlock
• Unrelenting Force: Push enemies away
• Fire Breath: Area damage
• Slow Time: Tactical advantage"#.to_string()
            },
            GameType::Roblox => {
                r#"🎮 ROBLOX CHEAT SHEET 🎮

🎮 GAME DEVELOPMENT:
• Use Roblox Studio for creating games
• Start simple: Obstacle courses, tycoons
• Learn scripting in Lua
• Use models for reusable components

💰 ROBUX EARNING:
• Create popular games
• Trade items carefully
• Game passes for extra revenue
• Group funds for shared games

🏗️ BUILDING TIPS:
• Use grid snapping for precision
• Group parts for performance
• Use meshes for detailed models
• Lighting affects game feel

🎯 POPULAR GAMES:
• Jailbreak: Police vs criminals
• Adopt Me: Pet simulation
• Tower of Hell: Obstacle course
• Natural Disaster Survival: Survival challenges

🎮 SCRIPTING BASICS:
• Variables store data
• Functions for reusable code
• Events for user interactions
• Loops for repetitive tasks
• Use LocalScripts for client-side code"#.to_string()
            },
            GameType::PUBG => {
                r#"🎮 PUBG CHEAT SHEET 🎮

🚗 VEHICLE STRATEGY:
• Cars: Fast but loud, attracts attention
• Motorcycles: Quick and quiet, good for flanking
• Bikes: Silent but slow, perfect for stealth
• Boats: Essential for water maps, use bridges carefully
• Buggies: Fast but exposed, use cover when driving

🎯 LOOTING STRATEGY:
• Prioritize: Armor, helmet, backpack, weapons
• Level 3 armor is rare but worth finding
• Backpack size affects loot capacity
• Attachments are crucial for weapon performance
• Medical supplies are more important than ammo early game

🗺️ MAP CONTROL:
• Stay near the edge of the circle early game
• Use vehicles to rotate safely between zones
• High ground provides better visibility and positioning
• Buildings offer cover but are risky in late game
• Water routes can be safer but slower

⚡ COMBAT TIPS:
• Third person gives better awareness than first person
• Use cover and peek carefully
• Sound is crucial - listen for footsteps and vehicles
• Grenades can flush enemies out of buildings
• Revive teammates quickly but safely

🎯 ENDGAME STRATEGY:
• Stay mobile and avoid staying in one spot
• Use the blue zone to your advantage
• Build fighting positions with available materials
• Watch the sky for enemy rotations
• Coordinate with squad for better positioning

📱 SQUAD COMMUNICATION:
• Call out enemy positions clearly
• Share loot and coordinate looting
• Revive order: prioritize key players first
• Use ping system effectively
• Stay in voice contact for better coordination"#.to_string()
            },
            _ => {
                format!("🎮 {} CHEAT SHEET 🎮

🎯 GENERAL GAMING TIPS:
• Practice regularly to improve skills
• Watch pro players for strategies
• Learn maps and callouts
• Communicate with teammates
• Take breaks to avoid burnout

🛠️ SETUP OPTIMIZATION:
• Comfortable seating and posture
• Proper lighting and ergonomics
• Good internet connection
• Updated hardware drivers
• Clean gaming environment

📈 IMPROVEMENT STRATEGY:
• Focus on one skill at a time
• Record and review gameplay
• Join communities for tips
• Set realistic goals
• Track progress over time

⚡ MENTAL GAME:
• Stay positive and focused
• Learn from mistakes
• Avoid tilt and frustration
• Balance gaming with life
• Have fun and enjoy the process", self.current_game.get_name())
            }
        }
    }

    fn get_ai_response(&mut self, user_message: &str) -> String {
        let lower_message = user_message.to_lowercase();
        self.last_was_item_answer = false;

        // Conversational layer: react to build/loadout talk like a person
        // ("i'm going for a mage build…") before answering the question part.
        let build_ack = self.memory.capture_build_talk(user_message);
        let is_question = memory::message_is_question(user_message);
        let opener = match &build_ack {
            Some(ack) if is_question => format!("{}\n\n", ack),
            _ => String::new(),
        };
        if let Some(ack) = build_ack {
            if !is_question {
                // Pure self-statement: acknowledge it, then be useful.
                let hint = self.memory.context_hint().unwrap_or_default().to_string();
                let hits = self.kb.search(&self.current_game, &hint, 1);
                if hits.is_empty() {
                    return format!(
                        "{}\n\nAsk me anything about it while you're at it — builds, items, \
                         whatever you need 🚀",
                        ack
                    );
                }
                return format!(
                    "{}\n\nSince you brought it up — here's something useful:\n\n{}",
                    ack,
                    knowledge::format_chat_answer(&hits)
                );
            }
        }

        // Personality: vibe switches ("be funny").
        if let Some(vibe_id) = vibe::parse_vibe_command(user_message) {
            let intro = self.memory.set_vibe(vibe_id);
            self.memory.save();
            return intro;
        }

        // Personality: celebrate wins ("i finally beat the ender dragon").
        if memory::parse_mission_completion(user_message).is_some() {
            let vibe = self.memory.current_vibe();
            match self.memory.complete_mission_by_text(user_message) {
                Some(mission) => {
                    let (leveled, _) = self.memory.leveling.add_experience(75);
                    let lines = self.memory.check_progress_rewards();
                    if leveled {
                        self.show_level_up = true;
                        self.level_up_details = format!(
                            "🎉 LEVEL UP! You reached level {}!",
                            self.memory.leveling.current_level
                        );
                    }
                    if !lines.is_empty() {
                        self.flash_quests(lines);
                    }
                    self.memory.save();
                    let remaining = self.memory.active_missions().len();
                    let next = if remaining > 0 {
                        format!(
                            "\n\n📋 {} mission(s) still on our board — what's next, partner?",
                            remaining
                        )
                    } else {
                        "\n\n📋 Board's clear — got another one for us? 🎯".to_string()
                    };
                    let done_line = vibe::fill(
                        vibe::line(vibe.mission_done),
                        "{mission}",
                        &format!("{} ({})", mission.title, mission.game),
                    );
                    return format!("{}\n\n+75 XP for the win!{}", done_line, next);
                }
                None => {
                    // Untracked win — still celebrate like a teammate.
                    let (leveled, _) = self.memory.leveling.add_experience(25);
                    if leveled {
                        self.show_level_up = true;
                        self.level_up_details = format!(
                            "🎉 LEVEL UP! You reached level {}!",
                            self.memory.leveling.current_level
                        );
                    }
                    self.memory.save();
                    return format!(
                        "🏆 LET'S GO — big win, even if it wasn't on our board! 🎉\n\n+25 XP\n\n{}",
                        vibe::line(vibe.tails)
                    );
                }
            }
        }

        // Personality: co-op missions ("let's beat the ender dragon").
        if let Some(target) = memory::parse_mission_creation(user_message) {
            let already = self
                .memory
                .active_missions()
                .iter()
                .any(|m| m.title.eq_ignore_ascii_case(&target));
            let created = if already {
                false
            } else {
                self.memory
                    .create_mission(&target, self.current_game.get_name())
            };
            let vibe = self.memory.current_vibe();
            let head = if created {
                vibe::fill(vibe::line(vibe.mission_created), "{mission}", &target)
            } else if already {
                format!("📋 \"{}\" is already on our mission board — we're on it, partner!", target)
            } else {
                "📋 Mission board is full — let's finish something first, then I'm all yours 🎯"
                    .to_string()
            };
            self.memory.save();
            // Our opening move comes from the game's own knowledge.
            let hits = self.kb.search(&self.current_game, &target, 1);
            let plan = if hits.is_empty() {
                "Our roadmap: main objectives first, upgrade gear between chapters, keep heals/ammo stocked. Ask me about any step and I'll break it down.".to_string()
            } else {
                knowledge::format_chat_answer(&hits)
            };
            return format!("{}\n\n{}", head, plan);
        }

        // Check for follow-up questions
        if lower_message.contains("more") || lower_message.contains("additional") || lower_message.contains("another") {
            // Get the last answered topic and provide additional facts
            if let Some(last_topic) = &self.last_answered_topic {
                // Get additional facts from the same topic
                let additional_kb_hits = self.kb.search(&self.current_game, last_topic, 2);
                if !additional_kb_hits.is_empty() {
                    // Return the second fact if available
                    if additional_kb_hits.len() > 1 {
                        return format!(
                            "{}{}",
                            opener,
                            knowledge::format_chat_answer(&[additional_kb_hits[1]])
                        );
                    }
                }

                // If no additional KB hits, ask for more details
                return format!(
                    "Want more details on that? Try asking about specific aspects like '{}', '{}', or '{}'.",
                    last_topic, "tips", "advanced techniques"
                );
            }
        }

        // Personality: occasional vibe-flavored banter after an answer.
        let vibe = self.memory.current_vibe();
        let with_tail = |text: String| -> String {
            if rand::random::<f32>() < 0.25 {
                format!("{}\n\n{}", text, vibe::line(vibe.tails))
            } else {
                text
            }
        };

        // Knowledge Base first: ground the answer in curated game data.
        match knowledge_answer(&self.kb, &self.current_game, user_message) {
            Some(KnowledgeAnswer::Game(answer, first_tag)) => {
                if !first_tag.is_empty() {
                    self.last_answered_topic = Some(first_tag.to_string());
                }
                let tag = if first_tag.is_empty() { None } else { Some(first_tag) };
                let note = self.codex_answer_hook(tag, user_message, true);
                return format!("{}{}{}", opener, with_tail(answer), note);
            }
            Some(KnowledgeAnswer::CrossGame(answer)) => {
                let note = self.codex_answer_hook(None, user_message, true);
                return format!("{}{}{}", opener, answer, note);
            }
            Some(KnowledgeAnswer::UniversalItem(answer)) => {
                self.last_was_item_answer = true;
                self.last_answered_topic = None;
                let note = self.codex_answer_hook(None, user_message, true);
                return format!("{}{}{}", opener, with_tail(answer), note);
            }
            None => {
                // Ungrounded answer — no scholarship credit; the ordeal run
                // (if any) resets, recorded without punishment.
                let _ =
                    self.codex
                        .record_realm_question(&self.current_game, None, user_message, false);
                self.codex.save();
            }
        }

        // Apply skill level difficulty scaling
        let difficulty_prefix = match self.memory.skill_level {
            memory::SkillLevel::Beginner => "🟢 BEGINNER TIP: ",
            memory::SkillLevel::Intermediate => "🟡 INTERMEDIATE TIP: ",
            memory::SkillLevel::Advanced => "🔴 ADVANCED TIP: ",
        };

        // Game-specific responses with difficulty scaling
        let game_response = match self.current_game {
            GameType::Minecraft => {
                if lower_message.contains("diamond") || lower_message.contains("diamonds") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Diamonds are your first major goal! Start by looking for caves or strip mining at y-level -58. You'll need an iron pickaxe to mine them. Don't forget to bring torches and food - caves can be dangerous!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Diamonds spawn between y=-64 and y=-58, with -58 being optimal. Use branch mining for consistent results. Fortune III can double or triple your yield. Always have water buckets ready for lava pockets!", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Optimal diamond strategy: bed mining at y=-58 for maximum efficiency. Fortune III + Efficiency V is ideal for speed. Consider using a beacon with Haste II for large-scale operations. Ancient Debris mining should be your next priority after diamond gear.", difficulty_prefix),
                    }
                } else if lower_message.contains("netherite") || lower_message.contains("ancient debris") || lower_message.contains("debris") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Ancient Debris is found in the Nether. Start by exploring caves around y-level 15. It's blast-proof, so you can mine it safely. Smelt it to get Netherite Scrap!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Ancient Debris spawns most commonly at y=15 (range 8-22). Use bed mining for efficiency: tunnel at y=15, place bed, trigger explosion, repeat. Smelt scraps + gold ingots → Netherite Ingots. Upgrade diamond gear at Smithing Table!", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Advanced Netherite strategy: strip-mine at y=15 with Efficiency V. Bastion Remnants contain Smithing Templates for diamond → Netherite upgrades. Netherite gear has 110% durability, fire resistance, and knockback reduction. Consider Blast Protection IV for endgame gear.", difficulty_prefix),
                    }
                } else if lower_message.contains("build") || lower_message.contains("construction") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Start simple! Gather wood, make a crafting table, then build a basic house. Try different block types for variety. Redstone comes later - focus on survival first!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Experiment with architecture! Use different materials for texture contrast. Learn basic redstone for automatic doors, farms, and lighting. Consider symmetry and scale in your builds.", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Advanced building: Use WorldEdit for large projects, master complex redstone systems (computers, farms), and explore architectural styles. Consider lighting optimization, mob farms, and automation systems for maximum efficiency.", difficulty_prefix),
                    }
                } else if lower_message.contains("survival") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Survival basics: Punch trees → wood → crafting table → wooden tools → stone tools. Find a village for beds and food. Build shelter before night 1. Always carry food!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Survival strategy: Set up efficient farms (wheat, animals, iron). Create a base with storage, enchanting table, and nether portal. Explore for resources but keep backup gear. Always have a bed and compass!", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Endgame survival: Create automated farms for all resources. Build a beacon for potion effects. Explore all dimensions systematically. Set up item sorting systems. Always have backup gear in different locations.", difficulty_prefix),
                    }
                } else if lower_message.contains("redstone") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Redstone basics: Redstone dust carries power 15 blocks. Repeaters extend distance and add delay. Start with simple doors, traps, and piston doors. Watch YouTube tutorials!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Redstone mastery: Learn comparators, observers, and hoppers. Build automatic farms, sorting systems, and complex contraptions. Understand redstone torch behavior and block update mechanics.", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Advanced redstone: Create computers, calculators, and complex automation systems. Master command blocks and functions for massive builds. Understand game mechanics deeply for innovative contraptions.", difficulty_prefix),
                    }
                } else if lower_message.contains("nether") || lower_message.contains("fortress") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Nether basics: Bring gold armor to avoid piglin hostility. Find Nether Fortresses for Blaze Rods. Build portals carefully - watch for lava! Bring fire resistance potions for long journeys.", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Nether strategy: Use gold armor strategically (only one piece needed). Build safe paths with blocks. Nether Fortresses have specific layouts - learn them. Bastion Remnants contain gold and ancient debris.", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Expert Nether: Create highway systems for efficient travel. Use beds for explosive mining. Master fortress navigation and blaze farming. Set up base operations near multiple fortress locations for efficiency.", difficulty_prefix),
                    }
                } else if lower_message.contains("enderman") || lower_message.contains("end") {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Endermen: Look at their legs, not their eyes! Carved pumpkins let you stare safely. In the End, bring beds (they explode!) and lots of ender pearls. The Ender Dragon fight is challenging!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Enderman strategy: Use water to push them away. In the End, destroy crystals first, then attack the dragon while it's perched. Ender Pearls are valuable - collect as many as possible. Consider building a fighting arena.", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}Expert End game: Create a dragon-fighting arena with beds for explosions. Use crystal strategies and efficient pearl farming. After defeating the dragon, explore outer islands for shulkers and end cities. Consider building an end farm system.", difficulty_prefix),
                    }
                } else {
                    match self.memory.skill_level {
                        memory::SkillLevel::Beginner => format!("{}Welcome to Minecraft! Let me help you get started. What would you like to learn about? I can teach you about survival, building, or finding resources like diamonds!", difficulty_prefix),
                        memory::SkillLevel::Intermediate => format!("{}Minecraft offers endless possibilities! What aspect interests you most? Redstone automation, advanced building techniques, endgame preparation, or exploring new mods and updates?", difficulty_prefix),
                        memory::SkillLevel::Advanced => format!("{}As an experienced player, you're ready for complex challenges. Interested in redstone computers, command block contraptions, speedrunning strategies, or mastering specific game mechanics? I can provide advanced insights!", difficulty_prefix),
                    }
                }
            },
            GameType::Roblox => {
                if lower_message.contains("game") || lower_message.contains("create") {
                    "Roblox is all about creativity! Try using Roblox Studio to make your own games. Start simple with obstacle courses or tycoon games!".to_string()
                } else if lower_message.contains("robux") {
                    "Robux can be earned through creating popular games or trading items. Be careful with scams - only buy from official Roblox sites!".to_string()
                } else {
                    "Roblox offers endless possibilities! What type of games do you enjoy playing or creating?".to_string()
                }
            },
            GameType::Fortnite => {
                if lower_message.contains("building") {
                    "Building is key in Fortnite! Practice editing structures quickly. Try 90s, double edits, and ramp-rush techniques to improve your gameplay!".to_string()
                } else if lower_message.contains("weapon") {
                    "Weapon drops matter! Try landing at Tilted or Retail Row for better loot. Aim for blue or purple weapons, and don't forget shields!".to_string()
                } else {
                    "Fortnite combines building and shooting! What aspect do you want to improve - building skills, aim, or strategy?".to_string()
                }
            },
            GameType::LeagueOfLegends => {
                if lower_message.contains("champion") {
                    "Choose champions that fit your playstyle! If you're new, try Garen, Annie, or Ashe. Learn their abilities and practice in the practice mode!".to_string()
                } else if lower_message.contains("strategy") {
                    "Map awareness is crucial! Watch the minimap, communicate with your team, and objectives over kills. Baron and Dragon win games!".to_string()
                } else {
                    "League of Legends is a team-based MOBA! What role or champion would you like advice on?".to_string()
                }
            },
            GameType::Valorant => {
                if lower_message.contains("aim") {
                    "Practice aim in the range! Use different crosshairs and try flick shots. Learning spray patterns is essential for consistent shooting!".to_string()
                } else if lower_message.contains("agent") {
                    "Agents have unique abilities! For beginners, try Reyna for dueling or Sage for support. Learn each agent's role and abilities!".to_string()
                } else {
                    "Valorant is tactical FPS! Work on aim, game sense, and agent compositions. Communication wins matches!".to_string()
                }
            },
            GameType::PUBG => {
                if lower_message.contains("vehicle") || lower_message.contains("car") || lower_message.contains("bike") {
                    "Vehicles in PUBG: Cars are fast but loud, motorcycles are quick and quiet for flanking, bikes are silent but slow for stealth. Use cover when driving and be careful with boat routes on water maps!".to_string()
                } else if lower_message.contains("loot") || lower_message.contains("gear") {
                    "Looting strategy: Prioritize armor, helmet, backpack, and weapons. Level 3 armor is rare but worth finding. Attachments are crucial for weapon performance. Medical supplies are more important than ammo early game!".to_string()
                } else if lower_message.contains("circle") || lower_message.contains("zone") {
                    "Circle strategy: Stay near the edge early game, use vehicles to rotate safely between zones. High ground provides better visibility. Blue zone can be used to your advantage - plan your rotations carefully!".to_string()
                } else if lower_message.contains("combat") || lower_message.contains("fight") {
                    "Combat tips: Use third person for better awareness, use cover and peek carefully. Sound is crucial - listen for footsteps and vehicles. Grenades can flush enemies out of buildings. Revive teammates quickly but safely!".to_string()
                } else if lower_message.contains("endgame") || lower_message.contains("final") {
                    "Endgame strategy: Stay mobile and avoid staying in one spot. Use blue zone to your advantage. Build fighting positions with available materials. Watch the sky for enemy rotations. Coordinate with squad for better positioning!".to_string()
                } else {
                    "PUBG is about survival and strategy! What aspect do you want to improve - vehicle tactics, looting, circle management, combat skills, or endgame positioning?".to_string()
                }
            },
            GameType::Skyrim => {
                if lower_message.contains("build") {
                    "Skyrim builds matter! Choose your playstyle - warrior, mage, or thief. Skills level up based on usage, so focus on what you enjoy!".to_string()
                } else if lower_message.contains("quest") {
                    "Main story leads you through the game! Side quests offer great rewards and world-building. Guild quests provide unique gear and stories!".to_string()
                } else {
                    "Skyrim offers endless adventures! What type of character are you playing - warrior, mage, or stealth specialist?".to_string()
                }
            },
            _ => {
                // Generic game responses
                if lower_message.contains("help") {
                    "I'm here to help! What specific gaming question do you have?".to_string()
                } else if lower_message.contains("strategy") {
                    "Strategy is key! Watch pro players, study maps, and practice your mechanics. Every game has its own meta - stay updated!".to_string()
                } else if lower_message.contains("tip") {
                    "Here's a pro tip: Always know your surroundings and communicate with your team! Practice makes perfect in any game!".to_string()
                } else if lower_message.contains("hello") || lower_message.contains("hi") {
                    let greeting = match &self.memory.user_name {
                        Some(name) => format!("Hello, {}!", name),
                        None => String::from("Hello!"),
                    };
                    // Enhanced greeting logic based on current mode
                    if self.current_game == GameType::Generic {
                        // General conversation mode - more friendly and open
                        if lower_message.len() < 10 && (lower_message == "hello" || lower_message == "hi" || lower_message == "hey") {
                            let greetings = [
                                format!("{} 👋 I'm ROCKET, your AI assistant! What's on your mind today?", greeting),
                                format!("{} 🤓 Great to chat with you! What would you like to talk about?", greeting),
                                format!("{} 💬 Nice to meet you! I can help with questions, chat about any topic, or just have a conversation!", greeting),
                                format!("{} 🌟 Hello there! Feel free to ask me anything - from serious questions to fun conversations!", greeting),
                            ];
                            greetings[rand::random::<usize>() % greetings.len()].to_string()
                        } else {
                            format!("{} I'm ROCKET, your personal AI assistant! How can I help you with {} today?", greeting, self.current_game.get_name())
                        }
                    } else {
                        // Gaming mode - game-specific responses
                        if lower_message.len() < 10 && (lower_message == "hello" || lower_message == "hi" || lower_message == "hey") {
                            format!("{} 🎮 I'm your gaming AI companion for {}! What game shall we dive into?", greeting, self.current_game.get_name())
                        } else {
                            format!("{} I'm your gaming AI companion for {}. What would you like to know?", greeting, self.current_game.get_name())
                        }
                    }
                } else {
                    // Enhanced generic responses based on current mode
                    if self.current_game == GameType::Generic {
                        let general_responses = [
                            "I'm here to help with any questions or topics you have! What would you like to discuss?".to_string(),
                            "Feel free to ask me anything! I can help with information, creative ideas, problem-solving, or just have a friendly chat.".to_string(),
                            "What's on your mind today? I'm ready to help with questions, advice, or conversation on any topic you choose!".to_string(),
                            "I'm your AI assistant for everything! Ask me questions, seek advice, or let's chat about whatever interests you.".to_string(),
                        ];
                        general_responses[rand::random::<usize>() % general_responses.len()].to_string()
                    } else {
                        let context_note = match self.memory.context_hint() {
                            Some(ctx) => format!(" And since you're running {}, ask me about that too!", ctx),
                            None => String::new(),
                        };
                        format!("I know a lot about {}! Ask about a specific topic — like 'beginner tips', 'best loadout', or a mechanic you're stuck on — or open 📚 Knowledge to browse every guide I have. You can also try 📋 Cheat Sheet for a full overview.{}", self.current_game.get_name(), context_note)
                    }
                }
            }
        };
        
        // General conversation handlers for non-gaming mode
        if self.current_game == GameType::Generic {
            // Handle common conversation topics
            if ["how are you", "how are you doing", "how's it going", "hows it going", "you good", "how do you feel"].contains(&lower_message.as_str()) {
                let responses = [
                    "I'm doing great, thank you for asking! As an AI, I'm always ready to help and learn from our conversations. How about you?",
                    "Running smoothly and ready to assist! I enjoy helping with questions and having interesting conversations. What's been happening with you?",
                    "Feeling fantastic! Every conversation helps me learn and improve. How's your day going?",
                    "Great! I'm excited to help you with whatever you need. What's on your mind today?",
                ];
                return responses[rand::random::<usize>() % responses.len()].to_string();
            }
            
            if ["thanks", "thank you", "thx", "ty", "appreciate it", "thanks rocket"].contains(&lower_message.as_str()) {
                let responses = [
                    "You're very welcome! I'm glad I could help.",
                    "Happy to assist! Feel free to ask me anything anytime.",
                    "No problem at all! That's what I'm here for.",
                    "My pleasure! Let me know if you need more help.",
                ];
                return responses[rand::random::<usize>() % responses.len()].to_string();
            }
            
            if ["bye", "goodbye", "cya", "see you", "see ya", "good night", "gn", "later"].contains(&lower_message.as_str()) {
                let responses = [
                    "Goodbye! It was nice chatting with you. Feel free to come back anytime!",
                    "See you later! Have a great day!",
                    "Take care! I'll be here whenever you need help.",
                    "Goodbye! Hope our conversation was helpful. Talk soon!",
                ];
                return responses[rand::random::<usize>() % responses.len()].to_string();
            }
            
            if ["what can you do", "what are your capabilities", "help me understand", "tell me about yourself"].contains(&lower_message.as_str()) {
                return "I'm ROCKET, your AI assistant! I can help with:
• Answering questions on any topic
• Providing information and explanations
• Creative writing and brainstorming
• Problem-solving and advice
• Having friendly conversations
• Gaming help and tips (when in gaming mode)
• Learning and educational topics
• Research and analysis

Just ask me anything! I'm always ready to help.".to_string();
            }
        }
        
        // Add some variety
        if rand::random::<f32>() < 0.3 {
            // Add a random encouraging message
            let encouragements = [
                "Keep practicing - you're getting better!",
                "Great question! Let me help you with that.",
                "Gaming is all about learning and having fun!",
                "You've got this! Stay focused and keep trying.",
            ];
            return format!("{}\n\n{}", game_response, encouragements[rand::random::<usize>() % encouragements.len()]);
        }
        
        game_response
    }
}

/// Makes the online LLM call with fully-owned data captured from the real
/// app state (selected game, KB grounding, chat history, optional picture).
async fn call_ai_api(
    settings: llm::Settings,
    game: GameType,
    user_message: String,
    history: Vec<llm::Turn>,
    kb_context: String,
    memory_context: String,
    image_data_url: Option<String>,
) -> Result<String, String> {
    if !settings.is_ready() {
        return Err(
            "🔑 AI API not configured — add your API key in settings, or toggle AI API OFF to use the local brain."
                .to_string(),
        );
    }

    let game_context = format!(
        "You are a gaming AI assistant specialized in {}. The user is asking about: {}{}{}",
        game.get_name(),
        user_message,
        kb_context,
        memory_context
    );

    let mut full_history = vec![llm::Turn {
        from_user: false,
        text: game_context,
        image_data_url: None,
    }];
    full_history.extend(history);
    // The current question goes last, carrying the picture if one was attached.
    full_history.push(llm::Turn {
        from_user: true,
        text: user_message,
        image_data_url,
    });

    let system_prompt = llm::system_prompt();
    match llm::chat(settings, system_prompt, full_history).await {
        Ok(response) => Ok(response),
        Err(error) => Err(friendly_api_error(&error)),
    }
}

/// Translate raw provider errors into helpful, human messages.
fn friendly_api_error(error: &str) -> String {
    let msg = if error.contains("API key not set") {
        "🔑 No API key found. Add your key in the AI API settings."
    } else if error.contains("Network error") {
        "🌐 Network error. Check your internet connection and try again."
    } else if error.contains("HTTP 401") || error.to_lowercase().contains("unauthorized") {
        "🔑 Invalid API key. Double-check your key and provider."
    } else if error.contains("HTTP 429") {
        "⏰ Rate limit exceeded. Wait a moment and try again."
    } else if error.contains("HTTP 403") {
        "🚫 API access forbidden. The provider may have blocked this key."
    } else if error.contains("HTTP 500") || error.contains("server error") {
        "🔧 Server error on the provider side. Try again later."
    } else if error.contains("No custom URL set") {
        "🔗 No custom endpoint set. Add your OpenAI-compatible URL in settings."
    } else {
        return format!("❌ AI API Error: {}", error);
    };
    msg.to_string()
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::InputChanged(value) => {
            state.input_text = value;
            Task::none()
        }
        Message::Send => {
            if state.input_text.trim().is_empty() {
                return Task::none();
            }
            // Roll a fresh quest set if the calendar day changed.
            state.memory.ensure_daily_quests();
            let user_text = state.input_text.clone();
            let attachment = state.attached_image.take();

            // Show an attachment marker on the user's bubble.
            let display_text = match &attachment {
                Some(att) => format!("📎 [{}]\n{}", att.file_name, user_text),
                None => user_text.clone(),
            };

            // Records the message, awards conversation/streak XP.
            state.add_message("user".to_string(), display_text);

            // Personal memory capture — teaching ROCKET about yourself
            // short-circuits the AI flow with a confirmation reply.
            if let Some(confirmation) = state.memory.capture(&user_text) {
                let streak_days = memory::update_streak(&mut state.memory);
                if streak_days > 1 {
                    let (leveled, _) = state.memory.leveling.award_streak_xp(streak_days);
                    if leveled {
                        state.show_level_up = true;
                        state.level_up_details = format!(
                            "🎉 LEVEL UP! You reached level {}!",
                            state.memory.leveling.current_level
                        );
                    }
                    state.show_xp_gain = true;
                    state.xp_gain_details =
                        format!("🔥 Streak bonus — {} days in a row! Keep it going!", streak_days);
                }
                let (leveled, _) = state.memory.award_teaching_xp();
                if leveled {
                    state.show_level_up = true;
                    state.level_up_details = format!(
                        "🎉 LEVEL UP! You reached level {}!",
                        state.memory.leveling.current_level
                    );
                }
                state.show_xp_gain = true;
                state.xp_gain_details = "🧠 +25 XP for teaching me something new!".to_string();
                let teach_lines = state.memory.quest_progress("teach", 1);
                state.flash_quests(teach_lines);
                // Codex: teaching feeds Restoration quests and the chronicle.
                let codex_teach_lines = state.codex.record_teaching(&state.current_game);
                state.flash_quests(codex_teach_lines);
                // 🌳 CREATE TREE — teaching advances Restoration tree nodes too.
                let tree_out = state.codex.tree_teach_event(&state.current_game);
                if tree_out.xp > 0 {
                    let (leveled, _) = state.memory.leveling.add_experience(tree_out.xp);
                    if leveled {
                        state.show_level_up = true;
                        state.level_up_details =
                            codex::proclaim_level_up(state.memory.leveling.current_level);
                    }
                }
                state.flash_quests(tree_out.lines);
                state.codex.save();
                state.memory.ensure_defaults();
                state.flash_progress_rewards();
                state.memory.save();
                state.add_message("bot".to_string(), confirmation);
                state.input_text.clear();
                return Task::none();
            }

            // Online AI path — real LLM (and the only path that can see
            // attached pictures). Falls back to the local brain otherwise.
            if state.use_ai_api && state.llm_settings.is_ready() {
                let settings = state.llm_settings.clone();
                let game = state.current_game.clone();
                let history = state.build_llm_history();
                let kb_context = {
                    let hits = state.kb.search(&state.current_game, &user_text, 2);
                    if hits.is_empty() {
                        String::new()
                    } else {
                        format!("\n\nRelevant knowledge:\n{}", knowledge::format_answer(&hits))
                    }
                };
                let memory_context = state.memory.recall_context();
                let image = attachment.map(|att| att.data_url);
                state.is_typing = true;
                state.input_text.clear();
                return Task::perform(
                    async move {
                        call_ai_api(
                            settings,
                            game,
                            user_text,
                            history,
                            kb_context,
                            memory_context,
                            image,
                        )
                        .await
                    },
                    |result| match result {
                        Ok(reply) => Message::ApiResponse(reply),
                        Err(error) => Message::ApiResponse(error),
                    },
                );
            }

            // A picture was attached but online AI isn't available.
            if attachment.is_some() {
                state.add_message(
                    "bot".to_string(),
                    "📷 I can't see images in Local AI mode yet! Turn on 🤖 AI API and add a vision-capable key (Groq, OpenAI, OpenRouter…), then send the picture again.".to_string(),
                );
                state.input_text.clear();
                return Task::none();
            }

            let bot_response = state.get_ai_response(&user_text);

            // Answering from the knowledge base → mastery XP for new topics.
            if let Some(topic) = state.last_answered_topic.clone() {
                let (leveled, _) = state.memory.award_mastery_xp(&topic);
                if leveled {
                    state.show_level_up = true;
                    state.level_up_details = format!(
                        "🎉 LEVEL UP! You reached level {}!",
                        state.memory.leveling.current_level
                    );
                }
            }

            // Follow-up questions earn a little extra too.
            let lower = user_text.to_lowercase();
            if ["more", "additional", "another"].iter().any(|k| lower.contains(k)) {
                let (leveled, _) = state.memory.award_followup_xp();
                if leveled {
                    state.show_level_up = true;
                    state.level_up_details = format!(
                        "🎉 LEVEL UP! You reached level {}!",
                        state.memory.leveling.current_level
                    );
                }
            }

            // Conversation record: +10 XP and achievement/badge milestones.
            let (leveled, _) = state.memory.add_conversation(
                user_text.clone(),
                bot_response.clone(),
                state.last_answered_topic.clone(),
            );
            if leveled {
                state.show_level_up = true;
                state.level_up_details = format!(
                    "🎉 LEVEL UP! You reached level {}!",
                    state.memory.leveling.current_level
                );
                state.show_xp_gain = true;
                state.xp_gain_details = "✨ +10 XP for asking a question!".to_string();
            }
            state.memory.check_milestone_badges();
            // Daily quests: asking + item questions.
            let mut quest_lines = state.memory.quest_progress("ask", 1);
            if state.last_was_item_answer {
                quest_lines.extend(state.memory.quest_progress("item", 1));
                state.last_was_item_answer = false;
            }
            state.flash_quests(quest_lines);
            state.flash_progress_rewards();
            state.memory.save();

            state.add_message("bot".to_string(), bot_response);
            state.input_text.clear();
            Task::none()
        }
        Message::DetectGameTick => {
            if !state.auto_detect {
                return Task::none();
            }
            // Scan OFF the UI thread — `tasklist` can take hundreds of ms and
            // used to freeze the whole window every tick.
            Task::perform(
                async {
                    tokio::task::spawn_blocking(detect::scan_running_game)
                        .await
                        .unwrap_or(None)
                },
                Message::DetectedGame,
            )
        }
        Message::DetectedGame(game) => {
            match game {
                Some(game) => {
                    let changed = state.detected.as_ref() != Some(&game);
                    if changed {
                        state.detected = Some(game.clone());
                    }
                    // Don't yank the context away from a manual pick.
                    if changed && !state.manual_pick && game != state.current_game {
                        state.current_game = game.clone();
                        // Transient banner — flashes, then vanishes by itself.
                        state.detection_banner =
                            Some(format!("🎯 {} detected — switching context!", game.get_name()));
                        let vibe = state.memory.current_vibe();
                        state.add_message(
                            "system".to_string(),
                            vibe::fill(vibe::line(vibe.game_switch), "{game}", game.get_name()),
                        );
                        return Task::perform(
                            async {
                                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                            },
                            |_| Message::ClearDetectionBanner,
                        );
                    }
                    Task::none()
                }
                None => {
                    state.detected = None;
                    if state.current_game != GameType::Generic {
                        state.current_game = GameType::Generic;
                        state.add_message(
                            "system".to_string(),
                            "🎯 No game detected — switched to general mode".to_string(),
                        );
                    }
                    Task::none()
                }
            }
        }
        Message::ToggleAutoDetect => {
            state.auto_detect = !state.auto_detect;
            detect::save_enabled(state.auto_detect);
            if state.auto_detect {
                match detect::scan_running_game() {
                    Some(game) => {
                        state.detected = Some(game.clone());
                        if game != state.current_game {
                            state.current_game = game.clone();
                            state.add_message(
                                "system".to_string(),
                                format!("🎯 Auto-detection enabled — detected {} running!", game.get_name()),
                            );
                        } else {
                            state.add_message(
                                "system".to_string(),
                                "🎯 Auto-detection enabled".to_string(),
                            );
                        }
                    }
                    None => {
                        state.detected = None;
                        state.add_message(
                            "system".to_string(),
                            "🎯 Auto-detection enabled — no game currently running".to_string(),
                        );
                    }
                }
            } else {
                state.add_message("system".to_string(), "🎯 Auto-detection disabled".to_string());
            }
            Task::none()
        }
        Message::ToggleConversationMode => {
            if state.current_game == GameType::Generic {
                if let Some(game) = detect::scan_running_game() {
                    state.current_game = game.clone();
                    state.detected = Some(game.clone());
                    state.manual_pick = true;
                    state.add_message(
                        "system".to_string(),
                        format!("🎮 Switched to gaming mode — {} detected!", game.get_name()),
                    );
                } else {
                    state.current_game = GameType::Minecraft;
                    state.manual_pick = true;
                    state.add_message(
                        "system".to_string(),
                        "🎮 Gaming mode — no game running, defaulting to Minecraft. Tap the banner to pick another game!".to_string(),
                    );
                }
            } else {
                state.current_game = GameType::Generic;
                state.add_message(
                    "system".to_string(),
                    "💬 Switched to general conversation mode — I'm now your AI assistant for any topic!".to_string(),
                );
            }
            Task::none()
        }
        Message::ChangeGame(game) => {
            if state.current_game != game {
                // Reward exploring new games.
                let (leveled, _) = state.memory.award_game_exploration_xp();
                if leveled {
                    state.show_level_up = true;
                    state.level_up_details = format!(
                        "🎉 LEVEL UP! You reached level {}!",
                        state.memory.leveling.current_level
                    );
                }
                state.current_game = game.clone();
                state.detected = Some(game.clone());
                state.manual_pick = true;
                let vibe = state.memory.current_vibe();
                state.add_message(
                    "system".to_string(),
                    vibe::fill(vibe::line(vibe.game_switch), "{game}", game.get_name()),
                );
                // Explore quest: count distinct games tried today.
                if !state.memory.games_today.contains(&game.get_name().to_string()) {
                    state.memory.games_today.push(game.get_name().to_string());
                    let lines = state.memory.quest_progress("explore", 1);
                    state.flash_quests(lines);
                }
                state.memory.save();
            }
            // Picking a game takes you back to the conversation.
            state.page = Page::Chat;
            Task::none()
        }
        Message::Nav(page) => {
            state.page = page;
            if page == Page::Knowledge {
                let lines = state.memory.quest_progress("kb", 1);
                state.flash_quests(lines);
            }
            if page == Page::Scriptorium {
                let level = state.memory.leveling.current_level;
                let pinned = state.codex.ensure_board(&state.kb, level);
                state.flash_quests(pinned);
                state.codex.save();
            }
            if page == Page::Profile {
                state.profile_nickname_input = state.memory.nickname.clone().unwrap_or_default();
                state.profile_bio_input = state.memory.bio.clone().unwrap_or_default();
            }
            Task::none()
        }
        Message::DeskInputChanged(value) => {
            state.desk_input = value;
            Task::none()
        }
        Message::DeskSend => {
            let input = std::mem::take(&mut state.desk_input);
            if !input.trim().is_empty() {
                let level = state.memory.leveling.current_level;
                let trees_before = state.codex.trees.len();
                // Reminders ("remind me …") route through Rocket's lore first.
                if state.codex.re_explain(&input) {
                    // A reminder was served by Rocket himself.
                } else {
                    let _ = state.codex.desk_reply(&input, &state.kb, level);
                    let pinned = state.codex.ensure_board(&state.kb, level);
                    state.flash_quests(pinned);
                }
                // 🌳 A new tree appeared — reveal it in the atelier.
                if state.codex.trees.len() > trees_before {
                    state.scribe_section = ScribeSection::Atelier;
                    state.tree_view = state.codex.focused_tree.clone();
                    state.tree_selected_node = None;
                }
            }
            Task::none()
        }
        Message::DeskGenerate => {
            let level = state.memory.leveling.current_level;
            let _ = state.codex.desk_reply("set me a trial", &state.kb, level);
            Task::none()
        }
        Message::DeskToggleInterject => {
            let _ = state
                .codex
                .desk_reply("interjections", &state.kb, state.memory.leveling.current_level);
            Task::none()
        }
        Message::TreeSelect(id) => {
            // Clicking a minimized tab chip restores it (no data loss).
            if let Some(tree) = state.codex.trees.iter_mut().find(|t| t.id == id) {
                tree.active = true;
                tree.archived = false;
            }
            state.tree_view = Some(id);
            state.tree_selected_node = None;
            Task::none()
        }
        Message::TreeNodeClicked(node_id) => {
            state.tree_selected_node = Some(node_id);
            Task::none()
        }
        Message::TreeNodeMarkDone(tree_id, node_id) => {
            match state.codex.tree_complete_node(&tree_id, &node_id) {
                Ok((xp, lines)) => {
                    let (leveled, _) = state.memory.leveling.add_experience(xp);
                    if leveled {
                        state.show_level_up = true;
                        state.level_up_details = codex::proclaim_level_up(
                            state.memory.leveling.current_level,
                        );
                    }
                    state.show_xp_gain = true;
                    state.xp_gain_details = format!("🌳 +{} pages — node sealed", xp);
                    state.flash_quests(lines);
                }
                Err(gentle) => {
                    // Locked nodes receive an in-character explanation, never a hard error.
                    state.flash_quests(vec![gentle]);
                }
            }
            Task::none()
        }
        Message::TreeTabMove(id, delta) => {
            state.codex.tree_tab_move(&id, delta);
            Task::none()
        }
        Message::CreateTreeButton => {
            state.page = Page::Scriptorium;
            // The commission conversation lives on the desk — go where the
            // question and the quill are.
            state.scribe_section = ScribeSection::Desk;
            let suggested = if state.current_game == GameType::Generic {
                None
            } else {
                Some(state.current_game.get_name().to_string())
            };
            let _ = state.codex.begin_commission(suggested);
            Task::none()
        }
        Message::ScribeSection(section) => {
            state.scribe_section = section;
            Task::none()
        }
        Message::TreeTabClose(id) => {
            // Closing a tab never destroys the tree — it minimizes (state kept)
            // and the focus moves to the next open tab.
            state.codex.tree_tab_close(&id);
            state.tree_view = state.codex.focused_tree.clone();
            state.tree_selected_node = None;
            Task::none()
        }
        Message::Suggest(text) => {
            state.input_text = text;
            Task::none()
        }
        Message::ShowKnowledge => {
            state.page = Page::Knowledge;
            let lines = state.memory.quest_progress("kb", 1);
            state.flash_quests(lines);
            Task::none()
        }
        Message::KbSearchChanged(query) => {
            state.kb_search = query;
            Task::none()
        }
        Message::ToggleGamePicker => {
            state.page = Page::Games;
            Task::none()
        }
        Message::SelectCategory(filter) => {
            state.game_filter = filter;
            Task::none()
        }
        Message::ShowCheatSheet => {
            state.page = Page::CheatSheet;
            Task::none()
        }
        Message::ToggleSettings => {
            state.page = Page::Settings;
            Task::none()
        }
        Message::OpenSettings => {
            state.page = Page::Settings;
            Task::none()
        }
        Message::ToggleProfile => {
            state.page = Page::Profile;
            Task::none()
        }
        Message::ShowDecorations => {
            state.page = Page::Shop;
            Task::none()
        }
        Message::EquipDecoration(id) => {
            state.memory.leveling.equip_decoration(&id);
            state.memory.save();
            Task::none()
        }
        Message::ProfileNicknameChanged(s) => {
            state.profile_nickname_input = s;
            Task::none()
        }
        Message::SaveNickname => {
            state.memory.set_nickname(&state.profile_nickname_input);
            state.memory.save();
            Task::none()
        }
        Message::ProfileBioChanged(s) => {
            state.profile_bio_input = s;
            Task::none()
        }
        Message::SaveBio => {
            state.memory.set_bio(&state.profile_bio_input);
            state.memory.save();
            Task::none()
        }
        Message::PickProfilePicture => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                    .pick_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Message::ProfilePicturePicked,
        ),
        Message::ProfilePicturePicked(picked) => {
            if let Some(path) = picked {
                match state.memory.set_profile_picture_from_path(&path) {
                    Ok(()) => state.memory.save(),
                    Err(e) => state.add_message(
                        "system".to_string(),
                        format!("🖼️ Couldn't load that picture: {}", e),
                    ),
                }
            }
            Task::none()
        }
        Message::ClearProfilePicture => {
            state.memory.clear_profile_picture();
            state.memory.save();
            Task::none()
        }
        Message::EquipTitle(id) => {
            state.memory.equip_title(&id);
            state.memory.save();
            Task::none()
        }
        Message::SetVibe(id) => {
            let intro = state.memory.set_vibe(&id);
            state.memory.save();
            state.add_message("system".to_string(), intro);
            Task::none()
        }
        Message::SetProvider(provider) => {
            state.llm_settings.provider = provider;
            // Swapping providers swaps in that provider's default model.
            state.llm_settings.model = provider.default_model().to_string();
            state.llm_settings.save();
            Task::none()
        }
        Message::SetApiKey(key) => {
            state.llm_settings.api_key = key;
            state.llm_settings.save();
            Task::none()
        }
        Message::SetModel(model) => {
            state.llm_settings.model = model;
            state.llm_settings.save();
            Task::none()
        }
        Message::SetCustomUrl(url) => {
            state.llm_settings.custom_url = url;
            state.llm_settings.save();
            Task::none()
        }
        Message::ShowApiHelp => Task::none(), // key hints are shown inline
        Message::TestAiApi => {
            let settings = state.llm_settings.clone();
            state.add_message(
                "system".to_string(),
                format!("🛰️ Pinging {} — sending a real test message…", settings.provider.label()),
            );
            Task::perform(llm::ping(settings), Message::AiTestResult)
        }
        Message::AiTestResult(result) => {
            let text = match result {
                Ok(reply) => format!("🟢 {}", reply),
                Err(e) => format!("🔴 Connection test failed: {}", e),
            };
            state.add_message("system".to_string(), text);
            Task::none()
        }
        Message::ToggleAiApi => {
            state.use_ai_api = !state.use_ai_api;
            state.add_message(
                "system".to_string(),
                if state.use_ai_api {
                    "🤖 Online AI mode — replies now come from your configured API provider.".to_string()
                } else {
                    "💻 Local AI mode — instant offline answers.".to_string()
                },
            );
            Task::none()
        }
        Message::NextTheme => {
            state.theme_idx = (state.theme_idx + 1) % theme::themes().len();
            theme::save_index(state.theme_idx);
            Task::none()
        }
        Message::ClearHistory => {
            state.clear_message_history();
            state.add_message("system".to_string(), "🗑️ Chat history cleared.".to_string());
            Task::none()
        }
        Message::ForgetMemory => {
            // Wipe personal facts but keep hard-earned level progress & rewards.
            let leveling = state.memory.leveling.clone();
            let streak = state.memory.learning_streak;
            state.memory.forget_everything();
            state.memory.leveling = leveling;
            state.memory.learning_streak = streak;
            state.memory.ensure_defaults();
            state.memory.save();
            state.stats = UserStats::new();
            state.add_message(
                "system".to_string(),
                "🧹 Forgot all your personal facts — your level and rewards are safe!".to_string(),
            );
            Task::none()
        }
        Message::UpdateStats => {
            state.stats.update_from_memory(&state.memory);
            Task::none()
        }
        Message::ShowLevelUp => {
            state.show_level_up = false;
            Task::none()
        }
        Message::DismissXpGain => {
            state.show_xp_gain = false;
            Task::none()
        }
        Message::ShowAchievement => {
            state.show_achievement = false;
            Task::none()
        }
        Message::GameSearchChanged(value) => {
            state.game_search = value;
            Task::none()
        }
        Message::SelectPlatform(platform) => {
            state.platform_filter = platform;
            Task::none()
        }
        Message::SelectEra(era) => {
            state.era_filter = era;
            Task::none()
        }
        Message::ClearDetectionBanner => {
            state.detection_banner = None;
            Task::none()
        }
        Message::AttachPicture => {
            Task::perform(
                async {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"]);
                    match dialog.pick_file().await {
                        Some(file) => {
                            let name = file.file_name();
                            let bytes = file.read().await;
                            Some((name, bytes))
                        }
                        None => None,
                    }
                },
                |picked| match picked {
                    Some((name, bytes)) => Message::PictureLoaded(name, bytes),
                    None => Message::ClearAttachment,
                },
            )
        }
        Message::PictureLoaded(name, bytes) => {
            if bytes.len() > 15_000_000 {
                state.add_message(
                    "system".to_string(),
                    "📷 That image is too big (over 15 MB) — try a smaller screenshot.".to_string(),
                );
                return Task::none();
            }
            let mime = llm::sniff_image_mime(&bytes);
            let data_url = format!("data:{};base64,{}", mime, llm::base64_encode(&bytes));
            state.attached_image = Some(AttachedImage {
                file_name: name.clone(),
                data_url,
            });
            state.add_message(
                "system".to_string(),
                format!("📎 Attached {} — tell me what you want me to look at, then send!", name),
            );
            Task::none()
        }
        Message::ClearAttachment => {
            state.attached_image = None;
            Task::none()
        }
        Message::ToggleCompact => {
            state.compact = !state.compact;
            let size = if state.compact {
                iced::Size::new(480.0, 190.0)
            } else {
                iced::Size::new(1020.0, 780.0)
            };
            iced::window::oldest().then(move |id| match id {
                Some(id) => iced::window::resize(id, size),
                None => Task::none(),
            })
        }
        Message::BuyDecoration(id) => {
            let result = state.memory.leveling.buy_decoration(&id);
            state.show_achievement = true;
            state.achievement_details = result.unwrap_or_else(|err| err);
            state.memory.save();
            Task::none()
        }
        Message::BuyMysteryBox => {
            let result = state.memory.leveling.buy_mystery_box();
            state.show_achievement = true;
            state.achievement_details = result.unwrap_or_else(|err| err);
            state.flash_progress_rewards();
            state.memory.save();
            Task::none()
        }
        Message::ApiResponse(response) => {
            state.is_typing = false;
            state.add_message("bot".to_string(), response.clone());
            state.memory.ensure_daily_quests();
            let (leveled, _) = state.memory.add_conversation(
                "📷 online chat".to_string(),
                response.clone(),
                state.last_answered_topic.clone(),
            );
            if leveled {
                state.show_level_up = true;
                state.level_up_details = format!(
                    "🎉 LEVEL UP! You reached level {}!",
                    state.memory.leveling.current_level
                );
            }
            state.memory.check_milestone_badges();
            let ask_lines = state.memory.quest_progress("ask", 1);
            state.flash_quests(ask_lines);
            state.flash_progress_rewards();
            state.memory.save();
            Task::none()
        }
        Message::CopyMessage(content) => iced::clipboard::write::<Message>(content),
        _ => Task::none(),
    }
}

fn view(state: &App) -> Element<'_, Message> {
    let th = theme::palette(state.theme_idx);
    let danger = iced::Color::from_rgb8(0xff, 0x6b, 0x6b);
    let online_green = iced::Color::from_rgb8(0x34, 0xd3, 0x99);

    // ---- Compact overlay mode: a mini chat bar -----------------------------------
    if state.compact {
        return compact_view(state, th, danger, online_green);
    }

    // ---- App frame: sidebar + main area -------------------------------------------
    let sidebar = sidebar_view(state, th, danger, online_green);

    let mut main_col: iced::widget::Column<'_, Message> =
        column![].spacing(8).width(Length::Fill).height(Length::Fill);
    main_col = main_col.push(top_bar(state, th, danger, online_green));
    main_col = main_col.push(toasts_view(state, th, danger, online_green));
    let content: Element<Message> = match state.page {
        Page::Chat => chat_page(state, th, danger, online_green),
        Page::Profile => profile_page(state, th),
        Page::Games => games_page(state, th),
        Page::Shop => shop_page(state, th),
        Page::Knowledge => knowledge_page(state, th),
        Page::CheatSheet => cheatsheet_page(state, th),
        Page::Scriptorium => scriptorium_page(state, th),
        Page::Info => info_page(state, th),
        Page::Settings => settings_page(state, th, danger),
    };
    main_col = main_col.push(content);
    if state.page == Page::Chat {
        main_col = main_col.push(chat_input_row(state, th));
    }

    let base: Element<Message> = row![sidebar, main_col]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    base
}

/// Left rail: identity card, navigation, live status.
fn sidebar_view<'a>(
    state: &'a App,
    th: theme::UiTheme,
    danger: Color,
    online_green: Color,
) -> Element<'a, Message> {
    let leveling = &state.memory.leveling;
    let lvl = leveling.current_level;
    let xp_now = leveling.experience_points;
    let max_level = memory::LEVEL_XP_REQUIREMENTS.len() as u32;
    let cost_next = memory::LEVEL_XP_REQUIREMENTS.get(lvl as usize).copied();
    let pct = match cost_next {
        Some(cost) if cost > 0 && lvl < max_level => {
            ((xp_now as f32) / (cost as f32) * 100.0).min(100.0)
        }
        _ => 100.0,
    };

    // Identity card — level, title, XP, coins.
    let order_rank = codex::OrderRank::from_level(lvl);
    let level_card: Element<Message> = container(
        column![
            text(format!("⭐ Level {}", lvl)).size(16).color(th.text),
            text(memory::level_title(lvl)).size(12).color(th.highlight),
            text(format!(
                "{} {} of the Order",
                order_rank.sigil(),
                order_rank.name()
            ))
            .size(11)
            .color(th.highlight),
            progress_bar(0.0..=100.0, pct).girth(6.0),
            text(if lvl >= max_level {
                "💫 Max level reached".to_string()
            } else {
                format!(
                    "✒️ {} pages to the next volume",
                    cost_next.unwrap_or(0).saturating_sub(xp_now)
                )
            })
            .size(10)
            .color(th.text_dim),
            text(format!("💰 {} coins", leveling.coins))
                .size(11)
                .color(th.highlight),
        ]
        .spacing(4),
    )
    .style(theme::card(th))
    .width(Length::Fill)
    .padding(10)
    .into();

    // Navigation.
    let nav = |label: &'static str, page: Page| -> Element<'a, Message> {
        let active = state.page == page;
        let (bg, edge, tc, hov) = if active {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.bg, th.bg, th.text, th.surface_alt)
        };
        button(text(label).size(13).color(tc))
            .style(theme::chip(bg, edge, tc, hov))
            .on_press(Message::Nav(page))
            .padding([8, 10])
            .width(Length::Fill)
            .into()
    };

    // Live status footer.
    let (dot_color, dot_label) = if state.current_game != GameType::Generic {
        (online_green, format!("🟢 {}", state.current_game.get_name()))
    } else {
        (danger, "🔴 No game detected".to_string())
    };
    let ai_label = if state.use_ai_api {
        if state.llm_settings.is_ready() {
            "🤖 Online AI"
        } else {
            "🔑 API not configured"
        }
    } else {
        "💻 Local AI"
    };

    let scriptorium_nav_item: Element<'a, Message> = nav("📜 Scriptorium", Page::Scriptorium);

    container(
        column![
            text("🚀 ROCKET").size(18).color(th.text),
            level_card,
            nav("💬 Chat", Page::Chat),
            nav("🎮 Games", Page::Games),
            nav("👤 Profile", Page::Profile),
            nav("🛒 XP Shop", Page::Shop),
            nav("📚 Knowledge", Page::Knowledge),
            nav("📋 Cheat Sheet", Page::CheatSheet),
            scriptorium_nav_item,
            nav("📖 Info", Page::Info),
            nav("⚙️ Settings", Page::Settings),
            Space::new().height(Length::Fill),
            text(dot_label).size(10).color(dot_color),
            text(ai_label).size(10).color(th.text_dim),
            row![
                button(text("🗔").size(12).color(th.text))
                    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                    .on_press(Message::ToggleCompact)
                    .padding([4, 8]),
                button(text("🎨").size(12).color(th.text))
                    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                    .on_press(Message::NextTheme)
                    .padding([4, 8]),
                button(text("🗑️").size(12).color(danger))
                    .style(theme::chip(th.surface, th.border, danger, th.surface_alt))
                    .on_press(Message::ClearHistory)
                    .padding([4, 8]),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ]
        .spacing(8),
    )
    .style(theme::flat(th.surface, th.border, 12.0))
    .padding(12)
    .width(Length::Fixed(196.0))
    .height(Length::Fill)
    .into()
}

/// Top strip: page title + game-context chip.
fn top_bar<'a>(
    state: &'a App,
    th: theme::UiTheme,
    _danger: Color,
    _online_green: Color,
) -> Element<'a, Message> {
    let title = match state.page {
        Page::Chat => "💬 Chat",
        Page::Profile => "👤 Profile",
        Page::Games => "🎮 Game Library",
        Page::Shop => "🛒 XP Shop",
        Page::Knowledge => "📚 Knowledge Base",
        Page::CheatSheet => "📋 Cheat Sheet",
        Page::Scriptorium => "📜 Scriptorium",
        Page::Info => "📖 Info",
        Page::Settings => "⚙️ Settings",
    };
    let typing_note = if state.is_typing { " · thinking…" } else { "" };

    let game_chip: Element<Message> = button(
        text(if state.current_game == GameType::Generic {
            "💬 General Chat ▸".to_string()
        } else {
            format!("🎯 {} ▸", state.current_game.get_name())
        })
        .size(12)
        .color(th.text),
    )
    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
    .on_press(Message::Nav(Page::Games))
    .padding([6, 10])
    .into();

    row![
        text(title).size(15).color(th.text),
        text(typing_note).size(11).color(th.text_dim),
        Space::new().width(Length::Fill),
        game_chip,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Dismissible reward toasts — level ups, XP gains, unlocks, quest completions.
fn toasts_view<'a>(
    state: &'a App,
    th: theme::UiTheme,
    danger: Color,
    _online_green: Color,
) -> Element<'a, Message> {
    if !(state.show_level_up || state.show_xp_gain || state.show_achievement) {
        return Space::new().width(Length::Fill).into();
    }
    let mut col: iced::widget::Column<'_, Message> = column![].spacing(4);
    if state.show_level_up {
        col = col.push(
            container(
                row![
                    text(state.level_up_details.clone())
                        .size(12)
                        .color(Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(11).color(Color::WHITE))
                        .style(theme::chip(
                            th.accent_hover,
                            th.accent_hover,
                            Color::WHITE,
                            th.accent
                        ))
                        .on_press(Message::ShowLevelUp)
                        .padding([2, 6]),
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center),
            )
            .style(theme::flat(th.accent, th.accent_hover, 8.0))
            .padding(8)
            .width(Length::Fill),
        );
    }
    if state.show_xp_gain {
        col = col.push(
            container(
                row![
                    text(state.xp_gain_details.clone()).size(12).color(th.text),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(11).color(th.text_dim))
                        .style(theme::chip(
                            th.surface_alt,
                            th.border,
                            th.text_dim,
                            th.surface
                        ))
                        .on_press(Message::DismissXpGain)
                        .padding([2, 6]),
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center),
            )
            .style(theme::flat(th.surface_alt, th.border, 8.0))
            .padding(8)
            .width(Length::Fill),
        );
    }
    if state.show_achievement {
        col = col.push(
            container(
                row![
                    text(state.achievement_details.clone())
                        .size(12)
                        .color(th.text),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(11).color(danger))
                        .style(theme::chip(
                            th.surface_alt,
                            th.border,
                            danger,
                            th.surface
                        ))
                        .on_press(Message::ShowAchievement)
                        .padding([2, 6]),
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center),
            )
            .style(theme::flat(th.highlight, th.border, 8.0))
            .padding(8)
            .width(Length::Fill),
        );
    }
    col.into()
}

/// Chat page — welcome hero with suggestion chips, or message bubbles.
/// 🌳 The CREATE TREE atelier — tabbed, multi-instance quest-tree workspace.
/// Authoring flows through Rocket's desk (side-channel); this panel surfaces
/// every living tree and the selected tree's nodes with live progress.
fn trees_atelier(state: &App, th: theme::UiTheme) -> Element<'static, Message> {
    // --- prominent Create Tree control (minimal-friction entry) -------------
    let game_ctx = if state.current_game == GameType::Generic {
        String::new()
    } else {
        format!(" ({})", state.current_game.get_name())
    };
    let create_btn = container(
        button(
            text(format!("🌳 Create Tree{}", game_ctx))
                .size(12)
                .color(th.text),
        )
        .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
        .on_press(Message::CreateTreeButton)
        .padding([5, 10]),
    )
    .padding(0);

    // --- empty state: the button plus examples -------------------------------
    if state.codex.trees.is_empty() {
        return container(
            column![
                create_btn,
                text("…or commission through Rocket's desk:")
                    .size(12)
                    .color(th.text_dim),
                text("`create tree for minecraft` · `create tree branching for terraria depth 4` · `create tree comparative for minecraft and terraria`")
                    .size(11)
                    .color(th.highlight),
            ]
            .spacing(6),
        )
        .style(theme::card(th))
        .padding(10)
        .into();
    }

    // --- browser-style tab strip (✕ closes the tab, tree kept safe) ---------
    // Only OPEN trees appear as tabs; minimized ones rest via `open tree`.
    let mut tabs: iced::widget::Row<'_, Message> = row![].spacing(2);
    let open_trees: Vec<&codex_tree::QuestTree> = state
        .codex
        .trees
        .iter()
        .filter(|t| t.active && !t.archived)
        .collect();
    let selected = state
        .tree_view
        .clone()
        .or_else(|| state.codex.focused_tree.clone())
        .filter(|id| open_trees.iter().any(|t| &t.id == id))
        .unwrap_or_else(|| {
            open_trees
                .first()
                .map(|t| t.id.clone())
                .unwrap_or_default()
        });
    for t in &open_trees {
        let is_sel = t.id == selected;
        let dot = if is_sel {
            "🔵"
        } else if t.active {
            "🟢"
        } else {
            "⚪"
        };
        let done = t.completed_count();
        let label = format!("{} {} ({}/{})", dot, t.name, done, t.nodes.len());
        let (id_sel, id_close) = (t.id.clone(), t.id.clone());
        let (surface, border, txt) = if is_sel {
            (th.surface, th.highlight, th.text)
        } else {
            (th.bg, th.border, th.text_dim)
        };
        tabs = tabs.push(
            container(
                row![
                    button(text(label).size(11).color(txt))
                        .style(theme::chip(surface, surface, txt, th.surface_alt))
                        .on_press(Message::TreeSelect(id_sel))
                        .padding([4, 4]),
                    button(text("✕").size(9).color(th.text_dim))
                        .style(theme::chip(surface, surface, th.text_dim, th.surface_alt))
                        .on_press(Message::TreeTabClose(id_close))
                        .padding([4, 5]),
                ]
                .spacing(0),
            )
            .style(move |_| container::Style {
                background: Some(surface.into()),
                border: iced::Border {
                    color: border,
                    width: if is_sel { 1.5 } else { 1.0 },
                    radius: iced::border::Radius {
                        top_left: 8.0,
                        top_right: 8.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                },
                ..Default::default()
            })
            .padding([4, 2]),
        );
    }
    // ＋ new-tab: opens the commission conversation.
    tabs = tabs.push(
        button(text("＋").size(14).color(th.highlight))
            .style(move |_, _| iced::widget::button::Style {
                background: Some(th.surface.into()),
                border: iced::Border {
                    color: th.border,
                    width: 1.0,
                    radius: iced::border::Radius {
                        top_left: 8.0,
                        top_right: 8.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                },
                ..Default::default()
            })
        .on_press(Message::CreateTreeButton)
        .padding([4, 10]),
    );
    let archived_n = state.codex.trees.iter().filter(|t| t.archived).count();
    if archived_n > 0 {
        tabs = tabs.push(
            text(format!("🗄️ {} archived (restore via desk)", archived_n))
                .size(10)
                .color(th.text_dim),
        );
    }
    // Reorder chips for the focused tab.
    tabs = tabs.push(
        row![
            button(text("◀").size(9).color(th.text))
                .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                .on_press(Message::TreeTabMove(selected.clone(), -1))
                .padding([3, 6]),
            button(text("▶").size(9).color(th.text))
                .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                .on_press(Message::TreeTabMove(selected.clone(), 1))
                .padding([3, 6]),
        ]
        .spacing(2),
    );

    // --- the interactive graph + synchronized detail panel -------------------
    let mut panel: iced::widget::Column<'_, Message> = column![].spacing(6);
    let focused = open_trees.iter().find(|t| t.id == selected).map(|t| t.id.clone());
    if let Some(tree) = state.codex.trees.iter().find(|t| Some(&t.id) == focused.as_ref()) {
        panel = panel.push(
            text(format!(
                "{} — {}{}{} · {} pages earned{}",
                tree.name,
                tree.archetype.label(),
                tree.game2
                    .as_ref()
                    .map(|g| format!(" · {} ⚔️ {}", tree.game, g))
                    .unwrap_or_else(|| format!(" · {}", tree.game)),
                tree.subtitle
                    .as_ref()
                    .map(|s| format!(" — “{}”", s))
                    .unwrap_or_default(),
                tree.xp_earned,
                if tree.completed_sealed { " · 🏆 SEALED" } else { "" }
            ))
            .size(12)
            .color(th.text),
        );

        // The living graph (click nodes to inspect them).
        let graph = tree_graph::TreeGraph::new(
            tree,
            state.tree_selected_node.as_deref(),
            tree_graph::GraphColors {
                surface: th.surface,
                border: th.border,
                highlight: th.highlight,
                text: th.text,
                text_dim: th.text_dim,
                danger: iced::Color::from_rgb8(0xe6, 0x39, 0x46),
                dark: Color::from_rgb8(0x10, 0x1a, 0x33),
            },
            Box::new(|node_id| Message::TreeNodeClicked(node_id)),
        );
        panel = panel.push(
            container(
                canvas(graph)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
                .width(Length::Fill)
                .height(Length::Fixed(300.0))
                .style(theme::card(th)),
        );

        // Synchronized detail panel for the selected node.
        let selected_node = state
            .tree_selected_node
            .as_deref()
            .and_then(|id| tree.node(id));
        let frontier = tree.frontier();
        let detail: Element<'static, Message> = match selected_node {
            None => text(
                "Select a node on the graph — ❓ available, ✓ sealed, · locked — to read its full decree.",
            )
            .size(11)
            .color(th.text_dim)
            .into(),
            Some(node) => {
            let is_frontier = frontier.contains(&node.id);
            let state_word = if node.completed {
                "✅ sealed"
            } else if node.stale {
                "⚠️ stale"
            } else if is_frontier {
                "⚔️ available"
            } else {
                "🔒 locked"
            };
            container(
                    column![
                        text(format!(
                            "{} · [T{}] {}{} · +{} pages",
                            state_word,
                            node.tier,
                            node.title,
                            if node.optional { " · optional" } else { "" },
                            node.xp_reward
                        ))
                        .size(13)
                        .color(th.text),
                        text(node.description.clone())
                            .size(11)
                            .color(th.text_dim),
                        text(format!(
                            "Success: {} · {}",
                            node.progress_line(),
                            node.failure_guidance
                        ))
                        .size(10)
                        .color(th.text_dim),
                        text(format!("📚 {}", node.grimoire_sections.join(", ")))
                            .size(10)
                            .color(th.highlight),
                        {
                            let actions: Element<'static, Message> = if node.completed {
                                row![].into()
                            } else if is_frontier && !node.stale {
                                row![
                                    button(
                                        text("✔ Mark Done")
                                            .size(11)
                                            .color(Color::from_rgb8(0x10, 0x1a, 0x33))
                                    )
                                    .style(theme::solid(th.highlight, th.surface_alt))
                                    .on_press(Message::TreeNodeMarkDone(
                                        tree.id.clone(),
                                        node.id.clone(),
                                    ))
                                    .padding([4, 10]),
                                    text("Sealing lights the path forward.")
                                        .size(10)
                                        .color(th.text_dim),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .into()
                            } else {
                                text("🔒 Not yet within reach — complete its prerequisites first (see the lit paths).")
                                    .size(10)
                                    .color(th.text_dim)
                                    .into()
                            };
                            let actions_col: Element<'static, Message> = column![actions].spacing(6).into();
                            actions_col
                        }
                    ]
                    .spacing(4),
                )
                .style(theme::card(th))
                .padding(8)
                .into()
            }
        };
        panel = panel.push(detail);
        panel = panel.push(
            text("Outline: `outline tree <name>` · Refine: `expand tree …` · `rebalance tree …` · `note tree …` · `validate trees` · `repair tree …` · Auto-saved on every change (`confirm saved`)")
                .size(10)
                .color(th.text_dim),
        );
    }

    column![
        row![create_btn, Space::new().width(Length::Fill)],
        tabs,
        scrollable(panel).height(Length::Fill),
    ]
    .spacing(6)
    .into()
}

/// 📜 The Scriptorium — the Order's quest board and Rocket's desk
/// (the side-chat agent for practical tasks).
fn scriptorium_page(state: &App, th: theme::UiTheme) -> Element<'static, Message> {
    let codex = &state.codex;
    let lvl = state.memory.leveling.current_level;
    let rank = codex::OrderRank::from_level(lvl);
    let desk_input_ui: Element<'static, Message> = row![
        text_input(
            "Ask Rocket — e.g. `quest report`, `audit minecraft`, `where stand I`…",
            &state.desk_input,
        )
        .on_input(Message::DeskInputChanged)
        .on_submit(Message::DeskSend)
        .style(theme::input(th))
        .size(13),
        button(text("✒️").size(14).color(th.text))
            .style(theme::solid(th.highlight, th.surface_alt))
            .on_press(Message::DeskSend)
            .padding([6, 10]),
    ]
    .spacing(8)
    .width(Length::Fill)
    .into();
    let atelier_ui: Element<'static, Message> = trees_atelier(state, th);

    // --- Quest Board ---
    let mut board_col: iced::widget::Column<'_, Message> = column![].spacing(8);
    let open: Vec<&codex::CodexQuest> = codex.board.iter().filter(|q| !q.sealed).collect();
    let sealed_count = codex.board.iter().filter(|q| q.sealed).count();
    if open.is_empty() {
        board_col = board_col.push(
            text("📜 The board stands empty of open quests — set yourself a trial below.")
                .size(13)
                .color(th.text_dim),
        );
    }
    for q in &open {
        let pct = if q.target > 0 {
            (q.progress.min(q.target) as f32 / q.target as f32) * 100.0
        } else {
            0.0
        };
        board_col = board_col.push(container(
            column![
                text(format!("{} — {}", q.kind.label(), q.title))
                    .size(13)
                    .color(th.text),
                progress_bar(0.0..=100.0, pct).girth(4.0),
                text(q.progress_line()).size(11).color(th.text_dim),
                text(format!(
                    "+{} pages · +{} lore · exercises: {}",
                    q.xp_reward,
                    q.lore_reward,
                    q.grimoire_sections.join(", ")
                ))
                .size(10)
                .color(th.text_dim),
            ]
            .spacing(4),
        )
        .style(theme::card(th))
        .padding(10));
    }

    // --- Rocket's desk transcript (most recent last) ---
    let mut desk_col: iced::widget::Column<'_, Message> = column![].spacing(6);
    for m in codex.desk_history.iter().rev().take(30).rev() {
        let who = if m.role == "user" { "🧑‍🎓" } else { "🕯️" };
        desk_col = desk_col.push(
            text(format!("{} {} · {}", who, m.role, m.timestamp))
                .size(9)
                .color(th.text_dim),
        );
        desk_col = desk_col.push(
            text(m.content.clone())
                .size(12)
                .color(if m.role == "user" { th.text } else { th.highlight }),
        );
    }
    if codex.desk_history.is_empty() {
        desk_col = desk_col.push(
            text("The desk is quiet. Rocket waits with quill ready — try `help`.")
                .size(12)
                .color(th.text_dim),
        );
    }

    // --- sub-category tab bar (the Scriptorium's own browser tabs) ----------
    let section_bar: iced::widget::Row<'_, Message> = {
        let mut bar = row![].spacing(4);
        for section in [ScribeSection::Atelier, ScribeSection::Desk, ScribeSection::Board] {
            let active = state.scribe_section == section;
            let label = section.label();
            bar = bar.push(
                button(text(label).size(12).color(if active { Color::from_rgb8(0x10, 0x1a, 0x33) } else { th.text }))
                    .style(move |_, _| iced::widget::button::Style {
                        background: Some(if active { th.highlight } else { th.surface }.into()),
                        border: iced::Border {
                            color: if active { th.highlight } else { th.border },
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    })
                    .on_press(Message::ScribeSection(section))
                    .padding([5, 12]),
            );
        }
        bar
    };

    // --- section content (one sub-category at a time, full height) ----------
    let content: Element<'static, Message> = match state.scribe_section {
        ScribeSection::Atelier => atelier_ui,
        ScribeSection::Desk => column![
            text("🕯️ Rocket's Desk — your familiar for practical tasks")
                .size(15)
                .color(th.highlight),
            scrollable(desk_col).height(Length::Fill),
            desk_input_ui,
            row![
                button(text("🎲 Set me a trial").size(11).color(th.text))
                    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                    .on_press(Message::DeskGenerate)
                    .padding([4, 8]),
                button(
                    text(if codex.desk_interjections {
                        "🔔 Marginalia ON"
                    } else {
                        "🔕 Marginalia OFF"
                    })
                    .size(11)
                    .color(th.text)
                )
                .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                .on_press(Message::DeskToggleInterject)
                .padding([4, 8]),
                text(format!(
                    "Chronicle: {} deeds · audit log: {} entries · {} quests sealed",
                    codex.chronicle.len(),
                    codex.actions.len(),
                    sealed_count
                ))
                .size(10)
                .color(th.text_dim),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .into(),
        ScribeSection::Board => column![
            text("📋 Quest Board — pinned by the Order from the grimoires")
                .size(15)
                .color(th.highlight),
            container(scrollable(board_col).height(Length::Fill))
                .style(theme::card(th))
                .padding(4),
        ]
        .spacing(8)
        .into(),
    };

    column![
        text(format!(
            "📜 The Scriptorium — {} {} of the Eternal Codex",
            rank.sigil(),
            rank.name()
        ))
        .size(20)
        .color(th.text),
        text("Every question is an act of scholarship; every answer, a page transcribed toward the next volume.")
            .size(12)
            .color(th.text_dim),
        section_bar,
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::card(th))
            .padding(10),
    ]
    .spacing(8)
    .padding(10)
    .into()
}

/// 📖 The Info page — the complete guidance sheet for the Codex.
fn info_page(state: &App, th: theme::UiTheme) -> Element<'static, Message> {
    let head = |s: &str| text(s.to_string()).size(15).color(th.highlight);
    let body = |s: &str| text(s.to_string()).size(12).color(th.text);
    let dim = |s: &str| text(s.to_string()).size(11).color(th.text_dim);
    let cmd = |s: &str| text(s.to_string()).size(11).color(th.highlight);
    let sheet = column![
        text("📖 The Codex Guide — how everything works")
            .size(20)
            .color(th.text),
        dim("The scholar's sheet. Read it in any order; return whenever the shelves blur."),
        head("🕯️ The Two Channels"),
        body("The MAIN HALL (the Chat page) is for scholarly inquiry into the grimoires: ask about any game and every grounded answer earns pages. THIS DESK — the 📜 Scriptorium's side channel — is where you address Rocket alone for practical tasks. Desk talk never earns XP; main-hall questions always might."),
        head("📈 Asking, Earning, Rising"),
        body("Grounded answers mint PAGES (and coins — coins equal all XP ever earned, spending never lowers your level). Conversation +10 XP, teaching a fact +25, corrections via `wrong: <answer>` +15, follow-ups (`more`) +15, new topic mastery +20, switching games +5, streak bonuses 5–75, daily quests 25–60 each, missions +75. Levels run 1→50 across ten Order ranks: Novice Scribe → Ink Apprentice → Journeyman Scribe → Adept Scholar → Loremaster → Master Archivist → Grand Archivist. Your identity card (sidebar) and 👤 Profile show the tally; ask Rocket `where stand I` anytime."),
        head("📋 The Quest Board"),
        body("Every day the Order pins 3 quests (ask, chat, teach, explore, item-hunting, opening the Knowledge base) worth 25–60 XP — they complete themselves the moment their target is hit. Co-op MISSIONS: say the word (`help me beat the ender dragon`) and Rocket tracks up to 5 on the board; completing one pays 75 XP. Everything below feeds the same character."),
        head("🌳 CREATE TREE — the atelier"),
        body("Press 🌳 **Create Tree** and answer two questions (the realm, then the shape) — or tell Rocket `create tree for <game>`. Archetypes: Linear Mastery · Branching Web · Ordeal Ladder · Comparative Network · Restoration · Timed Trials · Narrative Chronicle · Edge-Case Gauntlet — with depth, breadth and gentle/steep economies. Each node on the graph is a real piece of the game's grimoire: ❓ available, ✓ sealed, · locked behind its prerequisites."),
        body("Click a node to read its full decree; **✔ Mark Done** seals it and lights the path forward. Trees live in their own browser-style tabs — ✕ closes the tab (tree kept safe), ＋ commissions another, ◀ ▶ reorder. Refine from the desk: `expand tree <name> depth N` · `rebalance tree <name> [to N]` · `note tree …` · `validate trees` · `repair tree …`. Every change saves itself."),
        head("🕯️ Rocket's Desk — commands"),
        cmd("`quest report` — the board · `set me a trial` — pin a fresh quest"),
        cmd("`audit <game>` · `gaps <game>` — grimoire completeness"),
        cmd("`compare <a> and <b>` — scholarship across two realms"),
        cmd("`where stand I` — rank, pages, realm tiers · `trees` — the atelier survey"),
        cmd("`outline tree <name>` — pure text outline · `confirm saved` — persistence check"),
        cmd("`chronicle` — export your deeds · `log` — the audit trail"),
        cmd("`remind me …` — re-explain anything, forever"),
        head("💾 Where Everything Lives"),
        body("Every memory is a JSON file beside the app: rocket_memory.json (profile, XP, quests), rocket-brain.json (taught answers), rocket_chat_history.json, codex_character.json (trees, ranks, chronicle), rocket-llm.json (online key). Backing up ROCKET = copying those files. Ask `confirm saved` and Rocket verifies."),
        head("🌐 Online AI (optional)"),
        body("⚙️ Settings → flip to 🌐 online, pick a provider, paste a key. All your taught facts travel as Rocket's instructions. If anything fails, the local brain answers and says why. Full sheet: the 📖 Info you are reading is complete — but `remind me …` on the desk still works word-by-word."),
    ]
    .spacing(10);

    container(scrollable(sheet))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}

fn chat_page<'a>(
    state: &'a App,
    th: theme::UiTheme,
    danger: Color,
    _online_green: Color,
) -> Element<'a, Message> {
    let _ = danger;
    if state.messages.is_empty() {
        let suggestions: [&str; 4] = [
            "💎 How do I get diamonds?",
            "🏆 How do quests and XP work?",
            "🔥 Give me a pro tip",
            "📚 What do you know about me?",
        ];
        let mut chip_rows: iced::widget::Column<'_, Message> = column![].spacing(6);
        for pair in suggestions.chunks(2) {
            let mut r = row![].spacing(6);
            for s in pair {
                r = r.push(
                    button(text(*s).size(12).color(th.text))
                        .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                        .on_press(Message::Suggest(s.to_string()))
                        .padding([6, 10]),
                );
            }
            chip_rows = chip_rows.push(r);
        }
        container(
            column![
                text("🚀").size(42),
                text("Hey, I'm ROCKET!").size(20).color(th.text),
                text("Your gaming AI companion. Ask anything, attach a 📎 screenshot,\nor open 🎮 Games to switch me into expert mode.")
                    .size(12)
                    .color(th.text_dim),
                chip_rows,
            ]
            .spacing(8)
            .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    } else {
        let mut msg_col: iced::widget::Column<'_, Message> =
            column![].spacing(8).padding([4, 4]).width(Length::Fill);
        // Render only the last 50 messages — fewer widgets keeps scrolling
        // smooth even with a huge saved history.
        let visible: Vec<&ChatMessage> = state
            .messages
            .iter()
            .rev()
            .take(50)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if state.messages.len() > visible.len() {
            msg_col = msg_col.push(
                text(format!(
                    "— {} older messages not shown —",
                    state.messages.len() - visible.len()
                ))
                .size(10)
                .color(th.text_dim),
            );
        }
        for msg in visible {
            match msg.role.as_str() {
                "user" => {
                    let bubble: Element<Message> = container(
                        column![
                            text(format!("🧑 You · {}", msg.timestamp))
                                .size(9)
                                .color(Color::WHITE),
                            text(msg.content.clone()).size(13).color(Color::WHITE),
                        ]
                        .spacing(2),
                    )
                    .style(theme::chat_message(th.accent, Color::WHITE))
                    .padding(10)
                    .max_width(520)
                    .into();
                    msg_col = msg_col.push(
                        row![Space::new().width(Length::Fill), bubble]
                            .align_y(Alignment::Center),
                    );
                }
                "bot" => {
                    let bubble: Element<Message> = container(
                        column![
                            text(format!("🚀 ROCKET · {}", msg.timestamp))
                                .size(9)
                                .color(th.highlight),
                            text(msg.content.clone()).size(13).color(th.text),
                        ]
                        .spacing(2),
                    )
                    .style(theme::chat_message(th.surface_alt, th.text))
                    .padding(10)
                    .max_width(560)
                    .into();
                    let copy_btn: Element<Message> = button(text("⧉").size(10).color(th.text_dim))
                        .style(theme::chip(
                            th.surface,
                            th.border,
                            th.text_dim,
                            th.surface_alt,
                        ))
                        .on_press(Message::CopyMessage(msg.content.clone()))
                        .padding([2, 6])
                        .into();
                    msg_col = msg_col.push(
                        row![bubble, copy_btn, Space::new().width(Length::Fill)]
                            .align_y(Alignment::Start),
                    );
                }
                _ => {
                    msg_col = msg_col.push(
                        container(
                            text(format!("— {} —", msg.content))
                                .size(11)
                                .color(th.text_dim),
                        )
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center),
                    );
                }
            }
        }
        scrollable(msg_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// The message input row: attachment chip + 📎 + text field + send.
fn chat_input_row<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    let attach_button: Element<Message> = button(text("📎").size(13).color(th.text))
        .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
        .on_press(Message::AttachPicture)
        .padding(10)
        .into();

    let attachment_chip: Element<Message> = if let Some(att) = &state.attached_image {
        row![
            text(format!("📎 {} ✓", att.file_name))
                .size(11)
                .color(th.highlight),
            button(text("✕").size(10).color(th.text_dim))
                .style(theme::chip(th.surface, th.border, th.text_dim, th.surface_alt))
                .on_press(Message::ClearAttachment)
                .padding([2, 6]),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
    } else {
        Space::new().width(Length::Fill).into()
    };

    // 🕯️ The rite may spotlight the Main Hall input.
    let input_row: Element<Message> = row![
        text_input("Ask ROCKET anything…", &state.input_text)
            .on_input(Message::InputChanged)
            .on_submit(Message::Send)
            .size(13)
            .padding(10)
            .style(theme::input(th))
            .width(Length::Fill),
        attach_button,
        button(text("🚀 Send").size(13).color(Color::WHITE))
            .style(theme::solid(th.accent, th.accent_hover))
            .on_press(Message::Send)
            .padding(10),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into();
    column![attachment_chip, input_row]
        .spacing(4)
        .width(Length::Fill)
        .into()
}

/// Games page — searchable, filterable library of every catalog game.
fn games_page<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    let mut chips: iced::widget::Row<'_, Message> = row![].spacing(6);
    for f in ALL_FILTERS.iter() {
        let (bg, edge, tc, hov) = if state.game_filter == *f {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        chips = chips.push(
            button(text(f.label()).size(11).color(tc))
                .style(theme::chip(bg, edge, tc, hov))
                .on_press(Message::SelectCategory(*f))
                .padding([5, 8]),
        );
    }

    let mut platform_chips: iced::widget::Row<'_, Message> = row![].spacing(6);
    for p in ALL_PLATFORMS.iter() {
        let selected = state.platform_filter == *p;
        let (bg, edge, tc, hov) = if selected {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        platform_chips = platform_chips.push(
            button(text(p.label()).size(11).color(tc))
                .style(theme::chip(bg, edge, tc, hov))
                .on_press(Message::SelectPlatform(*p))
                .padding([5, 8]),
        );
    }

    let mut era_chips: iced::widget::Row<'_, Message> = row![].spacing(6);
    for e in ALL_ERAS.iter() {
        let selected = state.era_filter == *e;
        let (bg, edge, tc, hov) = if selected {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        era_chips = era_chips.push(
            button(text(e.label()).size(11).color(tc))
                .style(theme::chip(bg, edge, tc, hov))
                .on_press(Message::SelectEra(*e))
                .padding([5, 8]),
        );
    }

    // Combined filters: name search AND genre AND device AND era.
    let search_lower = state.game_search.trim().to_lowercase();
    let mut grid: iced::widget::Column<'_, Message> = column![].spacing(6);
    let mut current_row: iced::widget::Row<'_, Message> = row![].spacing(6);
    let mut in_row = 0;
    let mut shown = 0;
    for g in all_games() {
        let name_ok =
            search_lower.is_empty() || g.get_name().to_lowercase().contains(&search_lower);
        if !name_ok
            || !state.game_filter.matches(&g)
            || !state.platform_filter.matches(&g)
            || !state.era_filter.matches(g.year())
        {
            continue;
        }
        let selected = state.current_game == g;
        let (bg, edge, tc, hov) = if selected {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        current_row = current_row.push(
            button(
                text(format!("{} {}", g.get_emoji(), g.get_name()))
                    .size(12)
                    .color(tc),
            )
            .style(theme::chip(bg, edge, tc, hov))
            .on_press(Message::ChangeGame(g.clone()))
            .padding([6, 8])
            .width(Length::Fill),
        );
        in_row += 1;
        shown += 1;
        if in_row == 3 {
            grid = grid.push(current_row);
            current_row = row![].spacing(6);
            in_row = 0;
        }
    }
    if in_row > 0 {
        grid = grid.push(current_row);
    }
    if shown == 0 {
        grid = grid.push(
            text("No games match those filters — try clearing the search.")
                .size(12)
                .color(th.text_dim),
        );
    }

    scrollable(
        column![
            text_input("Search games by name…", &state.game_search)
                .on_input(Message::GameSearchChanged)
                .on_submit(Message::Nav(Page::Chat))
                .size(13)
                .padding(8)
                .style(theme::input(th)),
            row![text("Genre:").size(11).color(th.text_dim), chips]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            row![text("Device:").size(11).color(th.text_dim), platform_chips]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            row![text("Era:").size(11).color(th.text_dim), era_chips]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            text(format!("{} games match — click one to switch expert mode on.", shown))
                .size(11)
                .color(th.text_dim),
            grid,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Profile page — identity, quests, badges, achievements.
fn profile_page<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    let leveling = &state.memory.leveling;
    let lvl = leveling.current_level;
    let xp_now = leveling.experience_points;
    let max_level = memory::LEVEL_XP_REQUIREMENTS.len() as u32;
    let cost_next = memory::LEVEL_XP_REQUIREMENTS.get(lvl as usize).copied();
    let pct = match cost_next {
        Some(cost) if cost > 0 && lvl < max_level => {
            ((xp_now as f32) / (cost as f32) * 100.0).min(100.0)
        }
        _ => 100.0,
    };
    let next_line = if lvl >= max_level {
        "💫 Max level reached — legendary!".to_string()
    } else {
        format!(
            "✨ {} XP to level {}",
            cost_next.unwrap_or(0).saturating_sub(xp_now),
            lvl + 1
        )
    };

    // Identity header — avatar (profile picture or mascot), nickname, title.
    let nickname = state.memory.nickname.clone();
    let learned_name = state.memory.user_name.clone();
    let display_name = match (&nickname, &learned_name) {
        (Some(nick), Some(name)) => format!("{} (@{})", nick, name),
        (Some(nick), None) => nick.clone(),
        (None, Some(name)) => name.clone(),
        (None, None) => "Player".to_string(),
    };
    let title_chip = state
        .memory
        .equipped_title_display()
        .map(|(icon, name)| format!("{} {}", icon, name))
        .unwrap_or_else(|| "no title equipped".to_string());
    let picture_ok = state
        .memory
        .profile_picture
        .as_deref()
        .map(std::path::Path::new)
        .map(|p| p.exists())
        .unwrap_or(false);
    let avatar: Element<Message> = if picture_ok {
        container(
            image(
                state
                    .memory
                    .profile_picture
                    .as_deref()
                    .unwrap_or_default(),
            )
            .width(Length::Fixed(52.0))
            .height(Length::Fixed(52.0)),
        )
        .style(theme::flat(th.accent, th.accent_hover, 26.0))
        .padding(2)
        .into()
    } else {
        container(text("🚀").size(26))
            .style(theme::flat(th.accent, th.accent_hover, 16.0))
            .padding(10)
            .into()
    };
    let header_card: Element<Message> = container(
        row![
            avatar,
            column![
                text(format!("{} — Level {}", display_name, lvl))
                    .size(17)
                    .color(th.text),
                text(format!(
                    "{} • {} • {}",
                    memory::level_title(lvl),
                    title_chip,
                    state.memory.skill_level.label()
                ))
                .size(12)
                .color(th.highlight),
                progress_bar(0.0..=100.0, pct).girth(6.0),
                text(format!(
                    "{} • 🏅 total score: {}",
                    next_line,
                    leveling.total_score()
                ))
                .size(11)
                .color(th.text_dim),
            ]
            .spacing(3)
            .width(Length::Fill),
            column![
                text(format!("💰 {}", leveling.coins))
                    .size(15)
                    .color(th.highlight),
                button(text("🛒 Shop").size(11).color(Color::WHITE))
                    .style(theme::solid(th.accent, th.accent_hover))
                    .on_press(Message::Nav(Page::Shop))
                    .padding([4, 8]),
            ]
            .spacing(4)
            .align_x(iced::alignment::Horizontal::Center),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .style(theme::card(th))
    .width(Length::Fill)
    .padding(12)
    .into();

    // Daily quests.
    let mut quest_col: iced::widget::Column<'_, Message> = column![].spacing(3);
    for q in &state.memory.daily_quests {
        let line = if q.done {
            format!("✅ {} — done! (+{} XP)", q.title, q.xp_reward)
        } else {
            format!("▫️ {} — {}/{}", q.title, q.progress, q.target)
        };
        quest_col = quest_col.push(
            text(line)
                .size(12)
                .color(if q.done { th.highlight } else { th.text }),
        );
    }

    // Badges — chips, four per row.
    let mut badges_col: iced::widget::Column<'_, Message> = column![].spacing(6);
    if leveling.badges.is_empty() {
        badges_col = badges_col.push(
            text("No badges yet — chat with ROCKET to earn some!")
                .size(11)
                .color(th.text_dim),
        );
    } else {
        let mut brow: iced::widget::Row<'_, Message> = row![].spacing(6);
        let mut count = 0;
        for b in &leveling.badges {
            brow = brow.push(
                container(text(b.label()).size(11).color(th.text))
                    .style(theme::flat(th.surface_alt, th.border, 8.0))
                    .padding([4, 8]),
            );
            count += 1;
            if count % 4 == 0 {
                badges_col = badges_col.push(brow);
                brow = row![].spacing(6);
            }
        }
        if count % 4 != 0 {
            badges_col = badges_col.push(brow);
        }
    }

    // Achievements — two per row, locked ones dimmed.
    let mut ach_col: iced::widget::Column<'_, Message> = column![].spacing(4);
    let mut arow: iced::widget::Row<'_, Message> = row![].spacing(10);
    let mut acount = 0;
    for a in &leveling.achievements {
        let (icon, col) = if a.unlocked {
            (a.icon.clone(), th.text)
        } else {
            ("🔒".to_string(), th.text_dim)
        };
        arow = arow.push(
            text(format!(
                "{} {}{}",
                icon,
                a.title,
                if a.unlocked { "" } else { " (locked)" }
            ))
            .size(12)
            .color(col),
        );
        acount += 1;
        if acount % 2 == 0 {
            ach_col = ach_col.push(arow);
            arow = row![].spacing(10);
        }
    }
    if acount % 2 != 0 {
        ach_col = ach_col.push(arow);
    }

    // Profile settings — picture, nickname, bio editors.
    let picture_status = if picture_ok {
        "🖼️ Profile picture set".to_string()
    } else {
        "🖼️ No profile picture yet".to_string()
    };
    // Personality picker — pick how Rocket talks to you.
    let mut vibe_row: iced::widget::Row<'_, Message> = row![].spacing(6);
    for v in vibe::VIBES {
        let current = if state.memory.vibe.is_empty() {
            v.id == vibe::DEFAULT_VIBE_ID
        } else {
            state.memory.vibe == v.id
        };
        let (bg, edge, tc, hov) = if current {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        let mut chip = button(text(v.label).size(11).color(tc))
            .style(theme::chip(bg, edge, tc, hov))
            .padding([5, 8]);
        if !current {
            chip = chip.on_press(Message::SetVibe(v.id.to_string()));
        }
        vibe_row = vibe_row.push(chip);
    }
    // Co-op mission board.
    let missions = state.memory.active_missions();
    let missions_text = if missions.is_empty() {
        "🎯 Nothing on the board yet — tell Rocket \"let's beat the ender dragon\" and we're on it!".to_string()
    } else {
        missions
            .iter()
            .map(|m| format!("▫️ {} ({})", m.title, m.game))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut picture_row: iced::widget::Row<'_, Message> = row![
        text(picture_status).size(12).color(th.text_dim),
        button(text("🖼️ Choose picture…").size(11).color(Color::WHITE))
            .style(theme::solid(th.accent, th.accent_hover))
            .on_press(Message::PickProfilePicture)
            .padding([5, 9]),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    if picture_ok {
        let danger = iced::Color::from_rgb8(0xff, 0x6b, 0x6b);
        picture_row = picture_row.push(
            button(text("❌ Remove").size(11).color(danger))
                .style(theme::chip(th.surface, th.border, danger, th.surface_alt))
                .on_press(Message::ClearProfilePicture)
                .padding([5, 9]),
        );
    }
    let profile_settings_card: Element<Message> = container(
        column![
            text("👤 Profile Settings").size(13).color(th.highlight),
            picture_row,
            row![
                text("🎭").size(13),
                text_input("Set a nickname…", &state.profile_nickname_input)
                    .on_input(Message::ProfileNicknameChanged)
                    .on_submit(Message::SaveNickname)
                    .size(12)
                    .padding(8)
                    .style(theme::input(th))
                    .width(Length::Fill),
                button(text("💾").size(12).color(Color::WHITE))
                    .style(theme::solid(th.accent, th.accent_hover))
                    .on_press(Message::SaveNickname)
                    .padding([6, 9]),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .width(Length::Fill),
            row![
                text("📝").size(13),
                text_input("Write a bio — one line about you…", &state.profile_bio_input)
                    .on_input(Message::ProfileBioChanged)
                    .on_submit(Message::SaveBio)
                    .size(12)
                    .padding(8)
                    .style(theme::input(th))
                    .width(Length::Fill),
                button(text("💾").size(12).color(Color::WHITE))
                    .style(theme::solid(th.accent, th.accent_hover))
                    .on_press(Message::SaveBio)
                    .padding([6, 9]),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .width(Length::Fill),
            text("🤝 Rocket's vibe — how it talks to you:").size(12).color(th.text_dim),
            vibe_row,
            text("🎯 Our mission board:").size(12).color(th.text_dim),
            text(missions_text).size(11).color(th.text),
        ]
        .spacing(8),
    )
    .style(theme::card(th))
    .width(Length::Fill)
    .padding(12)
    .into();

    // Titles gallery — every catalog title, locked ones with their requirement.
    let (unlocked_count, total_titles) = state.memory.title_progress();
    let mut titles_col: iced::widget::Column<'_, Message> = column![].spacing(6);
    let mut trow: iced::widget::Row<'_, Message> = row![].spacing(6);
    let mut tcount = 0;
    for def in memory::PROFILE_TITLES {
        let unlocked = state
            .memory
            .titles
            .iter()
            .any(|t| t.id == def.id && t.unlocked);
        let equipped = state.memory.equipped_title.as_deref() == Some(def.id);
        let label = if equipped {
            format!("✓ {} {}", def.icon, def.name)
        } else if unlocked {
            format!("{} {}", def.icon, def.name)
        } else {
            format!(
                "🔒 {} [{}] · {}",
                def.name,
                def.rarity.label(),
                def.requirement()
            )
        };
        let (bg, edge, tc, hov) = if equipped {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else if unlocked {
            (th.surface, th.border, th.text, th.surface_alt)
        } else {
            (th.bg, th.border, th.text_dim, th.bg)
        };
        let mut btn = button(text(label).size(11).color(tc))
            .style(theme::chip(bg, edge, tc, hov))
            .padding([5, 8]);
        if unlocked {
            btn = btn.on_press(Message::EquipTitle(def.id.to_string()));
        }
        trow = trow.push(btn);
        tcount += 1;
        if tcount % 2 == 0 {
            titles_col = titles_col.push(trow);
            trow = row![].spacing(6);
        }
    }
    if tcount % 2 != 0 {
        titles_col = titles_col.push(trow);
    }

    scrollable(
        column![
            header_card,
            profile_settings_card,
            text(format!(
                "📋 Daily Quests {}",
                if state.memory.all_quests_done() {
                    "— all done! 🎉"
                } else {
                    ""
                }
            ))
            .size(13)
            .color(th.highlight),
            quest_col,
            text("🏅 Badges").size(13).color(th.highlight),
            badges_col,
            text("🏆 Achievements").size(13).color(th.highlight),
            ach_col,
            text(format!(
                "🎖️ Titles — {} of {} unlocked (tap an unlocked one to wear it)",
                unlocked_count, total_titles
            ))
            .size(13)
            .color(th.highlight),
            titles_col,
            text("🎨 Cosmetics live in the 🛒 XP Shop — spend coins to unlock and equip them.")
                .size(11)
                .color(th.text_dim),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Shop page — coin balance, mystery boxes, decoration collection.
fn shop_page<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    let leveling = &state.memory.leveling;

    // Decorations — three per row; equipped/unlocked equip, affordable buy.
    let mut deco_col: iced::widget::Column<'_, Message> = column![].spacing(6);
    let mut drow: iced::widget::Row<'_, Message> = row![].spacing(6);
    let mut dcount = 0;
    for d in &leveling.profile_decorations {
        let price = memory::decoration_price(&d.rarity);
        let affordable = !d.unlocked && price > 0 && leveling.coins >= price;
        let label = if d.equipped {
            format!("{} {} ✓", d.icon, d.name)
        } else if d.unlocked {
            format!("{} {}", d.icon, d.name)
        } else if price > 0 {
            format!("🔒 {} · {}🪙", d.name, price)
        } else {
            format!("🔒 {}", d.name)
        };
        let (bg, edge, tc, hov) = if d.equipped {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else if d.unlocked {
            (th.surface, th.border, th.text, th.surface_alt)
        } else if affordable {
            (th.surface_alt, th.highlight, th.highlight, th.surface)
        } else {
            (th.bg, th.border, th.text_dim, th.bg)
        };
        let mut btn = button(text(label).size(11).color(tc))
            .style(theme::chip(bg, edge, tc, hov))
            .padding([5, 8]);
        if d.unlocked {
            btn = btn.on_press(Message::EquipDecoration(d.id.clone()));
        } else if affordable {
            btn = btn.on_press(Message::BuyDecoration(d.id.clone()));
        }
        drow = drow.push(btn);
        dcount += 1;
        if dcount % 3 == 0 {
            deco_col = deco_col.push(drow);
            drow = row![].spacing(6);
        }
    }
    if dcount % 3 != 0 {
        deco_col = deco_col.push(drow);
    }

    scrollable(
        column![
            container(
                row![
                    column![
                        text("💰 Coin Bank").size(18).color(th.text),
                        text(format!(
                            "{} coins earned from chats, quests & badges",
                            leveling.coins
                        ))
                        .size(11)
                        .color(th.text_dim),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    button(
                        text(format!("🎁 Open Mystery Box — {}🪙", memory::MYSTERY_BOX_PRICE))
                            .size(12)
                            .color(Color::WHITE)
                    )
                    .style(theme::solid(th.accent, th.accent_hover))
                    .on_press(Message::BuyMysteryBox)
                    .padding([8, 12]),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
            )
            .style(theme::card(th))
            .width(Length::Fill)
            .padding(12),
            text("🎨 Decoration Collection — click ✓ to equip, click a 🔒 you can afford to buy it:")
                .size(13)
                .color(th.highlight),
            deco_col,
            text("Rarity prices: Common 300🪙 · Rare 800🪙 · Epic 1500🪙 · Legendary 3000🪙 — or gamble on a mystery box.")
                .size(11)
                .color(th.text_dim),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Knowledge page — searchable curated guides.
fn knowledge_page<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    let results: Element<Message> = if state.kb_search.trim().is_empty() {
        text("Type below to search every guide for this game…")
            .size(12)
            .color(th.text_dim)
            .into()
    } else {
        let hits = state
            .kb
            .search(&state.current_game, state.kb_search.trim(), 5);
        if hits.is_empty() {
            text("No curated matches here — try the exact item name, or ask in chat (the universal item brain will handle it).")
                .size(12)
                .color(th.text_dim)
                .into()
        } else {
            text(knowledge::format_answer(&hits))
                .size(12)
                .color(th.text)
                .into()
        }
    };

    container(
        column![
            text_input(
                "Search guides… (e.g. diamonds, loadout, smithing)",
                &state.kb_search
            )
            .on_input(Message::KbSearchChanged)
            .size(13)
            .padding(8)
            .style(theme::input(th)),
            scrollable(results)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Cheat sheet page — the full guide for the current game.
fn cheatsheet_page<'a>(state: &'a App, th: theme::UiTheme) -> Element<'a, Message> {
    container(
        scrollable(
            text(state.get_cheat_sheet())
                .size(12)
                .color(th.text)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Settings page — detection, AI mode, memory.
fn settings_page<'a>(state: &'a App, th: theme::UiTheme, danger: Color) -> Element<'a, Message> {
    let auto_detect_label = if state.auto_detect {
        "🟢 Auto-detect ON"
    } else {
        "🔴 Auto-detect OFF"
    };
    let ai_api_label = if state.use_ai_api {
        "🟢 Online AI ON"
    } else {
        "💻 Local AI"
    };

    // --- Online AI key setup -------------------------------------------------
    let ai_status = if state.use_ai_api && state.llm_settings.is_ready() {
        format!(
            "🟢 Online AI is live — answering with {} ({})",
            state.llm_settings.provider.label(),
            state.llm_settings.model
        )
    } else if state.llm_settings.is_ready() {
        "🟢 Key saved — flip \"💻 Local AI\" to \"🟢 Online AI ON\" (button above) to use it."
            .to_string()
    } else {
        "🟡 Pick a provider and paste your API key below — it stays on this PC (rocket-llm.json)."
            .to_string()
    };
    let provider_chip_name = |p: llm::Provider| -> &'static str {
        match p {
            llm::Provider::Groq => "Groq ⭐",
            llm::Provider::Gemini => "Gemini",
            llm::Provider::OpenRouter => "OpenRouter",
            llm::Provider::OpenAi => "OpenAI",
            llm::Provider::Cerebras => "Cerebras ⚡",
            llm::Provider::Mistral => "Mistral",
            llm::Provider::DeepSeek => "DeepSeek",
            llm::Provider::Together => "Together",
            llm::Provider::Ollama => "Ollama 🏠",
            llm::Provider::LMStudio => "LM Studio 🏠",
            llm::Provider::Custom => "Custom URL",
        }
    };
    let mut provider_rows: iced::widget::Column<'_, Message> = column![].spacing(6);
    let mut prow: iced::widget::Row<'_, Message> = row![].spacing(6);
    let mut pcount = 0;
    for p in llm::Provider::ALL {
        let selected = state.llm_settings.provider == *p;
        let (bg, edge, tc, hov) = if selected {
            (th.accent, th.accent, Color::WHITE, th.accent_hover)
        } else {
            (th.surface, th.border, th.text, th.surface_alt)
        };
        let mut chip = button(text(provider_chip_name(*p)).size(11).color(tc))
            .style(theme::chip(bg, edge, tc, hov))
            .padding([5, 8]);
        if !selected {
            chip = chip.on_press(Message::SetProvider(*p));
        }
        prow = prow.push(chip);
        pcount += 1;
        if pcount % 4 == 0 {
            provider_rows = provider_rows.push(prow);
            prow = row![].spacing(6);
        }
    }
    if pcount % 4 != 0 {
        provider_rows = provider_rows.push(prow);
    }

    let mut ai_card_col: iced::widget::Column<'_, Message> = column![
        text("🌐 Online AI — paste your API key here").size(13).color(th.highlight),
        text(ai_status).size(11).color(th.text_dim),
        provider_rows,
        text(state.llm_settings.provider.key_hint()).size(11).color(th.highlight),
        text_input("Paste your API key here…", &state.llm_settings.api_key)
            .on_input(Message::SetApiKey)
            .size(12)
            .padding(8)
            .style(theme::input(th))
            .width(Length::Fill),
        text_input(
            "Model — the default is filled in for you",
            &state.llm_settings.model,
        )
        .on_input(Message::SetModel)
        .size(12)
        .padding(8)
        .style(theme::input(th))
        .width(Length::Fill),
        row![
            button(text("🔌 Test connection").size(11).color(Color::WHITE))
                .style(theme::solid(th.accent, th.accent_hover))
                .on_press(Message::TestAiApi)
                .padding([6, 10]),
            text("Sends a real \"ping\" message so you know the key works.")
                .size(10)
                .color(th.text_dim),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8);
    if state.llm_settings.provider == llm::Provider::Custom {
        ai_card_col = ai_card_col.push(
            text_input(
                "Custom endpoint URL (OpenAI-compatible, …/v1)",
                &state.llm_settings.custom_url,
            )
            .on_input(Message::SetCustomUrl)
            .size(12)
            .padding(8)
            .style(theme::input(th))
            .width(Length::Fill),
        );
    }
    let ai_card: Element<Message> = container(ai_card_col)
        .style(theme::card(th))
        .width(Length::Fill)
        .padding(12)
        .into();

    container(
        column![
            ai_card,
            text("Toggles").size(13).color(th.highlight),
            row![
                button(text(auto_detect_label).size(12).color(th.text))
                    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                    .on_press(Message::ToggleAutoDetect)
                    .padding([6, 10]),
                button(text(ai_api_label).size(12).color(th.text))
                    .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                    .on_press(Message::ToggleAiApi)
                    .padding([6, 10]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text("About your memory").size(13).color(th.highlight),
            text(format!(
                "🧠 {} facts remembered • {} conversations • skill: {}\n💰 {} coins • ⭐ level {} • {} badges • {} achievements",
                state.memory.facts.len(),
                state.memory.conversation_history.len(),
                state.memory.skill_level.label(),
                state.memory.leveling.coins,
                state.memory.leveling.current_level,
                state.memory.leveling.badges.len(),
                state.memory
                    .leveling
                    .achievements
                    .iter()
                    .filter(|a| a.unlocked)
                    .count(),
            ))
            .size(11)
            .color(th.text_dim),
            button(text("🧹 Forget everything (keeps your level & rewards)").size(12).color(danger))
                .style(theme::chip(th.surface, danger, danger, th.surface_alt))
                .on_press(Message::ForgetMemory)
                .padding([6, 10]),
            text("Tip: 🎨 in the sidebar cycles through 5 hand-tuned themes.")
                .size(11)
                .color(th.text_dim),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// The compact overlay bar: detection dot, level/coins, last answer, input.
fn compact_view<'a>(
    state: &'a App,
    th: theme::UiTheme,
    danger: Color,
    online_green: Color,
) -> Element<'a, Message> {
    let detection = if state.current_game != GameType::Generic {
        text(format!("🟢 {}", state.current_game.get_name()))
            .size(11)
            .color(online_green)
    } else {
        text("🔴 No game".to_string()).size(11).color(danger)
    };

    let last_bot = state.messages.iter().rev().find(|m| m.role == "bot");
    let answer_text: Element<Message> = match last_bot {
        Some(msg) => text(msg.content.clone()).size(12).color(th.text).into(),
        None => text("Ask me anything — 📎 attaches a picture for analysis.")
            .size(12)
            .color(th.text_dim)
            .into(),
    };

    let mut content: iced::widget::Column<'_, Message> =
        column![].spacing(6).width(Length::Fill).height(Length::Fill);
    content = content.push(
        row![
            text("🚀").size(14),
            detection,
            Space::new().width(Length::Fill),
            text(format!(
                "⭐ Lv {} · {}🪙",
                state.memory.leveling.current_level, state.memory.leveling.coins
            ))
            .size(11)
            .color(th.highlight),
            button(text("⤢ Expand").size(11).color(th.text))
                .style(theme::chip(th.surface, th.border, th.text, th.surface_alt))
                .on_press(Message::ToggleCompact)
                .padding([2, 6]),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    );
    content = content.push(
        scrollable(container(answer_text).width(Length::Fill)).height(Length::Fill),
    );
    content = content.push(
        row![
            text_input("Ask…", &state.input_text)
                .on_input(Message::InputChanged)
                .on_submit(Message::Send)
                .size(12)
                .padding(6)
                .style(theme::input(th))
                .width(Length::Fill),
            button(text("➤").size(12).color(Color::WHITE))
                .style(theme::solid(th.accent, th.accent_hover))
                .on_press(Message::Send)
                .padding([4, 8]),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    );

    container(content)
        .style(theme::flat(th.bg, th.border, 10.0))
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn main() -> iced::Result {
    println!("ROCKET AI - Starting desktop application...");

    iced::application(App::new, update, view)
        .subscription(subscription)
        .theme(Theme::Dark)
        .title("ROCKET")
        .window_size((1020.0, 780.0))
        .window(iced::window::Settings {
            icon: rocket_icon(),
            ..iced::window::Settings::default()
        })
        .centered()
        .run()
}

/// Loads the mascot as the window/taskbar icon (raw 48x48 RGBA, embedded at compile time).
fn rocket_icon() -> Option<iced::window::Icon> {
    let rgba = include_bytes!("../assets/rocket-icon.rgba").to_vec();
    iced::window::icon::from_rgba(rgba, 48, 48).ok()
}

/// Polls running processes every 6 seconds while auto-detect is enabled.
fn subscription(state: &App) -> iced::Subscription<Message> {
    if !state.auto_detect {
        return iced::Subscription::none();
    }
    iced::time::every(std::time::Duration::from_secs(6)).map(|_| Message::DetectGameTick)
}

// Helper function to create periodic rocket tick
fn rocket_tick() -> Task<Message> {
    Task::batch([
        Task::perform(
            async {
                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
                Message::RocketTick
            },
            std::convert::identity
        ),
        Task::none()
    ])
}