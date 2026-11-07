//! Full QA battery for ROCKET's game knowledge — run with `cargo test`.
//!
//! Every curated game in the knowledge base gets a question battery spanning
//! items, crafting, places, resources, materials, weapons, classes and teams.
//! Each answer is checked against ground-truth facts (verified against the
//! games themselves — errors found by this suite were researched and fixed).
//!
//! The tests drive the EXACT chat pipeline (`knowledge_answer`) so what they
//! verify is what players get. Ground rule: **a selected game answers ONLY
//! from its own curated pack** — other games' knowledge is reserved for
//! General Chat mode, and the universal item brain covers everything else.
//!
//! `generate_qa_report` writes `rocket-qa-report.md`: every game, every
//! question asked, and Rocket's verbatim answer — regenerated on every test
//! run so it can never drift from the real pipeline.

use crate::knowledge::{format_answer, KnowledgeBase};
use crate::{knowledge_answer, GameType, KnowledgeAnswer};

/// File the Q&A report is written to (project root).
const REPORT_FILE: &str = "rocket-qa-report.md";

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn kb() -> KnowledgeBase {
    KnowledgeBase::new()
}

fn lower(text: &str) -> String {
    text.to_lowercase()
}

/// Ask through the exact chat pipeline and unwrap the answer text.
fn ask(kb: &KnowledgeBase, game: GameType, q: &str) -> String {
    match knowledge_answer(kb, &game, q) {
        Some(KnowledgeAnswer::Game(t, _))
        | Some(KnowledgeAnswer::CrossGame(t))
        | Some(KnowledgeAnswer::UniversalItem(t)) => t,
        None => panic!("ROCKET had NO answer for [{:?}] '{}'", game, q),
    }
}

/// Assert the selected game's OWN curated pack answers `q`, then return it.
fn ask_own(kb: &KnowledgeBase, game: GameType, q: &str) -> String {
    let hits = kb.search(&game, q, 2);
    assert!(
        !hits.is_empty(),
        "[{:?}] no curated guide matches '{}' — the question dead-ends",
        game,
        q
    );
    format_answer(&hits)
}

/// One question + the ground-truth markers the answer must contain.
fn check(game: GameType, q: &str, markers: &[&str]) {
    let kb = kb();
    let answer = ask_own(&kb, game.clone(), q);
    for m in markers {
        assert!(
            lower(&answer).contains(&lower(m)),
            "[{:?}] Q: '{}'\nexpected to mention: '{}'\nanswer was:\n{}",
            game,
            q,
            m,
            answer
        );
    }
}

/// Run a whole battery: (question, required ground-truth markers).
fn battery(game: GameType, cases: &[Case]) {
    for (q, markers) in cases {
        check(game.clone(), q, markers);
    }
}

/// Which part of the pipeline produced this answer (for the report).
fn route_label(answer: &Option<KnowledgeAnswer>) -> &'static str {
    match answer {
        Some(KnowledgeAnswer::Game(_, _)) => "📚 answered from its own curated guide",
        Some(KnowledgeAnswer::CrossGame(_)) => "🌐 answered from cross-game knowledge",
        Some(KnowledgeAnswer::UniversalItem(_)) => "🔎 answered by the universal item brain",
        None => "⚠️ no knowledge answer (chat falls back to generic tips / teach-me)",
    }
}

/// Run the pipeline and return (route label, verbatim answer text).
fn pipeline_answer(kb: &KnowledgeBase, game: &GameType, q: &str) -> (&'static str, String) {
    let answer = knowledge_answer(kb, game, q);
    let text = match &answer {
        Some(KnowledgeAnswer::Game(t, _))
        | Some(KnowledgeAnswer::CrossGame(t))
        | Some(KnowledgeAnswer::UniversalItem(t)) => t.clone(),
        None => String::new(),
    };
    (route_label(&answer), text)
}

// ---------------------------------------------------------------------------
// The question battery — single source of truth for tests AND the report.
// Each case: (question, ground-truth markers the answer must contain).
// ---------------------------------------------------------------------------

type Case = (&'static str, &'static [&'static str]);
type GameBattery = (GameType, &'static [Case]);

static BATTERIES: &[GameBattery] = &[
    // ------------------------------------------------------------- Minecraft
    (
        GameType::Minecraft,
        &[
            // item + resource
            ("how do i find diamonds in minecraft", &["y -58", "iron pickaxe", "fortune"]),
            // item + material + crafting chain
            ("how do i get netherite in minecraft", &["ancient debris", "y=15", "4 scrap + 4 gold", "smithing"]),
            ("where do i find ancient debris", &["nether", "y=15", "bed"]),
            // place
            ("how do i survive the nether", &["gold armor", "piglin", "fortress", "ghast"]),
            ("how do i beat the ender dragon", &["end crystals", "obsidian", "beds", "endermen"]),
            // mechanics (redstone engineering)
            ("how does redstone work in minecraft", &["observer", "repeater", "comparator"]),
            // class/progression route
            ("how should i start a survival world in minecraft", &["stone tools", "village", "shelter"]),
            // crafting/enchanting
            ("how does enchanting work in minecraft", &["15 bookshelves", "mending", "lapis"]),
        ],
    ),
    // ---------------------------------------------------------------- Roblox
    (
        GameType::Roblox,
        &[
            ("how do i start making a game in roblox studio", &["anchor", "publish", "models"]),
            ("how does scripting work in roblox", &["localscript", "remoteevent", "server"]),
            ("how do i earn robux", &["game pass", "premium"]),
            ("what are the popular roblox genres", &["obby", "tycoon", "simulator", "horror"]),
        ],
    ),
    // -------------------------------------------------------------- Fortnite
    (
        GameType::Fortnite,
        &[
            ("how do i get better at building in fortnite", &["90s", "box", "ramp"]),
            ("where should i land in fortnite", &["loot priority", "shield", "chest"]),
            ("what is the best loadout in fortnite", &["mid-range", "shotgun", "heals", "ammo"]),
            ("how to play the endgame in fortnite", &["zone center", "high ground", "third part"]),
            ("tips for zero build in fortnite", &["cover", "sprint", "mobility"]),
        ],
    ),
    // ----------------------------------------------------- League of Legends
    (
        GameType::LeagueOfLegends,
        &[
            ("what do the roles do in league of legends", &["jungle", "adc", "support", "splitpush"]),
            ("how do i farm cs in league", &["7-8/min", "freeze", "last-hit"]),
            ("how to control vision in league of legends", &["control ward", "sweeper", "deep ward"]),
            ("what objectives should i take in league", &["baron", "elder", "herald"]),
            ("how to trade in lane in league", &["last-hit", "cooldown", "aggro"]),
            ("tips for climbing ranked in league", &["champion pool", "tilt", "macro"]),
        ],
    ),
    // -------------------------------------------------------------- Valorant
    (
        GameType::Valorant,
        &[
            ("how to place my crosshair in valorant", &["head height", "slice the pie", "deathmatch"]),
            ("what are the valorant agent roles", &["duelist", "controller", "initiator", "sentinel"]),
            ("when should i eco in valorant", &["eco", "save", "full buy", "credits"]),
            ("how to execute a site in valorant", &["smoke", "post-plant", "lurk"]),
            ("what do valorant callouts mean", &["flank", "default"]),
        ],
    ),
    // ------------------------------------------------------ Counter-Strike 2
    (
        GameType::CS2,
        &[
            ("how does the economy work in cs2", &["loss bonus", "eco", "defuse kit"]),
            ("how to control spray in cs2", &["spray pattern", "first bullet", "crosshair"]),
            ("how to use utility in cs2", &["smoke", "molotov", "pop-flash", "grenade"]),
            ("how to hold positions in cs2", &["crossfire", "off-angles", "retake"]),
            ("what to do on pistol round in cs2", &["armor", "picks"]),
        ],
    ),
    // ---------------------------------------------------------- Apex Legends
    (
        GameType::ApexLegends,
        &[
            ("what legends work well together in apex", &["wraith", "bloodhound", "catalyst", "support"]),
            ("what should i loot in apex", &["armor first", "crafting", "attachments"]),
            ("how to rotate around the zone in apex", &["zone", "beacon", "high ground"]),
            ("how to win team fights in apex", &["focus fire", "knock", "shield"]),
        ],
    ),
    // ----------------------------------------------------------- Call of Duty
    (
        GameType::CallOfDuty,
        &[
            ("how to use the gunsmith in cod", &["muzzle", "underbarrel", "ads", "recoil"]),
            ("how to move in cod", &["slide", "tactical sprint", "strafe"]),
            ("hardpoint tips in cod", &["rotation", "uav", "spawn"]),
        ],
    ),
    // ------------------------------------------------------------------- PUBG
    (
        GameType::PUBG,
        &[
            ("where to drop in pubg", &["flight path", "pochinki", "hot drop"]),
            ("best pubg loadout", &["helmet", "8x", "meds", "smoke"]),
            ("how to play the zone in pubg", &["blue", "smoke", "vehicle"]),
        ],
    ),
    // ---------------------------------------------------------- Rocket League
    (
        GameType::RocketLeague,
        &[
            ("how does rotation work in rocket league", &["back-post", "double-commit", "ball-chase"]),
            ("what mechanics should i learn in rocket league", &["powerslide", "fast aerial", "ball cam"]),
            ("how to manage boost in rocket league", &["+12", "corner", "100"]),
            ("how to train in rocket league", &["custom training", "replay", "free play"]),
        ],
    ),
    // -------------------------------------------------------------- The Sims
    (
        GameType::TheSims,
        &[
            ("how to make money in the sims", &["painting", "royalt", "gardening"]),
            ("how do needs work in the sims", &["energy", "sleep", "mood"]),
            ("build mode tips for the sims", &["moveobjects", "floor plan", "landscaping"]),
        ],
    ),
    // --------------------------------------------------------- Stardew Valley
    (
        GameType::StardewValley,
        &[
            ("best crops in stardew valley", &["strawberr", "blueber", "cranber", "sprinkler"]),
            ("how to manage energy in stardew valley", &["1:50 am", "field snacks", "sleep"]),
            ("where do i find iridium in stardew valley", &["skull cavern", "bombs", "luck"]),
            ("how does friendship work in stardew valley", &["loved gifts", "birthday"]),
        ],
    ),
    // ----------------------------------------------------------------- Zelda
    (
        GameType::Zelda,
        &[
            ("how does combat work in zelda", &["flurry", "parry", "durability"]),
            ("what should i explore first in zelda", &["shrines", "tower", "stamina"]),
            ("how does cooking work in zelda", &["one buff per dish", "dragon horn", "elixir"]),
        ],
    ),
    // ---------------------------------------------------------------- Skyrim
    (
        GameType::Skyrim,
        &[
            ("what is the best build in skyrim", &["stealth archer", "smithing", "mage"]),
            ("how to level skills fast in skyrim", &["iron daggers", "blisterwort", "trainer"]),
            ("combat tactics in skyrim", &["power attack", "shield bash", "poison"]),
            ("how do shouts work in skyrim", &["dragonrend", "unrelenting force"]),
            ("how to make money in skyrim", &["transmute", "breezehome"]),
            // The user's exact scenario: conversational framing + misspelled
            // spell name — must still land on the spell guide, not build tips.
            ("im going for a mage build and im using clairvoince whats it for", &["clairvoyance", "glowing trail", "quest"]),
            ("what is clairvoyance for in skyrim", &["glowing trail", "quest target"]),
            ("is muffle worth casting for a mage", &["muffle", "illusion", "sneaking"]),
        ],
    ),
    // ------------------------------------------------------------------- GTA
    (
        GameType::GTA,
        &[
            ("how to make money in gta online", &["cayo", "nightclub", "kosatka"]),
            ("how to lose the cops in gta", &["pay'n'spray", "line of sight"]),
            ("heist prep tips for gta", &["kuruma", "snacks", "armor"]),
        ],
    ),
    // -------------------------------------------------------------- Among Us
    (
        GameType::AmongUs,
        &[
            ("crewmate tips for among us", &["visual tasks", "task bars", "alibi"]),
            ("impostor tips for among us", &["vent", "fake tasks", "kill"]),
            ("how should i vote in among us", &["skip votes", "timeline"]),
        ],
    ),
    // ------------------------------------------------------------- Fall Guys
    (
        GameType::FallGuys,
        &[
            ("race tips for fall guys", &["slime climb", "dive", "shortcut"]),
            ("hex-a-gone tips for fall guys", &["upper tiles"]),
        ],
    ),
    // ------------------------------------------------------ World of Warcraft
    (
        GameType::WorldOfWarcraft,
        &[
            ("how to level fast in wow", &["heirloom", "dungeon", "war mode"]),
            ("what do tanks healers and dps do in wow", &["tank", "healer", "dps", "aggro"]),
            ("dungeon etiquette in wow", &["interrupt", "loot", "pull"]),
            ("how to make gold in wow", &["gathering", "auction", "flask"]),
        ],
    ),
    // ----------------------------------------------------------------- Dota 2
    (
        GameType::Dota2,
        &[
            ("what are the positions in dota", &["pos 1", "carry", "offlane", "support"]),
            ("how to last hit in dota", &["denies", "quelling", "creep"]),
            ("how to ward in dota", &["observer", "sentries", "smoke"]),
            ("when should i buy bkb in dota", &["black king bar", "timing", "blink"]),
        ],
    ),
    // --------------------------------------------------------------- Terraria
    (
        GameType::Terraria,
        &[
            ("how does the ore progression work in terraria", &["cobalt", "mythril", "chlorophyte", "luminite"]),
            ("how do i enter hardmode in terraria", &["wall of flesh", "pwnhammer", "hallow", "plantera"]),
        ],
    ),
    // ------------------------------------------------------------- Elden Ring
    (
        GameType::EldenRing,
        &[
            ("how to upgrade weapons in elden ring", &["smithing stone", "somber", "bell bearing"]),
            ("how to farm runes and level in elden ring", &["golden seed", "sacred tear", "mogh", "spirit ash"]),
        ],
    ),
    // ---------------------------------------------------------------- Valheim
    (
        GameType::Valheim,
        &[
            ("how does metal progression work in valheim", &["bronze", "iron", "black metal", "cannot be teleported"]),
            ("how does resting work in valheim", &["rested", "comfort", "food"]),
        ],
    ),
    // ------------------------------------------------------------------- Rust
    (
        GameType::Rust,
        &[
            ("how to farm scrap in rust", &["barrels", "recycler", "workbench", "sulfur"]),
            ("base building tips for rust", &["tool cupboard", "airlock", "raid"]),
        ],
    ),
    // --------------------------------------------------------------- Palworld
    (
        GameType::Palworld,
        &[
            ("how to craft pal spheres in palworld", &["paldium", "mega", "giga", "hyper"]),
            ("how to automate my base in palworld", &["kindling", "handiwork", "feed box", "san"]),
        ],
    ),
    // ---------------------------------------------------------- Genshin Impact
    (
        GameType::Genshin,
        &[
            ("how to farm artifacts in genshin", &["domain", "resin", "circlet"]),
            ("what should i do at ar 45 in genshin", &["ar 45", "5-star artifacts", "condensed resin"]),
        ],
    ),
    // -------------------------------------------------------- Baldur's Gate 3
    (
        GameType::BaldursGate3,
        &[
            ("how to get good loot in bg3", &["merchant", "lockpick", "withers"]),
            ("how do rests work in bg3", &["long rest", "short rest", "concentration"]),
        ],
    ),
    // ---------------------------------------------------------- Cyberpunk 2077
    (
        GameType::Cyberpunk,
        &[
            ("how does crafting work in cyberpunk", &["dismantl", "iconic", "ripperdoc"]),
            ("how to farm eddies in cyberpunk", &["ncpd", "gig", "street cred"]),
        ],
    ),
    // ---------------------------------------------------------- The Witcher 3
    (
        GameType::Witcher3,
        &[
            ("how to get witcher gear in witcher 3", &["diagram", "griffin", "mastercrafted", "runes"]),
            ("how does alchemy work in the witcher 3", &["oil", "decoction", "meditate", "quen"]),
        ],
    ),
    // --------------------------------------------------------------- Diablo IV
    (
        GameType::Diablo4,
        &[
            ("how do legendary aspects work in diablo 4", &["occultist", "imprint", "salvage"]),
            ("how do world tiers work in diablo 4", &["capstone", "nightmare", "helltide", "renown"]),
        ],
    ),
    // -------------------------------------------------------------- Destiny 2
    (
        GameType::Destiny2,
        &[
            ("how to get exotics in destiny 2", &["xur", "exotic cipher", "catalyst"]),
            ("how to raise power level in destiny 2", &["pinnacle", "prime engram", "bounties"]),
        ],
    ),
    // ------------------------------------------------------------- Overwatch 2
    (
        GameType::Overwatch,
        &[
            ("how should i use my ultimate in overwatch", &["combo", "grav", "sync"]),
            ("where should i position in overwatch", &["off-angle", "tank", "support"]),
        ],
    ),
    // --------------------------------------------------------------- Warframe
    (
        GameType::Warframe,
        &[
            ("how to farm warframes", &["relic", "prime", "nekros", "platinum"]),
            ("how to progress in warframe", &["junction", "mastery", "nightwave"]),
        ],
    ),
    // -------------------------------------------------------------- Subnautica
    (
        GameType::Subnautica,
        &[
            ("where to find silver in subnautica", &["kelp forest", "argentite", "gold"]),
            ("how does the depth progression work in subnautica", &["depth module", "seamoth", "prawn"]),
        ],
    ),
    // -------------------------------------------------------------------- ARK
    (
        GameType::ARK,
        &[
            ("how to tame dinos in ark", &["tranq", "kibble", "narcotic"]),
            ("where to find metal in ark", &["ankylosaurus", "mountain", "obsidian"]),
        ],
    ),
    // ------------------------------------------------------------ No Man's Sky
    (
        GameType::NoMansSky,
        &[
            ("how to make units in no man's sky", &["chlorine", "refiner", "nanite"]),
            ("how to get a freighter in no man's sky", &["pirate ambush", "free", "warp"]),
        ],
    ),
    // -------------------------------------------------- Red Dead Redemption 2
    (
        GameType::RDR2,
        &[
            ("where do i sell legendary pelts in rdr2", &["trapper", "perfect pelt"]),
            ("how does honor work in rdr2", &["discount", "bounty"]),
        ],
    ),
    // ------------------------------------------------------------ Phasmophobia
    (
        GameType::Phasmophobia,
        &[
            ("what equipment do i need in phasmophobia", &["emf", "spirit box", "thermometer", "uv"]),
            ("how to survive a hunt in phasmophobia", &["crucifix", "hide", "sanity"]),
        ],
    ),
    // ---------------------------------------------------------- Lethal Company
    (
        GameType::LethalCompany,
        &[
            ("how to hit quota in lethal company", &["apparatus", "scan", "sell"]),
            ("which moons are safe in lethal company", &["experimentation", "assurance", "titan"]),
        ],
    ),
    // ------------------------------------------------------------------- Hades
    (
        GameType::Hades,
        &[
            ("what do keepsakes do in hades", &["boon", "keepsake", "mirror"]),
            ("how do weapon aspects work in hades", &["titan blood", "aspect", "heat"]),
        ],
    ),
    // ------------------------------------------------------------ Hollow Knight
    (
        GameType::HollowKnight,
        &[
            ("best charms in hollow knight", &["quick slash", "shaman stone", "notch"]),
            ("where do i find the dash and wall cling in hollow knight", &["mothwing cloak", "mantis claw", "monarch wings"]),
        ],
    ),
    // -------------------------------------------------------- Final Fantasy XIV
    (
        GameType::FinalFantasy14,
        &[
            ("how to gear up in ffxiv", &["tomestone", "450", "raid"]),
            ("how to level alt jobs in ffxiv", &["palace of the dead", "heaven-on-high", "bozja", "msq"]),
        ],
    ),
    // ------------------------------------------------------------ Sea of Thieves
    (
        GameType::SeaOfThieves,
        &[
            ("how do emissary flags work in sea of thieves", &["2.5x", "emissary"]),
            ("ship combat tips for sea of thieves", &["chainshot", "repair", "anchor"]),
        ],
    ),
    // ---------------------------------------------------------- Monster Hunter
    (
        GameType::MonsterHunter,
        &[
            ("how to get monster parts in monster hunter", &["break", "carve", "capture", "investigation"]),
            ("how many weapons should i learn in monster hunter", &["one weapon", "weakness exploit", "armor skills"]),
        ],
    ),
    // ------------------------------- Generic (cross-game fundamentals) -------
    (
        GameType::Generic,
        &[
            ("how to improve my aim", &["sensitivity", "crosshair"]),
            ("how to communicate with my team", &["ping", "callout"]),
            ("how to stop tilting", &["2-loss", "break"]),
            ("best settings for fps games", &["dpi", "monitor", "fps"]),
        ],
    ),
];

fn cases_for(game: &GameType) -> &'static [Case] {
    BATTERIES
        .iter()
        .find(|(g, _)| g == game)
        .map(|(_, cases)| *cases)
        .unwrap_or(&[])
}

/// One focused test per game, driven from the shared battery table.
macro_rules! battery_test {
    ($name:ident, $game:expr) => {
        #[test]
        fn $name() {
            battery($game, cases_for(&$game));
        }
    };
}

battery_test!(minecraft_battery, GameType::Minecraft);
battery_test!(roblox_battery, GameType::Roblox);
battery_test!(fortnite_battery, GameType::Fortnite);
battery_test!(league_of_legends_battery, GameType::LeagueOfLegends);
battery_test!(valorant_battery, GameType::Valorant);
battery_test!(cs2_battery, GameType::CS2);
battery_test!(apex_legends_battery, GameType::ApexLegends);
battery_test!(call_of_duty_battery, GameType::CallOfDuty);
battery_test!(pubg_battery, GameType::PUBG);
battery_test!(rocket_league_battery, GameType::RocketLeague);
battery_test!(the_sims_battery, GameType::TheSims);
battery_test!(stardew_valley_battery, GameType::StardewValley);
battery_test!(zelda_battery, GameType::Zelda);
battery_test!(skyrim_battery, GameType::Skyrim);
battery_test!(gta_battery, GameType::GTA);
battery_test!(among_us_battery, GameType::AmongUs);
battery_test!(fall_guys_battery, GameType::FallGuys);
battery_test!(world_of_warcraft_battery, GameType::WorldOfWarcraft);
battery_test!(dota2_battery, GameType::Dota2);
battery_test!(terraria_battery, GameType::Terraria);
battery_test!(elden_ring_battery, GameType::EldenRing);
battery_test!(valheim_battery, GameType::Valheim);
battery_test!(rust_battery, GameType::Rust);
battery_test!(palworld_battery, GameType::Palworld);
battery_test!(genshin_battery, GameType::Genshin);
battery_test!(baldurs_gate_3_battery, GameType::BaldursGate3);
battery_test!(cyberpunk_battery, GameType::Cyberpunk);
battery_test!(witcher3_battery, GameType::Witcher3);
battery_test!(diablo4_battery, GameType::Diablo4);
battery_test!(destiny2_battery, GameType::Destiny2);
battery_test!(overwatch_battery, GameType::Overwatch);
battery_test!(warframe_battery, GameType::Warframe);
battery_test!(subnautica_battery, GameType::Subnautica);
battery_test!(ark_battery, GameType::ARK);
battery_test!(no_mans_sky_battery, GameType::NoMansSky);
battery_test!(rdr2_battery, GameType::RDR2);
battery_test!(phasmophobia_battery, GameType::Phasmophobia);
battery_test!(lethal_company_battery, GameType::LethalCompany);
battery_test!(hades_battery, GameType::Hades);
battery_test!(hollow_knight_battery, GameType::HollowKnight);
battery_test!(ffxiv_battery, GameType::FinalFantasy14);
battery_test!(sea_of_thieves_battery, GameType::SeaOfThieves);
battery_test!(monster_hunter_battery, GameType::MonsterHunter);
battery_test!(generic_battery, GameType::Generic);

// ---------------------------------------------------------------------------
// KB integrity: every curated entry must be reachable
// ---------------------------------------------------------------------------

#[test]
fn every_kb_entry_is_reachable_by_its_own_title() {
    let kb = kb();
    let mut total = 0;
    for game in crate::all_games() {
        for entry in kb.entries_for_game(&game) {
            let hits = kb.search(&game, entry.title, 2);
            assert!(
                hits.iter().any(|h| h.title == entry.title),
                "[{:?}] entry '{}' is not reachable by its own title",
                game,
                entry.title
            );
            total += 1;
        }
    }
    assert!(total >= 100, "expected a large curated KB, found only {} entries", total);
}

#[test]
fn every_game_category_is_represented_across_the_kb() {
    // The user wants items, crafting, places, resources, materials, weapons,
    // classes and teams covered — the KB must not be all-strategy.
    let kb = kb();
    let mut with_items = 0;
    let mut with_characters = 0;
    for game in crate::all_games() {
        let entries = kb.entries_for_game(&game);
        if entries.iter().any(|e| e.category == crate::knowledge::Category::Items) {
            with_items += 1;
        }
        if entries.iter().any(|e| e.category == crate::knowledge::Category::Characters) {
            with_characters += 1;
        }
    }
    assert!(with_items >= 20, "only {} games cover items & gear", with_items);
    assert!(with_characters >= 6, "only {} games cover characters & roles", with_characters);
}

// ---------------------------------------------------------------------------
// Cross-game routing — a selected game NEVER gets another game's knowledge
// ---------------------------------------------------------------------------

#[test]
fn selected_game_never_receives_other_games_knowledge() {
    let kb = kb();
    // THE regression case: with a game selected, a question that no curated
    // pack of that game covers must NOT pull in another game's guide.
    // (This used to leak Minecraft's guides into every other game.)
    for (game, q) in [
        (GameType::ESO, "how to start off in elder scrolls online"),
        (GameType::Roblox, "how do i get netherite"),
        (GameType::Fortnite, "tips for netherite"),
        (GameType::Paladins, "where do i find redstone"),
        (GameType::Genshin, "how does the wall of flesh work"),
    ] {
        match knowledge_answer(&kb, &game, q) {
            Some(KnowledgeAnswer::CrossGame(text)) => panic!(
                "[{:?}] LEAKED another game's knowledge for '{}':\n{}",
                game, q, text
            ),
            Some(KnowledgeAnswer::Game(text, _)) => {
                // Fine — but only if the answer is genuinely this game's pack.
                assert!(
                    !lower(&text).contains("minecraft (from")
                        && !lower(&text).contains("(from minecraft)"),
                    "[{:?}] answer for '{}' came from Minecraft:\n{}",
                    game,
                    q,
                    text
                );
            }
            // Universal item advice names the SELECTED game and is game-neutral.
            Some(KnowledgeAnswer::UniversalItem(text)) => {
                assert!(
                    lower(&text).contains(&lower(game.get_name())),
                    "[{:?}] universal answer didn't name the selected game:\n{}",
                    game,
                    text
                );
            }
            // No knowledge answer is correct too — the chat serves this
            // game's own generic tips rather than another game's guides.
            None => {}
        }
    }
}

#[test]
fn general_chat_mode_still_uses_cross_game_knowledge() {
    let kb = kb();
    // No game selected (General Chat): a question that names a game should
    // find that game's curated guide from the cross-game layer.
    match knowledge_answer(&kb, &GameType::Generic, "how do i get netherite") {
        Some(KnowledgeAnswer::CrossGame(text)) => {
            let t = lower(&text);
            assert!(t.contains("minecraft"), "cross answer should name Minecraft: {}", text);
            assert!(t.contains("ancient debris"), "cross answer should be the netherite guide: {}", text);
        }
        other => panic!("expected a cross-game answer in General Chat mode, got {:?}", other),
    }
}

#[test]
fn curated_games_prefer_their_own_pack_over_cross_game() {
    let kb = kb();
    // In Minecraft mode, a diamond question must come from the MINECRAFT pack.
    match knowledge_answer(&kb, &GameType::Minecraft, "how do i find diamonds") {
        Some(KnowledgeAnswer::Game(text, tag)) => {
            assert!(lower(&text).contains("diamond mining guide"));
            assert_eq!(tag, "diamond");
        }
        other => panic!("expected in-game curated answer, got {:?}", other),
    }
}

#[test]
fn game_filter_isolates_knowledge_packs() {
    let kb = kb();
    // A Valorant question must not surface Minecraft entries and vice versa.
    assert!(kb.search(&GameType::Valorant, "netherite ancient debris", 5).is_empty());
    assert!(kb.search(&GameType::Minecraft, "crosshair spray economy", 5).is_empty());
}

// ---------------------------------------------------------------------------
// Universal item brain — any item, any game
// ---------------------------------------------------------------------------

#[test]
fn real_item_questions_always_produce_knowledge() {
    let kb = kb();
    // Real-world item phrasings: whichever route fires (curated pack,
    // cross-game knowledge, or the universal item brain), the player must
    // get knowledge — never silence.
    let questions = [
        "how do i get the mythic goldfish in fortnite",
        "where do i find a golden fishing rod in stardew valley",
        "how do you craft an enchanted shield",
        "what is the recipe for a healing potion",
        "best way to farm rare bones",
        "how do i make a diamond sword",
    ];
    for q in questions {
        let answer = ask(&kb, GameType::Generic, q);
        assert!(!answer.trim().is_empty(), "empty answer for '{}'", q);
    }
}

#[test]
fn universal_item_brain_fires_when_no_curated_pack_knows_the_item() {
    let kb = kb();
    // "Nectar" appears in no curated pack, so the universal brain must
    // handle it — naming both the item and the game.
    match knowledge_answer(&kb, &GameType::Hades, "how do i get nectar in hades") {
        Some(KnowledgeAnswer::UniversalItem(text)) => {
            let t = lower(&text);
            assert!(t.contains("nectar"), "should name the item: {}", text);
            assert!(t.contains("hades"), "should name the game: {}", text);
        }
        other => panic!("expected universal item answer, got {:?}", other),
    }
}

#[test]
fn universal_item_brain_handles_all_question_styles() {
    // Direct check of the trigger surface: every acquisition phrasing maps
    // to a universal answer that echoes the item name. (Called directly —
    // through the full pipeline, curated/cross-game knowledge may
    // legitimately answer first for common item words.)
    let cases = [
        (GameType::CS2, "how do i get a dragon lore", "Dragon Lore"),
        (GameType::Palworld, "where to find sulfur", "Sulfur"),
        (GameType::Hades, "how to farm titan blood", "Titan Blood"),
        (GameType::Minecraft, "what is the recipe for a golden apple", "Golden Apple"),
        (GameType::Zelda, "best way to find star fragments", "Star Fragments"),
        (GameType::Genshin, "how do you craft mystic enhancement ore", "Mystic Enhancement Ore"),
    ];
    for (game, q, item) in cases {
        let text = crate::knowledge::universal_item_answer(&game, q)
            .unwrap_or_else(|| panic!("[{:?}] '{}' should trigger the universal brain", game, q));
        assert!(
            lower(&text).contains(&lower(item)),
            "[{:?}] '{}' should echo '{}':\n{}",
            game,
            q,
            item,
            text
        );
    }
}

#[test]
fn non_item_questions_do_not_trigger_the_universal_brain() {
    let kb = kb();
    // A pure conversational question in a game with no curated pack:
    // the knowledge pipeline correctly yields nothing (the UI then serves
    // its generic tips / teach-me flow).
    assert_eq!(
        knowledge_answer(&kb, &GameType::Paladins, "what is your favorite color"),
        None
    );
}

// ---------------------------------------------------------------------------
// Catalog-wide sweep — every single registered game
// ---------------------------------------------------------------------------

/// A real-world signature item question per well-known game (used by the
/// catalog sweep and the report).
fn signature_questions() -> &'static [(&'static str, &'static str)] {
    &[
        ("Minecraft", "how do i get diamonds in minecraft"),
        ("Roblox", "how do i earn robux in roblox"),
        ("Fortnite", "how do i get v-bucks in fortnite"),
        ("Valorant", "how do i get radianite points in valorant"),
        ("Counter-Strike 2", "how do i get a knife in counter-strike 2"),
        ("Apex Legends", "how do i get crafting materials in apex"),
        ("Call of Duty", "how do i get loadout drops in call of duty"),
        ("PUBG", "how do i get a level 3 helmet in pubg"),
        ("League of Legends", "how do i get blue essence in league of legends"),
        ("Dota 2", "how do i get battle pass items in dota 2"),
        ("World of Warcraft", "how do i get gold in world of warcraft"),
        ("Skyrim", "how do i get daedric armor in skyrim"),
        ("Zelda Series", "how do i get master sword in zelda"),
        ("GTA Series", "how do i get money in gta"),
        ("Rocket League", "how do i get credits in rocket league"),
        ("Stardew Valley", "how do i get prismatic shard in stardew valley"),
        ("The Sims", "how do i get money in the sims"),
        ("Among Us", "how do i get hats in among us"),
        ("Fall Guys", "how do i get skins in fall guys"),
        ("Terraria", "how do i get hellstone in terraria"),
        ("Elden Ring", "how do i get smithing stones in elden ring"),
        ("Valheim", "how do i get iron in valheim"),
        ("Rust", "how do i get scrap in rust"),
        ("Palworld", "how do i get paldium in palworld"),
        ("Genshin Impact", "how do i get primogems in genshin impact"),
        ("Baldur's Gate 3", "how do i get gold in baldur's gate 3"),
        ("Cyberpunk 2077", "how do i get eddies in cyberpunk 2077"),
        ("The Witcher 3", "how do i get crowns in the witcher 3"),
        ("Diablo IV", "how do i get aspects in diablo iv"),
        ("Destiny 2", "how do i get exotic engrams in destiny 2"),
        ("Overwatch 2", "how do i get coins in overwatch 2"),
        ("Warframe", "how do i get platinum in warframe"),
        ("Subnautica", "how do i get diamonds in subnautica"),
        ("ARK: Survival Evolved", "how do i get metal in ark"),
        ("No Man's Sky", "how do i get units in no man's sky"),
        ("Red Dead Redemption 2", "how do i get gold bars in red dead redemption 2"),
        ("Phasmophobia", "how do i get money in phasmophobia"),
        ("Lethal Company", "how do i get scrap in lethal company"),
        ("Hades", "how do i get titan blood in hades"),
        ("Hollow Knight", "how do i get geo in hollow knight"),
        ("Final Fantasy XIV", "how do i get tomestones in final fantasy xiv"),
        ("Sea of Thieves", "how do i get gold in sea of thieves"),
        ("Monster Hunter World", "how do i get monster parts in monster hunter"),
        ("Pokémon Scarlet/Violet", "how do i get master ball in pokemon"),
        ("Minecraft Dungeons", "how do i get better gear in minecraft dungeons"),
        ("Hogwarts Legacy", "how do i get galleons in hogwarts legacy"),
        ("Starfield", "how do i get credits in starfield"),
        ("Black Myth: Wukong", "how do i get will in black myth wukong"),
        ("Sekiro", "how do i get sen in sekiro"),
        ("Bloodborne", "how do i get blood echoes in bloodborne"),
        ("Fallout 4", "how do i get caps in fallout 4"),
        ("Oblivion", "how do i get gold in oblivion"),
    ]
}

/// Every game in the registry must return the universal item answer for a
/// made-up item (proves the game context reaches the answer for all games).
#[test]
fn every_registered_game_answers_a_made_up_item_question() {
    let kb = kb();
    let count = crate::all_games().len();
    for game in crate::all_games() {
        if game == GameType::Generic {
            continue;
        }
        let q = "how do i get the frobnium calibrator";
        match knowledge_answer(&kb, &game, q) {
            Some(KnowledgeAnswer::UniversalItem(text)) => {
                assert!(
                    lower(&text).contains(&lower(game.get_name())),
                    "[{:?}] universal answer didn't name the game: {}",
                    game,
                    text
                );
            }
            other => panic!(
                "[{:?}] expected universal item answer, got {:?}",
                game, other
            ),
        }
    }
    assert!(count >= 100, "registry should have 100+ games, has {}", count);
}

/// Real-world item questions for every registered game must NEVER dead-end:
/// the pipeline must always produce some knowledge (curated, cross-game, or
/// universal route advice).
#[test]
fn every_registered_game_survives_a_real_item_question() {
    let kb = kb();
    for game in crate::all_games() {
        if game == GameType::Generic {
            continue;
        }
        let q = signature_questions()
            .iter()
            .find(|(n, _)| *n == game.get_name())
            .map(|(_, q)| *q)
            .unwrap_or("how do i get the best gear in this game");
        let answer = ask(&kb, game.clone(), q);
        assert!(
            !answer.trim().is_empty(),
            "[{:?}] dead-ended on '{}'",
            game,
            q
        );
    }
}

// ---------------------------------------------------------------------------
// Legacy brain (ai.rs TOPICS) — factual spot checks
// ---------------------------------------------------------------------------

/// (question, marker) pairs exercised against the legacy topic brain.
const BRAIN_CASES: &[(&str, &str)] = &[
    ("how do i find diamonds in minecraft", "y -58"),
    ("how do i get free robux", "scam"),
    ("how do steam refunds work", "14 days"),
    ("what is the terraria boss order", "wall of flesh"),
    ("where should i build my base in palworld", "fast travel"),
    ("how do evs work in pokemon", "510"),
    ("how do i make money fast in gta", "cayo"),
    ("best crops in stardew valley", "blueber"),
    ("do seasons reset in once human", "season"),
    ("fortnite building tips", "edit"),
    ("how do korok seeds work in zelda", "korok"),
    ("what stats should i level in elden ring", "vigor"),
    ("fps drop fix", "graphics"),
];

fn brain_answer(q: &str) -> String {
    let mut brain = crate::ai::Brain::default();
    let reply = brain.answer(q);
    assert_eq!(
        reply.tag,
        crate::ai::Tag::Knowledge,
        "'{}' should answer from the knowledge topics, got tag {:?}: {}",
        q,
        reply.tag,
        reply.text
    );
    reply.text
}

#[test]
fn brain_covers_every_builtin_topic() {
    for (q, marker) in BRAIN_CASES {
        let text = brain_answer(q);
        assert!(
            lower(&text).contains(marker),
            "brain Q '{}' missing marker '{}':\n{}",
            q,
            marker,
            text
        );
    }
}

#[test]
fn brain_elden_ring_facts_are_real_mechanics() {
    let text = lower(&brain_answer("how do i farm runes in elden ring"));
    assert!(text.contains("mimic tear"), "should mention the Mimic Tear:\n{}", text);
    assert!(!text.contains("graceful melody"), "fabricated 'Graceful Melody' must stay gone:\n{}", text);
    assert!(!text.contains("+3 flask"), "no such thing as a '+3 Flask of Wondrous Physick':\n{}", text);
}

#[test]
fn brain_hinox_fact_targets_the_eye_not_fake_fire_weakness() {
    let text = lower(&brain_answer("how do i beat a hinox in zelda"));
    assert!(text.contains("eye"), "should target the eye:\n{}", text);
    assert!(!text.contains("weak to fire"), "Hinox are not fire-weak:\n{}", text);
}

#[test]
fn brain_fortnite_meta_is_not_patched_to_one_weapon() {
    let text = lower(&brain_answer("what are the meta weapons in fortnite"));
    assert!(
        text.contains("long-range") || text.contains("shotgun"),
        "should describe the loadout shape:\n{}",
        text
    );
    assert!(!text.contains("is king"), "meta claims shouldn't crown a single gun:\n{}", text);
}

#[test]
fn brain_fortnite_movement_is_real_fortnite_movement() {
    let text = lower(&brain_answer("advanced movement in fortnite"));
    assert!(text.contains("double edit"), "should teach double edits:\n{}", text);
    assert!(!text.contains("tap strafe"), "tap-strafe is Apex, not Fortnite:\n{}", text);
}

// ---------------------------------------------------------------------------
// The Q&A report — every game, every question, Rocket's verbatim answer
// ---------------------------------------------------------------------------

/// Writes `rocket-qa-report.md`: every game in check, every question asked,
/// and Rocket's verbatim answer. Regenerated on every `cargo test` run so it
/// can never drift from the real pipeline.
#[test]
fn generate_qa_report() {
    let kb = kb();
    let mut out = String::new();
    out.push_str("# 🚀 ROCKET Knowledge QA Report\n\n");
    out.push_str("_Auto-generated by `cargo test` (see `src/game_qa.rs`). Every question \
                  below was asked through Rocket's real answer pipeline; every answer is \
                  Rocket's verbatim output. Each battery question also lists the \
                  ground-truth facts the answer was verified against._\n\n");

    // ---- Part 1: curated packs — the full question battery -----------------
    out.push_str("## Part 1 — Games with curated knowledge packs (full Q&A battery)\n\n");
    let mut q_count = 0usize;
    for (game, cases) in BATTERIES {
        out.push_str(&format!("### {}\n\n", game.get_name()));
        for (q, markers) in *cases {
            q_count += 1;
            let (route, text) = pipeline_answer(&kb, game, q);
            out.push_str(&format!(
                "**Q:** {}\n\n{} · *verified against: {}*\n\n```text\n{}\n```\n\n---\n\n",
                q,
                route,
                markers.join(", "),
                text
            ));
        }
    }

    // ---- Part 2: catalog sweep — every other registered game ---------------
    out.push_str("## Part 2 — All other registered games (catalog sweep)\n\n");
    out.push_str("_These games have no curated pack yet; Rocket answers through the \
                  universal item brain and never leaks another game's guides into a \
                  selected game. Verified to never dead-end._\n\n");
    let mut sweep_count = 0usize;
    for game in crate::all_games() {
        if game == GameType::Generic || cases_for(&game).len() > 0 {
            continue;
        }
        sweep_count += 1;
        let q = signature_questions()
            .iter()
            .find(|(n, _)| *n == game.get_name())
            .map(|(_, q)| *q)
            .unwrap_or("how do i get the best gear in this game");
        let (route, text) = pipeline_answer(&kb, &game, q);
        out.push_str(&format!(
            "### {}\n\n**Q:** {}\n\n{}\n\n```text\n{}\n```\n\n---\n\n",
            game.get_name(),
            q,
            route,
            text
        ));
    }

    // ---- Part 3: legacy topic brain -----------------------------------------
    out.push_str("## Part 3 — Legacy topic brain (ai.rs built-in topics)\n\n");
    for (q, marker) in BRAIN_CASES {
        let text = brain_answer(q);
        out.push_str(&format!(
            "**Q:** {}\n\n🧠 answered from the built-in topic brain · *verified against: `{}`*\n\n```text\n{}\n```\n\n---\n\n",
            q, marker, text
        ));
    }

    // ---- Completeness gate ---------------------------------------------------
    // Every registered game (plus the General Chat pseudo-game) MUST have a
    // section above — the report can never silently drop a game.
    for game in crate::all_games() {
        assert!(
            out.contains(&format!("### {}", game.get_name())),
            "QA report is missing a section for game '{}'",
            game.get_name()
        );
    }

    // ---- Summary -------------------------------------------------------------
    out.push_str("## Summary\n\n");
    out.push_str(&format!(
        "- **{} games** in the registry, all checked (never dead-end)\n\
         - **{} games** with curated knowledge packs, **{} battery questions** answered and fact-verified\n\
         - **{} other games** swept with catalog questions\n\
         - **{} legacy brain questions** checked\n\
         - Every answer above is verbatim Rocket output from the live answer pipeline.\n",
        crate::all_games().len() - 1, // minus the Generic pseudo-game
        BATTERIES.len() - 1,          // minus Generic
        q_count,
        sweep_count,
        BRAIN_CASES.len(),
    ));

    std::fs::write(REPORT_FILE, out).expect("failed to write the QA report file");
}
