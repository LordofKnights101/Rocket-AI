//! ROCKET's gaming knowledge base — structured, searchable game data.
//!
//! Each entry belongs to one game and one category, carries keyword tags,
//! and is scored by a lightweight search so both the Knowledge Base panel
//! and the local AI can ground their answers in curated content.

use crate::{GameFilter, GameType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Strategy,
    Mechanics,
    Items,
    Characters,
    Progression,
    Glossary,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Strategy => "🎯 Strategy",
            Category::Mechanics => "⚙️ Mechanics",
            Category::Items => "🗡️ Items & Gear",
            Category::Characters => "👥 Characters & Roles",
            Category::Progression => "📈 Progression",
            Category::Glossary => "📖 Glossary",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub category: Category,
    pub title: &'static str,
    pub content: &'static str,
    pub tags: &'static [&'static str],
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeBase {
    entries: Vec<(GameType, KnowledgeEntry)>,
}

fn e(category: Category, title: &'static str, content: &'static str, tags: &'static [&'static str]) -> KnowledgeEntry {
    KnowledgeEntry { category, title, content, tags }
}

/// Words ignored by the search scorer.
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "do", "does", "did", "how", "what", "when", "where", "why",
    "i", "my", "me", "you", "your", "to", "for", "of", "in", "on", "with", "and", "or", "can",
    "should", "it", "its", "be", "get", "got", "best", "good", "about",
];

/// Substring OR stem-ish match: "netherite" hits tag "nether",
/// "diamonds" hits tag "diamond", etc. Prefix matching needs both
/// sides to be at least 4 chars to avoid junk matches.
fn matches_word(hay: &str, word: &str) -> bool {
    if hay.contains(word) {
        return true;
    }
    if word.len() >= 4 && hay.len() >= 4 {
        return word.starts_with(hay) || hay.starts_with(word);
    }
    false
}

impl KnowledgeBase {
    pub fn new() -> Self {
        let mut kb = KnowledgeBase { entries: Vec::new() };

        // ------------------------------------------------------------- Minecraft
        [
            e(Category::Items, "Diamond Mining Guide",
                "• Best levels: Y -58 to -53 (deepslate layer, Y -58 is optimal).\n• Use an iron pickaxe or better — stone won't drop diamonds.\n• Branch mine in straight tunnels 2 blocks apart, or explore deepslate caves.\n• Bring torches, food, a water bucket (lava pockets are everywhere) and never dig straight down.\n• Fortune III pickaxe can double or triple your diamond yield.",
                &["diamond", "mining", "ore", "pickaxe", "y-level", "branch"]),
            e(Category::Mechanics, "Nether Survival",
                "• Wear at least one gold armor piece — piglins turn hostile without it.\n• Nether Fortresses hold Blaze Rods (essential for potions and the End).\n• Watch for ghasts — their fireballs can be punched back.\n• Carry fire resistance potions before long lava crossings.\n• Build your portal in a walled-off cobblestone shelter to stay safe.",
                &["nether", "piglin", "blaze", "fortress", "portal", "ghast", "lava"]),
            e(Category::Strategy, "The End & Ender Dragon Fight",
                "• Craft Eyes of Ender to locate the stronghold portal (needs Blaze Powder + Ender Pearl).\n• Endermen: never look above their legs — wear a carved pumpkin for safe staring.\n• Beds explode in the End — they're a classic high-damage weapon against the dragon, but bring plenty.\n• Destroy the End Crystals on the obsidian pillars FIRST (shoot the caged ones), or the dragon keeps healing.\n• Bring slow falling potions and stacked Ender Pearls for the void.",
                &["end", "ender dragon", "pearl", "crystal", "elytra", "dragon"]),
            e(Category::Mechanics, "Redstone Basics",
                "• Power sources: lever, button, pressure plate, redstone torch, observer.\n• Repeaters extend signal 15 blocks and add delay; comparators read container fill levels.\n• Observers detect block updates — the core of most automatic farms.\n• Start simple: automatic doors, item sorters, then sugarcane/pumpkin farms before complex contraptions.\n• Keep circuits tidy — run redstone dust in dedicated channels under your builds.",
                &["redstone", "circuit", "farm", "observer", "repeater", "comparator"]),
            e(Category::Progression, "Early Game Progression Route",
                "• Wood → wooden tools → stone tools → find iron in caves or mountains.\n• Smelt iron, make full armor + shield before venturing deep.\n• Loot a village for beds, food, and blacksmith chest loot.\n• Build a simple shelter and a wheat/cow food farm before your first night.\n• Get a bed ASAP so you can skip night and set your spawn point.",
                &["survival", "start", "progression", "tools", "iron", "village", "night"]),
            e(Category::Items, "Netherite & Ancient Debris Guide",
                "• Netherite comes from Ancient Debris — Nether-only ore, most common around Y=15 (range 8-22).\n• Ancient Debris is blast-proof: the fastest method is bed mining — tunnel at Y=15, place a bed, poke it, step back (beds explode in the Nether).\n• Alternatively strip-mine with a diamond pickaxe through netherrack tunnels.\n• Smelt Ancient Debris → Netherite Scrap; craft 4 scrap + 4 gold ingots → 1 Netherite Ingot.\n• Upgrade diamond gear at a smithing table using a Netherite Upgrade Smithing Template (bastion treasure rooms; duplicate templates with 7 diamonds + 1 netherrack block).\n• Netherite gear keeps all enchantments, floats in lava, and adds knockback resistance.",
                &["netherite", "ancient debris", "debris", "bastion", "smithing", "ingot", "upgrade", "gear"]),
            e(Category::Items, "Enchanting & XP",
                "• Enchanting table surrounded by 15 bookshelves unlocks level-30 enchants.\n• Lapis lazuli is required for every enchant.\n• Cheap path to god gear: enchant books, then combine on an anvil.\n• Mending makes tools repair with XP — the single most valuable enchant.\n• Fish with a Luck of the Sea rod or breed villagers for cheap enchanted books.",
                &["enchant", "xp", "anvil", "mending", "bookshelf", "lapis"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Minecraft, entry));

        // -------------------------------------------------------------- Fortnite
        [
            e(Category::Mechanics, "Building Fundamentals",
                "• Learn the 90s: ramp + wall + jump + floor — the fastest way to gain height.\n• Box fighting: wall yourself, listen for edits, and counter with your own edit plays.\n• Always build a ramp behind you when pushing — never peek the same angle twice.\n• Tunnel with floor-wall-ramp when crossing open ground under fire.\n• In Creative mode, drill 1v1 box fights and edit courses for 15 minutes daily.",
                &["building", "90s", "ramp", "edit", "box", "tunnel"]),
            e(Category::Strategy, "Landing & Loot Priority",
                "• Hot drops teach fights but cost rating; land one town over and rotate in for safer starts.\n• Loot priority: any shield → any weapon → heals → ammo.\n• Open chests by sound — the golden glow beam marks them through walls.\n• Grab a mobility item early (sprint, grapple, vehicle) for fast zone rotations.\n• Collect materials from every tree and wall you pass — you'll need them mid-game.",
                &["landing", "loot", "drop", "shield", "chest", "rotation"]),
            e(Category::Items, "Loadout Guide",
                "• Ideal 5 slots: AR (mid-range), shotgun (close), SMG or sniper (flex), heals x2.\n• Keep ammo types covered — an empty legendary is worse than a common you can feed.\n• Shield potions: smalls are stackable and fast; chug bigs between fights.\n• Replace gray/green weapons the moment you find blue or better.\n• Carry one utility (shockwave, grenades) for third-party escapes.",
                &["loadout", "weapons", "meta", "shield potion", "ammo", "heals"]),
            e(Category::Strategy, "Endgame Strategy",
                "• Rotate early to zone center — late rotations fight everyone.\n• Take high ground and hold it, but build a floor under you (people ramp from below).\n• Track third parties: after every fight, heal and reposition before pushing loot.\n• In storm surge, damage matters more than survival — force fights.\n• Final circle: build low, edit out, and let the other players fight first.",
                &["endgame", "zone", "rotation", "third party", "storm", "high ground"]),
            e(Category::Strategy, "Zero Build Mode Tips",
                "• Cover is everything — fight near rocks, buildings, and hills you can break line of sight with.\n• Sprint + mantle constantly; a moving target is hard to hit.\n• Sliding down hills while shooting wins most open-field duels.\n• Save a mobility item to disengage from losing fights — resets are cheap.\n• snipers and ARs dominate; always carry a long-range option.",
                &["zero build", "movement", "cover", "sprint", "sniper"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Fortnite, entry));

        // ----------------------------------------------------- League of Legends
        [
            e(Category::Characters, "Role Fundamentals",
                "• Top: island lane — splitpush, side-lane pressure, TP plays.\n• Jungle: map control — efficient clear, ganks, objectives, vision.\n• Mid: roam and flex — push wave then influence side lanes.\n• ADC (bot): scale up — farm safely, group for teamfights, stay behind your frontline.\n• Support: enable — peel, vision, engage/disengage, roam with your jungler.",
                &["role", "lane", "jungle", "top", "mid", "adc", "support"]),
            e(Category::Mechanics, "Farming & Wave Management",
                "• CS benchmarks: 7-8/min is good, 9-10/min is excellent in solo queue.\n• Last-hit with autos; use spells only when the wave is big or you're being zoned.\n• Freeze: tank the wave just outside your tower to starve the enemy and stay safe.\n• Slow push: kill caster minions to build a big wave before an objective.\n• Crash: shove fully into tower before roaming or backing so you lose nothing.",
                &["farm", "cs", "wave", "gold", "minion", "freeze", "push"]),
            e(Category::Strategy, "Vision Control",
                "• Buy a control ward every back — park it in river bush or objective pit.\n• Sweep before objectives; deny vision BEFORE the enemy can ward it.\n• Deep ward at 1:15 with your jungler for early-game safety.\n• Vision wins fights: never face-check when your sweeper shows nothing — walk wide.\n• Ward for the next play, not the last one: objective timers > lane bushes after laning.",
                &["ward", "vision", "map", "control ward", "sweeper"]),
            e(Category::Strategy, "Objective Priority",
                "• General order: Baron > Elder Dragon > Soul point > 4th drake > Herald > towers.\n• Trade objectives: if the enemy starts Baron, you take a tower or drake — never fight without vision.\n• Herald is a tower-removal tool: use it to crack mid before 20 minutes.\n• Never start an objective without: vision control, numbers advantage, or a clear escape plan.\n• Ping timers — a team that knows when drake spawns rotates on time.",
                &["dragon", "baron", "herald", "objective", "elder", "tower"]),
            e(Category::Mechanics, "Trading in Lane",
                "• Trade when the enemy goes for a last-hit — they're locked in their auto animation.\n• Track enemy cooldowns: if their key spell is down, that's your all-in window.\n• Minion aggro hurts early — don't trade inside a big enemy wave at level 1-3.\n• Poke, then commit: two good trades set up a kill; one random all-in usually throws.\n• Back before a cannon wave crashes to avoid missing CS.",
                &["trade", "lane", "poke", "cooldown", "all-in", "aggro"]),
            e(Category::Strategy, "Ranked Climb Tips",
                "• One-trick or two-trick a small champion pool — mastery beats counter-picking below Diamond.\n• Review every death: ask 'what information did I ignore?' instead of blaming teammates.\n• Mute trolls instantly — mental is half of climbing.\n• Play for macro: the team that groups first with numbers usually wins mid-game.\n• Stop after 2 losses in a row; tilt-queuing costs more LP than the break costs time.",
                &["ranked", "climb", "tilt", "lp", "champion pool", "macro"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::LeagueOfLegends, entry));

        // --------------------------------------------------------------- Valorant
        [
            e(Category::Mechanics, "Crosshair Placement",
                "• Keep your crosshair at head height at ALL times — even while rotating.\n• Pre-aim common angles as you walk; your crosshair should already be where an enemy appears.\n• Slice the pie: clear angles one at a time, never wide-swing two at once.\n• Don't stare at the floor when moving — this is the #1 aim mistake.\n• Deathmatch 10 minutes before ranked with placement focus, not flick focus.",
                &["crosshair", "aim", "head", "placement", "angle", "deathmatch"]),
            e(Category::Characters, "Agent Roles",
                "• Duelist (Jett, Reyna, Phoenix): entry fragger — first through the door.\n• Controller (Brimstone, Omen, Astra): smokes — cut off sightlines for executes.\n• Initiator (Sova, Breach, Fade): info + flashes — opens sites before the duelists enter.\n• Sentinel (Sage, Killjoy, Cypher): anchors a site and locks down flanks.\n• Team comp rule: at least one of each role; a team of duelists loses to utility.",
                &["agent", "duelist", "controller", "initiator", "sentinel", "comp", "smoke"]),
            e(Category::Mechanics, "Economy & Buying",
                "• Full buy: rifle + armor + full utility (~3900+ credits).\n• Eco (save) when you can't afford rifle + armor: buy a pistol and save everything.\n• Force buy only when losing the round loses the half, or you broke the enemy economy.\n• Lost a round with a full buy? Next round is likely a save — don't tilt-force.\n• Share: drop a teammate a weapon instead of half-buying as a duo.",
                &["eco", "buy", "economy", "credits", "force", "save", "armor"]),
            e(Category::Strategy, "Site Executes",
                "• Order of operations: clear info → smoke the dangerous angles → flash → enter.\n• Never enter a site on the same angle your smoke just landed — wait 1s for peeks.\n• Post-plant: crossfire positions > heroic 1vX peeks. Play time, not kills.\n• Default setups: two lurk/hold, three execute. Don't all five stack one entrance.\n• Learn one agent's lineups per map before queuing it in ranked.",
                &["execute", "site", "plant", "post-plant", "lurk", "entry"]),
            e(Category::Glossary, "Common Callouts",
                "• Callouts are short map names — learn your main map's callouts before ranked (review a map image between games).\n• Standard comms: location + number + health: 'A site, two, one shot'.\n• 'Flank' means someone is behind you — call it the moment you hear steps.\n• 'Default' = spread out and gather info before committing to a site.\n• Keep comms short: dead players give info, not criticism.",
                &["callout", "comms", "map", "flank", "default", "info"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Valorant, entry));

        // ---------------------------------------------------- Counter-Strike 2
        [
            e(Category::Mechanics, "Economy System",
                "• Rounds: eco (save) → force → full buy. Track the enemy loss bonus streak too.\n• Loss bonus grows each consecutive loss — sometimes losing 'correctly' sets up a big buy.\n• Save your rifle in a lost round (run to a corner, hide the gun) — 2700 credits kept.\n• CT side: armor + defuse kit is mandatory on full buys.\n• Don't force every pistol-round loss; a clean eco loses less over 5 rounds.",
                &["economy", "eco", "buy", "money", "loss bonus", "save", "kit"]),
            e(Category::Mechanics, "Aim & Spray Control",
                "• First bullet is nearly laser-accurate standing still — stop before you shoot.\n• Spray pattern: straight down for ~10 bullets, then pull left, then right. Practice on an empty map daily.\n• Tap or burst (2-3 bullets) beyond close range; spray only inside 10-15m.\n• Crouch-shooting tightens spray but makes you a slower target — use it for long duels only.\n• Crosshair placement at head level halves your required flick distance.",
                &["aim", "spray", "recoil", "pattern", "tap", "burst", "crosshair"]),
            e(Category::Strategy, "Utility Usage",
                "• Smokes buy time and block chokes — throw them BEFORE your team is in the open.\n• Molotov the common close angles (corners, ninja spots) before entering.\n• Pop-flashes for entries: teammate throws around a corner so enemies are blind when you peek.\n• HE grenades finish low-HP players hiding behind cover.\n• Utility wins rounds: a site take with 4 smokes > a site take with 4 kills.",
                &["smoke", "flash", "molotov", "grenade", "he", "utility", "lineup"]),
            e(Category::Strategy, "Map Positioning",
                "• Isolate angles: position so you only have to watch one spot at a time.\n• Set up crossfires with a teammate — two overlapping angles beat any entry.\n• Off-angles: stand slightly off from the common pre-aim spot — you peek first.\n• Retakes: group up, use utility on the planted bomb, enter together — solo retakes lose.\n• After winning an entry, swap roles: the entry-fragger trades, the trader refrags.",
                &["position", "angle", "crossfire", "retake", "off-angle", "trade"]),
            e(Category::Strategy, "Pistol Round Strategy",
                "• T side: armor + 5-7 or full utility spread. CT side: armor or nades + kit decision.\n• Play for picks and info, not brawls — one pick converts to a numbers advantage.\n• Stack or spread: 3-1-1 CT setups cover the two most common T hits.\n• Don't force second-duel heroics after winning the first pick — reset and group.\n• Winning pistol round usually wins the next 2 — play it slow and clean.",
                &["pistol", "round", "armor", "pick", "setup"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::CS2, entry));

        // ------------------------------------------------------------ Apex Legends
        [
            e(Category::Characters, "Legend Synergy",
                "• Build around a role: fragger (Wraith, Octane), support (Gibby, Lifeline), recon (Bloodhound, Valk).\n• Classic combo: Gibby dome + Wraith portal = safe revive or aggressive flank.\n• Bloodhound scan → team push is the most reliable engage in the game.\n• One Support legend per team for the craft banners / extra loot perk.\n• Counter-pick awareness: Catalyst walls and Maggie's traps zone-lock mobility legends.",
                &["legend", "comp", "synergy", "gibby", "wraith", "bloodhound", "scan"]),
            e(Category::Items, "Loot Priorities",
                "• Armor first: purple/blue armor > any weapon upgrade.\n• Crafting materials: save for a gold/red armor or your main weapon attachments.\n• Ammo and meds matter more than a third weapon — two guns is plenty.\n• Hop-ups (like Hammerpoint, Double Tap) change weapon tier power dramatically.\n• Vault keys and MRVNs give high-tier loot — contest them early, leave fast.",
                &["loot", "armor", "crafting", "materials", "hop-up", "vault", "attachments"]),
            e(Category::Strategy, "Rotations & Zone Play",
                "• Rotate early — one zone ahead of the pack. Late rotations get third-partied.\n• Use map beacons and the ring map (Tab) to plan your next move before the ring closes.\n• Hold high ground near zone center: easiest rotations and free damage on movers.\n• Vehicles for long rotations; drop them before the final circles (noise + no cover).\n• Never fight on the ring's edge with your back to the storm.",
                &["zone", "rotation", "ring", "high ground", "vehicle", "beacon"]),
            e(Category::Strategy, "Team Fighting",
                "• Focus fire: all three players damage one target — cracked shields get finished.\n• Knock, THEN push: a knock turns a 3v3 into a 3v2 with a free revive threat.\n• Use cover between damage — peak, shoot, reposition, repeat.\n• Gibby/Lifeline revive behind the dome; the third player watches the angle.\n• Call out shield status and knocked players — comms win more fights than aim.",
                &["fight", "push", "knock", "crack", "shield", "revive", "focus"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::ApexLegends, entry));

        // --------------------------------------------------------------- Call of Duty
        [
            e(Category::Items, "Gunsmith Basics",
                "• Every attachment is a tradeoff: recoil control vs ADS speed vs mobility.\n• AR builds: muzzle (recoil) + underbarrel (recoil) + optic you can actually use.\n• SMG builds: prioritize ADS speed and sprint-to-fire — you win by snapping onto targets first.\n• Test in a private match or vs AI before ranked — the numbers tab shows the tradeoffs.\n• Meta shifts with patches: check the in-game featured builds for current strong options.",
                &["gunsmith", "attachment", "build", "recoil", "ads", "smg", "ar"]),
            e(Category::Mechanics, "Movement & Slide",
                "• Tactical sprint constantly — you're a harder target and rotate faster.\n• Slide cancel keeps momentum while resetting your stance before a gunfight.\n• Jump-peek corners: the enemy pre-aims head level, so jumping wins the first shots.\n• Never stand still in a duel — strafe while shooting.\n• Learn 2-3 slide paths on your main map for unexpected angles.",
                &["movement", "slide", "tac sprint", "jump", "strafe", "peek"]),
            e(Category::Strategy, "Objective Modes",
                "• Hardpoint/Domination: rotation timing beats kills — be at the next point BEFORE it activates.\n• Play life over kills: a 1.0 K/D holding the objective outperforms a 3.0 K/D wandering the map.\n• Use killstreaks to break spawns and lock down the objective area, not for lone-wolf flanks.\n• UAV first: information wins respawns modes.\n• Anchor spawn: one player holding the back point keeps enemy spawns predictable.",
                &["objective", "hardpoint", "domination", "spawn", "uav", "streak"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::CallOfDuty, entry));

        // -------------------------------------------------------------------- PUBG
        [
            e(Category::Strategy, "Drop Selection",
                "• Read the flight path before jumping: plan a drop with loot AND a safe rotation.\n• Hot drops (school, Pochinki, Novo) give fast fights; edge drops give consistent top-10s.\n• Land roof-first on compounds: instant loot + high ground over interior landers.\n• Watch where other squads drop — their rotation will meet yours mid-map.\n• Consistency tip: pick 3 drops and master their loot routes.",
                &["drop", "landing", "flight", "hot", "compound", "route"]),
            e(Category::Items, "Loot & Loadout",
                "• Standard loadout: two ARs (one 5.56, one 7.62) or AR + DMR/SPAS for close range.\n• Level 3 helmet + vest priority: they literally win headshot trades.\n• Carry 4-6 meds (first aid/med kit), 2-3 smokes, 2 frags minimum late game.\n• 8x scopes go on DMRs/SRs; ARs cap at 6x effectively.\n• Gas cans and pans aren't memes — gas can = zone mobility, pan = headshot block.",
                &["loot", "loadout", "armor", "helmet", "scope", "meds", "smoke"]),
            e(Category::Strategy, "Zone Play",
                "• Rotate at the first blue move — the early zone is about position, not kills.\n• Vehicles: grab fuel, park facing your escape direction, ditch before the final 3 zones.\n• Edge vs center: center of circle = flexible; edge = fewer angles but risky shifts.\n• Final circles: smoke your repositioning, never run in the open.\n• Listen: footsteps in buildings and gunfire direction is free info every match.",
                &["zone", "blue", "rotate", "vehicle", "smoke", "circle", "footsteps"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::PUBG, entry));

        // ------------------------------------------------------------ Rocket League
        [
            e(Category::Strategy, "Rotation Basics",
                "• Rotate back-post: after a challenge, loop through your own net side, not through the play.\n• One player challenges, one supports (mid-boost), one waits back. Never two challenges on the same ball.\n• Don't ball-chase: if a teammate is closer, rotate out and let them take it.\n• Grab boost on your rotation path — a rotating player with 60+ boost is a threat.\n• In 2s, one up one back is the permanent rule; double-commits lose games.",
                &["rotation", "positioning", "back post", "challenge", "double commit"]),
            e(Category::Mechanics, "Fundamental Mechanics",
                "• Powerslide turns: hold slide through sharp turns to keep speed in rotations.\n• Fast aerial: jump, tilt back, double-jump, boost — the core of all aerial play.\n• Wavedash and flip-cancel come later; first master hitting the ball WHERE you aim.\n• Ball cam on 90% of the time; toggle to car cam only for boost grabs.\n• Shadow defense: mirror the attacker's movement from between them and your net.",
                &["aerial", "powerslide", "mechanics", "ball cam", "flip", "shadow"]),
            e(Category::Mechanics, "Boost Management",
                "• Small pads give +12 — learn the pad lines on your main maps and you'll rarely hit zero.\n• Don't boost while the ball is directly overhead waiting to drop — you'll whiff and be empty.\n• Keep 30+ boost minimum when your team is attacking; you can't rotate or defend on fumes.\n• Big boost pads (100) sit at the corners and midfield — steal the opponent's corners when ahead.\n• Boost-starving the enemy (taking their corner pads) is a real strategy in 1s.",
                &["boost", "pads", "corner", "aerial", "economy"]),
            e(Category::Progression, "Training Routine",
                "• 10 minutes of free play before ranked: ball control, powerslide shots, speed flips.\n• Custom training packs: 20 shots on saves + 20 on shooting daily beats 2 hours of games.\n• Aerial drills: rings maps (via workshops) build air control faster than matches.\n• Review one replay per session — watch YOUR car, not the ball.\n• Rank-specific focus: Gold = hitting the ball on target, Plat = powershots, Diamond = fast aerials + rotations.",
                &["training", "freeplay", "practice", "custom", "replay", "rank"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::RocketLeague, entry));

        // ----------------------------------------------------------------- The Sims
        [
            e(Category::Progression, "Money Making",
                "• Careers give steady income; skills give the real money (painting, writing, guitar royalties).\n• Level painting → sell canvases daily; masterpieces sell for thousands.\n• Writing: publish books for weekly royalties that stack.\n• Gardening is passive money: high-value plants (orchids, dragonsfruit) + fertilizer loops.\n• Traits matter: Loner/Fortune aspirations boost the solo money game.",
                &["money", "career", "skill", "painting", "writing", "gardening"]),
            e(Category::Mechanics, "Needs Management",
                "• Fill needs BEFORE work/school — a red Energy need tanks performance.\n• Quick meals vs gourmet: cook quality when Bladder+Hunger are both safe.\n• Sleep in shifts for big households; espresso machine is a mid-game energy patch.\n• Cheerful/moodlet stacking: nice decor + completed aspirations push 'Very' emotions for skill gains.\n• Freeze needs cheat exists (needs.disable_decay) but breaks aspiration scoring.",
                &["needs", "motive", "energy", "sleep", "mood", "emotion"]),
            e(Category::Strategy, "Build Mode Tips",
                "• bb.moveobjects ON lets you overlap objects precisely — the key to detailed builds.\n• Start every build with a floor plan (real floor plans are great templates).\n• Room height + tall windows make cheap builds look expensive.\n• Use railings and half-walls to fake open-plan architecture.\n• Landscaping sells the build: terrain paint + varied plant sizes beats flat lawns.",
                &["build", "house", "moveobjects", "architecture", "landscaping"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::TheSims, entry));

        // ------------------------------------------------------------ Stardew Valley
        [
            e(Category::Progression, "Crop Seasons",
                "• Spring: parsnips (year 1 start) → potatoes → cauliflower; strawberries at the Egg Festival are the best investment.\n• Summer: blueberries (regrow, 3 per harvest) are the money crop; starfruit if you have desert access.\n• Fall: cranberries (regrow) or pumpkin for pure profit.\n• Sprinklers free your energy: quality sprinklers at farming level 6, iridium at level 9.\n• Save 15+ crops for the Community Center bundles BEFORE selling everything.",
                &["crop", "season", "farm", "sprinkler", "blueberry", "community center"]),
            e(Category::Mechanics, "Energy Management",
                "• Passing out costs money and items — never mine past 1:50 AM.\n• Food = energy: field snacks early (forage items), cheese/salads once money flows.\n• Plan your day: one big activity (mines OR fishing OR farm clear) per day beats half-finished everything.\n• Upgraded tools (pickaxe/axe) save energy long-term — prioritize pickaxe first.\n• Sleep by 10 PM for full energy and no penalty.",
                &["energy", "stamina", "food", "pass out", "tool", "sleep"]),
            e(Category::Strategy, "The Mines",
                "• Floors 1-40: copper. 41-80: iron. 81-120: gold. Skull Cavern for iridium (bring bombs + staircases).\n• Kill enemies and break rocks for ladder spawns — big rock clusters often hide ladders.\n• Luck days (check the TV) = better ladders and ore — save deep runs for them.\n• Bring food, bombs (recipe from Dwarf), and a weapon upgrade before floor 40.\n• Elevator checkpoints save your progress every 5 floors.",
                &["mine", "ore", "ladder", "skull cavern", "iridium", "bomb", "luck"]),
            e(Category::Progression, "Friendship & Villagers",
                "• Two liked gifts per villager per week; loved gifts give way more points.\n• Universal loves: prismatic shard, rabbit's foot, pearl.\n• Birthdays: loved gift on a birthday = ~2.5x points — prioritize these.\n• Check the calendar (Pierre's/store) and wiki the heart events you're near.\n• Marriage candidates unlock useful spouse help (watering, feeding) at 12+ hearts.",
                &["gift", "friendship", "villager", "birthday", "marriage", "hearts"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::StardewValley, entry));

        // -------------------------------------------------------------- Zelda Series
        [
            e(Category::Mechanics, "Combat Basics (BotW/TotK)",
                "• Flurry rush: dodge back (or sideways vs sweeps) at the last moment for slow-mo counterattacks.\n• Perfect guard (parry) with ZL timing deflects Guardian beams entirely.\n• Weapon durability is a resource — use weak weapons on weak enemies, save the good ones.\n• Headshots stagger; backstabs (sneakstrike) deal 8x damage.\n• Elemental weapons: fire = grass, ice = fire enemies, shock = disarm armored foes.",
                &["combat", "dodge", "flurry", "parry", "guardian", "weapon", "durability"]),
            e(Category::Strategy, "Exploration Priority",
                "• Shrines first: each = spirit orb (hearts/stamina) + fast travel point.\n• Towers unlock map data — grab regional towers before deep exploration.\n• Stamina > hearts early: climbing/gliding reach opens more of the world.\n• Follow roads at night safely; horses auto-follow paths.\n• Mark Korok suspects and unusual rocks on your map for later sweeps.",
                &["shrine", "tower", "explore", "stamina", "korok", "map"]),
            e(Category::Items, "Cooking System",
                "• One buff per dish: mixing buff types wastes the dish (only last effect stays).\n• Five identical buff ingredients = strongest effect (e.g., 5 mighty bananas = attack up lvl 3).\n• Dragon horns (shard) = 30-minute buff duration.\n• Hearty foods overheal to full + extra hearts; endura = extra stamina wheel.\n• Elixirs use critters + monster parts for buffs when food is scarce.",
                &["cook", "food", "buff", "elixir", "hearty", "dragon"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Zelda, entry));

        // ------------------------------------------------------------------- Skyrim
        [
            e(Category::Characters, "Build Archetypes",
                "• Stealth Archer (the classic): Sneak + Archery + Light Armor. Muffled boots, bow from the shadows — one-shot most encounters.\n• Warrior: One-Handed/Two-Handed + Heavy Armor + Smithing. Get a follower to tank while you flank.\n• Mage: Destruction + Conjuration + Alteration. Atronach + dual cast = safe dps.\n• Battlemage: 1H + Destruction + Heavy Armor — the flexible hybrid.\n• Pick 3-4 skills max per build; spreading thin makes mid-game rough.",
                &["build", "class", "archer", "mage", "warrior", "stealth", "armor"]),
            e(Category::Progression, "Skill Leveling",
                "• Crafting loop: Smithing + Alchemy + Enchanting level each other (enchant gear → better potions → stronger enchants).\n• Smithing: make iron daggers/gold rings early; dwarven bows at 30+.\n• Alchemy: giant's toe + wheat + blisterwort = the classic leveling potion.\n• Pay trainers between levels (5 sessions/level) — Companions train combat skills cheaply.\n• skill books and quests that grant free skill-ups are worth planning around.",
                &["skill", "level", "smithing", "alchemy", "enchanting", "trainer", "xp"]),
            e(Category::Mechanics, "Combat Tactics",
                "• Power attacks (hold attack) stagger — stagger-locking enemies is the melee win condition.\n• Shield bash interrupts power attacks and dragon breath windups.\n• Dual-wield: double power attack for burst; blocking is impossible while dual-wielding.\n• Poisons on weapons (paralysis, damage Magicka) trivialize mage enemies.\n• Enemies scale by zone, not level — explore everywhere; the game won't outlevel you.",
                &["combat", "attack", "shield", "bash", "poison", "stagger", "dual"]),
            e(Category::Mechanics, "Shouts & Dragon Fights",
                "• Unrelenting Force (Fus Ro Dah) = crowd control; Whirlwind Sprint = gaps; Become Ethereal = safe repositioning.\n• Dragonrend (main quest) grounds flying dragons — essential at higher levels.\n• Dragons telegraph breath: strafe sideways or get behind terrain, don't backpedal.\n• melee dragons when they land — wait out the wing buffet, then burst.\n• Bring a follower with a bow: dragons hover predictably for ranged damage.",
                &["shout", "dragon", "thu'um", "dragonrend", "breath", "fus"]),
            e(Category::Items, "Economy & Money",
                "• Transmute spell (Halted Stream Camp) turns iron ore → gold ore: jewelry = smithing levels AND money.\n• Enchanted daggers/inventory from crafting loops sell for thousands per trip.\n• Thieves Guild fence + Speech perks remove the 750-1000 gold sell cap.\n• Don't hoard: value/weight ratio matters — sell potions, weapons, and gems first.\n• House purchases: Breezehome (5k) is the cheapest storage + spouse home.",
                &["gold", "money", "sell", "transmute", "fence", "speech", "house"]),
            e(Category::Mechanics, "Clairvoyance & Mage Spells Worth Casting",
                "The mage build staples — what each school actually does for you:\n• Clairvoyance (Novice Alteration): casts a glowing trail that leads you straight to your current quest target — perfect when you're lost, can't find a dungeon entrance, or the compass marker isn't enough. Buy it from court wizards like Farengar in Dragonsreach or at the College of Winterhold.\n• Muffle (Illusion): silences your footsteps for sneaking AND levels Illusion absurdly fast — cast it on repeat in town.\n• Oakflesh/Stoneflesh/Ironflesh: +armor rating while active — early mage survival.\n• Bound Sword/Bow: conjured weapons that level Conjuration, weigh nothing, and can't be disarmed.\n• Candlelight: a floating light that follows you — hands-free dungeon lighting.\n• Conjure Familiar: a free tank that draws aggro while you cast from range.\n• Healing: your only early self-heal — keep it hotkeyed.",
                &["clairvoyance", "clairvoince", "clairvoyence", "spell", "spells", "muffle", "oakflesh", "bound", "candlelight", "alteration", "illusion", "conjuration", "mage", "build", "magic"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Skyrim, entry));

        // --------------------------------------------------------------- GTA Series
        [
            e(Category::Progression, "Money Making Online",
            "• Businesses: MC businesses + Nightclub (passive) beat active grinding long-term.\n• Cayo Perico heist is the best money-per-time — soloable with prep.\n• VIP Work (Headhunter, Sightseer) = fast early money before businesses.\n• Buy a Kosatka early for Cayo access; skip impulse vehicle purchases before you have millions.\n• Double-money weeks rotate weekly — stack sales during bonus events.",
                &["money", "business", "heist", "cayo", "nightclub", "vip", "grind"]),
            e(Category::Mechanics, "Driving & Evasion",
                "• Wanted level: break line of sight, then use alleys/parking garages — cops lose you in multi-level structures.\n• Pay'n'Spray resprays clear wanted stars (unless spotted entering).\n• Off-road + handbrake turns beat straight-line speed in chases.\n• Armor your personal vehicles (armored Kuruma / Nightshark tier) for freemode survival.\n• Helicopters escape 3+ stars fast, but stay below the radar sweep or jets come.",
                &["police", "wanted", "driving", "escape", "chase", "vehicle"]),
            e(Category::Strategy, "Heist & Mission Prep",
                "• Always carry armor + snacks (quick menu restock) — snacks heal mid-mission.\n• Best setup vehicles: armored Kuruma for approach, Insurgent for survival jobs.\n• Heist prep: buy all the support crew items that reduce take cuts.\n• Join missions with a plan: who drives, who shoots, who collects.\n• Passive mode between missions protects your cargo and your cash.",
                &["mission", "heist", "prep", "armor", "snack", "kuruma", "cargo"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::GTA, entry));

        // ----------------------------------------------------------------- Among Us
        [
            e(Category::Strategy, "Crewmate Strategy",
                "• Do tasks in batches and remember WHERE people saw you — alibis win votes.\n• Buddy up on long tasks (download/upload, asteroids) — solo workers are easy kills.\n• Watch task bars: if no bar moves after a 'done' claim, someone lied.\n• Memorize room exits when entering — if a body drops, you can name who was where.\n• Visual tasks (medbay scan, asteroids) clear you instantly — do them when watched.",
                &["crew", "task", "alibi", "body", "visual", "buddy"]),
            e(Category::Strategy, "Impostor Play",
                "• Kill only when at least 2-3 players are out of sight range — never near witnesses.\n• Vent discipline: only vent where exits make sense; vent-killing your own alibi is fatal.\n• Self-report sparingly — once is often genius, twice is a pattern.\n• Fake tasks convincingly: stand at task locations for realistic durations.\n• Play the victim: accuses that land early = free ejected crewmates.",
                &["impostor", "kill", "vent", "report", "fake", "sus"]),
            e(Category::Mechanics, "Discussion & Voting",
                "• Early game: skip votes unless there's hard info — ejecting wrong loses tempo.\n• Ask timeline questions: 'Where were you when lights went out?' — inconsistencies expose impostors.\n• Impostors gang up subtly: one accuses, the other 'reluctantly agrees'.\n• Track vote counts out loud so silent impostors can't hide in the majority.\n• Information asymmetry: share what YOU know, request others do the same before voting.",
                &["vote", "meeting", "sus", "alibi", "skip", "eject"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::AmongUs, entry));

        // ----------------------------------------------------------------- Fall Guys
        [
            e(Category::Strategy, "Race Rounds",
                "• Learn 1-2 shortcuts per map — but only use them with a clear lead; risky paths under pressure = elimination.\n• Stay middle-pack at start: front runners eat the first trap wave.\n• Slime Climb: keep moving, never stop on the moving platforms.\n• Jump timing on spinning bars: jump as the bar is BEHIND the player in front of you.\n• Dive (jump + dive) at finish lines — it extends your reach past a lunging rival.",
                &["race", "shortcut", "slime climb", "jump", "dive", "finish"]),
            e(Category::Strategy, "Final Rounds",
                "• Hex-A-Gone: stay on upper tiles as long as possible; spiral inward and drop late.\n• Hex: don't chase the opponent — let the floor collapse do the work around them.\n• Crown Clash / Fall Mountain: wait for the first grab attempt to whiff, then move in.\n• Jagged/tag finals: patience — the map eliminates greedy players for you.\n• Nerves matter: everyone panics; the calmest player usually grabs the crown.",
                &["final", "crown", "hex", "fall mountain", "grab", "patience"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::FallGuys, entry));

        // -------------------------------------------------------- World of Warcraft
        [
            e(Category::Progression, "Leveling Routes",
                "• Expansion timeline (modern WoW): level through whichever expansion's quests you enjoy — Chromie Time lets you pick.\n• Heirloom gear gives XP bonuses — wear the chest/legs/weapons while leveling alts.\n• Dungeon spam with a tank/heal spec is the fastest queue-supported leveling.\n• Quest in circles: complete a zone's hubs before moving to the next.\n• War Mode ON for the XP bonus if you can handle world PvP.",
                &["level", "quest", "dungeon", "heirloom", "chromie", "war mode", "xp"]),
            e(Category::Characters, "Class Roles",
                "• Tank: leads pulls, holds aggro (AoE abilities), faces enemies away from the group.\n• Healer: keep the tank alive first, DPS dispel/spot-heal second.\n• DPS: follow the kill target, interrupt key casts, avoid avoidable damage.\n• Aggro rule: damage generates threat — DPS who open with burst on fresh pulls pull off the tank.\n• Every class has all three specs somewhere; try one of each role to learn their pain.",
                &["tank", "healer", "dps", "class", "spec", "aggro", "threat"]),
            e(Category::Strategy, "Dungeon Etiquette",
                "• Follow the tank's pace; ask before pulling extra.\n• Interrupt (kick) key enemy casts — a dps interrupting is worth more than 2% extra damage.\n• CC (stuns/polymorphs) on marked targets for hard pulls.\n• Loot rules: need only what your spec uses, greed everything else.\n• If you must leave early, say so — ghosting dungeons gets you reported.",
                &["dungeon", "group", "loot", "interrupt", "cc", "etiquette", "pull"]),
            e(Category::Progression, "Gold Making",
                "• Gathering professions (herbalism/mining/skinning) = steady money with zero investment.\n• Auction house flipping: buy underpriced materials on weekend dumps, relist midweek.\n• Craft consumables (flasks, potions, food) — raid nights create huge demand spikes.\n• TSM/Auctionator addons track deal percentages for you.\n• Long-game: material prices crash at expansion start and recover — stockpile accordingly.",
                &["gold", "auction", "profession", "flipping", "gathering", "consumable"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::WorldOfWarcraft, entry));

        // ------------------------------------------------------------------- Dota 2
        [
            e(Category::Characters, "Lane Roles (Pos 1-5)",
                "• Pos 1 (carry, safe lane): farm priority, scales into the late game.\n• Pos 2 (mid): tempo, runes, ganks, levels.\n• Pos 3 (offlane): space-taker, initiates fights, builds aura/utility items.\n• Pos 4 (soft support): roams, ganks mid, stacks camps, buys smokes/sentries.\n• Pos 5 (hard support): protects the carry, wards, sacrifices farm — the highest-impact low-gold role.",
                &["role", "position", "lane", "carry", "mid", "offlane", "support"]),
            e(Category::Mechanics, "Last Hitting & Denies",
                "• Practice 10-minute last-hit drills in demo mode with no items — 50+ is strong at low MMR.\n• Quelling Blade buys itself: +damage vs creeps for melee last-hitting.\n• Denies: last-hit YOUR creeps — denies the enemy XP/gold and shifts the wave.\n• Creep aggro: right-click the enemy hero then immediately attack-move a creep to pull aggro off yourself.\n• Wave equilibrium: don't mindlessly auto-attack — where the wave meets decides your safety.",
                &["lasthit", "deny", "creep", "aggro", "quelling", "wave", "farm"]),
            e(Category::Strategy, "Warding & Vision",
                "• Observer wards on high ground watch ramps — a ward that sees a ward is a wasted ward (deward first with sentries).\n• Sendries before objectives: a deward pickoff wins the following fight.\n• Ward for the NEXT 7 minutes: where will fights happen (objectives), not where you are now.\n• Smoke past enemy wards — smoked ganks ignore vision entirely.\n• Vision advantage matters most at night — Night Stalker games are won with map-wide vision.",
                &["ward", "vision", "sentry", "deward", "smoke", "night", "ramp"]),
            e(Category::Items, "Item Timing",
                "• BKB (Black King Bar) timing wins mid-game: buy it when enemy spells start killing you, not after.\n• Power spikes: items with immediate impact (Blink, BKB, Radiance timing) create the windows to force fights.\n• Farm patterns: jungle stacks + safe lane waves; never farm the same lane your carry needs.\n• Situational > suggested: the in-game build guide is a starting point, not a script.\n• Sell-and-replace: late game, swap early-game items (wands, bottles) for slot efficiency.",
                &["item", "bkb", "timing", "blink", "farm", "power spike", "slots"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Dota2, entry));

        // ------------------------------------------------------------------- Roblox
        [
            e(Category::Progression, "Roblox Studio Basics",
            "• Start with the toolbox-free basics: insert Parts, learn Move/Scale/Rotate (and the snap-to-grid shortcuts).\n• Anchor every part that shouldn't fall — unanchored builds collapse instantly.\n• Use Models + Folders to organize; name everything (future you will thank you).\n• Publish early and often: File → Publish gives you a live place URL to test on mobile.\n• Template games (Obby/tycoon) in Studio are dissectable learning material.",
                &["studio", "script", "create", "part", "anchor", "publish"]),
            e(Category::Mechanics, "Luau Scripting",
                "• Client vs Server: LocalScripts run on one player's device; Scripts run on the server — never trust the client with game logic.\n• RemoteEvents are the bridge: client fires, server validates.\n• Services: Players, ReplicatedStorage, ServerScriptService are your daily drivers.\n• Learn events first (Touched, PlayerAdded) before attempting complex systems.\n• The Creator Docs have copy-paste samples for every API — use them shamelessly.",
                &["lua", "script", "code", "remote", "client", "server", "event"]),
            e(Category::Progression, "Earning Robux",
                "• Game Passes (one-time perks) + Developer Products (repeat purchases) are the core monetization.\n• Premium Payouts: Premium players playing your game pay you by engagement time.\n• Group games let you split funds and look more professional.\n• Ethical rule: never buy/sell accounts or use free-Robux sites — scams everywhere.\n• Small game + strong retention beats big game + no players — update regularly.",
                &["robux", "pass", "monetize", "premium", "payout", "dev product"]),
            e(Category::Strategy, "Popular Genres",
                "• Obbies: checkpoint pacing, escalating difficulty, decoration sells the difficulty.\n• Tycoons: dropper loop → buttons → upgrades; the addictive core is visible progression.\n• Simulators: stat growth + pets/rebirths; retention hooks matter more than graphics.\n• Horror: lighting (future lighting + fog) does 80% of the scare work.\n• Study the top 10 in your genre weekly — steal STRUCTURE, not assets.",
                &["obby", "tycoon", "genre", "simulator", "horror", "retention"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Roblox, entry));

        // ------------------------------------------------------------- Terraria
        [
            e(Category::Items, "Ore & Bar Progression Guide",
                "• Pre-boss tiers: copper/tin → iron/lead → silver/tungsten → gold/platinum, all underground.\n• Hellstone (Underworld) → Molten gear — mine it with an Obsidian Skull + Nightmare/Deathbringer pickaxe.\n• Hardmode ores spawn in order: Cobalt/Palladium → Mythril/Orichalcum → Adamantite/Titanium — each spawns deeper after breaking altars with the Pwnhammer.\n• Chlorophyte grows in the Underground Jungle after Plantera.\n• Luminite drops from the Moon Lord — the final crafting tier.",
                &["ore", "bar", "hellstone", "chlorophyte", "luminite", "hardmode", "tier", "mining", "craft"]),
            e(Category::Mechanics, "Hardmode Transition",
                "• Beating the Wall of Flesh permanently flips the world to hardmode — gear up in molten/hellstone first.\n• Smash Demon or Crimson Altars with the Pwnhammer to spawn the new hardmode ores.\n• The Hallow and Corruption/Crimson start spreading aggressively — dig hellevators to quarantine them.\n• Mechanical bosses (Destroyer, Twins, Skeletron Prime) drop souls that unlock endgame recipes.\n• Plantera in the Jungle gates Chlorophyte and the next progression step.",
                &["hardmode", "wall of flesh", "altar", "pwnhammer", "hallow", "corruption", "plantera", "boss"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Terraria, entry));

        // ----------------------------------------------------------- Elden Ring
        [
            e(Category::Items, "Smithing Stones & Weapon Upgrading",
                "• Regular weapons: Smithing Stone [1]–[8] + Ancient Dragon Smithing Stone — each tier upgrades +0→+25 total.\n• Special/boss weapons: Somber Smithing Stone [1]–[10] — +0→+10 total.\n• Stones [1]-[4] are plentiful in tunnels/caves (check Limgrave and Liurnia mining tunnels).\n• Higher tiers are limited per playthrough — buy them after finding Bell Bearings and giving them to the Roundtable Hold blacksmith.\n• Upgrade ONE main weapon early instead of five — rune cost scales brutally.",
                &["smithing stone", "somber", "upgrade", "weapon", "bell bearing", "tunnel", "craft"]),
            e(Category::Strategy, "Runes, Flasks & Leveling",
                "• Runes are lost on death but dropped where you died — grab the glowing spot before dying again.\n• Golden Seeds (glowing small trees) upgrade Flask charges; Sacred Tears (churches) boost potency.\n• Level at churches/lost graces — spread points into HP and stamina early, then your damage stat.\n• Spirit Ashes summon AI allies — upgrade them at Roderika to keep them relevant.\n• Farming spot: the birds at Mogh's Palace or the gray dragon at Fort Faroth pay huge rune chunks.",
                &["rune", "level", "flask", "golden seed", "sacred tear", "spirit ash", "farm", "grace"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::EldenRing, entry));

        // ------------------------------------------------------------- Valheim
        [
            e(Category::Items, "Metal & Gear Progression",
                "• Tier order: wood/flint → copper + tin (Black Forest) → bronze → iron (Sunken Crypts in the Swamp) → silver (Mountains, detect with Wishbone) → black metal (Fuling drops in the Plains).\n• Copper/tin smelt in a Smelter fed by coal — build a Charcoal Kiln next to it.\n• Ores and metals CANNOT be teleported — plan boat/cart haul routes.\n• Upgrade the Workbench (chopping block, tanning rack, adze…) to unlock higher-quality gear.\n• Ferry ore home by longship — it carries the most cargo and survives ocean waves.",
                &["ore", "iron", "copper", "silver", "black metal", "smelter", "workbench", "bronze", "craft"]),
            e(Category::Mechanics, "Resting, Food & Comfort",
                "• The Rested buff (sitting near a fire under shelter) doubles stamina regen and boosts HP regen — re-up it constantly.\n• Food IS your HP/stamina pool — always run 3 different foods, upgrade tiers per biome.\n• Comfort items (chair, rug, banner…) lengthen the Rested timer — improve your base bedroom.\n• Sleep in a bed at night skips the dangerous dark hours and heals you.\n• Wet status drains stamina — carry a shelter plan when exploring.",
                &["rest", "food", "stamina", "comfort", "fire", "buff", "base"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Valheim, entry));

        // ---------------------------------------------------------------- Rust
        [
            e(Category::Items, "Scrap, Components & Workbenches",
                "• Scrap comes from barrels (roadside, monuments), crates, and recycling — recycle junk components for extra scrap.\n• Workbench Tier 1/2/3 gate blueprints: craft the bench, sit at it, and spend scrap to learn recipes.\n• Key components: SMG body, rifle body, metal pipe — farm the Airfield/Launch Site/Dome barrels for them.\n• Sulfur ore → sulfur → gunpowder for ammo; farm it in the desert or buy from Outpost.\n• Recyclers sit at the Dome, Outpost, and in-bandit camps — loot + recycle in one trip.",
                &["scrap", "component", "workbench", "recycler", "sulfur", "gunpowder", "blueprint", "farm"]),
            e(Category::Strategy, "Base Building & Raid Basics",
                "• Multi TC (tool cupboard) coverage stops griefers building into your base.\n• Airlocks: double doors with a gap between them — never let raiders walk straight in.\n• Stone is fine early; sheet metal and armored doors resist way more.\n• Raiding costs explosives — C4, rockets, or a cheap Eoka pistol with handmade shells on wooden doors; sulfur farming funds raids.\n• Hide a small stash box off-base — losing everything in one raid hurts less with a backup.",
                &["base", "raid", "tool cupboard", "airlock", "c4", "rocket", "explosive", "door"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Rust, entry));

        // ------------------------------------------------------------ Palworld
        [
            e(Category::Items, "Paldium, Spheres & Crafting",
                "• Paldium fragments are everywhere: blue glowing rocks, and any Pal-gear crafting consumes them.\n• Pal Spheres: basic → Mega → Giga → Hyper — craft higher tiers to catch stronger Pals.\n• Technology Points unlock recipes — level up and beat bosses for skill points.\n• Craft a Palbox early to build a base; assign Pals with kindling/handiwork traits to automate production.\n• Spheres have catch-power caps — a low-tier sphere on a boss Pal just breaks.",
                &["paldium", "sphere", "craft", "technology point", "palbox", "catch", "fragments"]),
            e(Category::Mechanics, "Base Pals & Automation",
                "• Assign Pals by work suitability: Kindling smelts, Handiwork crafts, Planting/Farming grows food.\n• Keep a Feed Box stocked — starving Pals stop working and get depressed.\n• Build chests near crafting stations so Pals can auto-pull materials.\n• Ranch Pals passively produce materials: Lamball → wool, Chikipi → eggs, Vixy → spheres/arrows.\n• SAN matters — pet your Pals and hot-spring them or they slack off.",
                &["base", "automation", "kindling", "handiwork", "feed", "ranch", "san"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Palworld, entry));

        // ------------------------------------------------------------- Genshin
        [
            e(Category::Items, "Artifacts & Ascension Materials",
                "• Artifacts come from Domains — farm the domain matching the set you need; resin gates runs.\n• Main stats matter more than set bonuses early: ATK% sands, elemental/hydro/cryo goblet, Crit circlet.\n• Ascension materials drop from world bosses — check the character's ascension screen for the exact boss.\n• Weapon ascension uses domain materials on specific days — weapons domains rotate daily.\n• Spend resin before logging off; it caps and wastes time.",
                &["artifact", "domain", "resin", "ascension", "material", "boss", "weapon", "farm"]),
            e(Category::Progression, "Adventure Rank & Resin Planning",
                "• AR 45 is the milestone — highest-level domains start dropping 5-star artifacts.\n• Don't farm artifacts before AR 45; pour resin into EXP books, mora, and talent materials.\n• Daily commissions + resin spend = the core loop; battle pass rewards follow naturally.\n• Level ONE carry character + one good support first — resources are scarce early.\n• Condensed resin doubles domain rewards per run — craft it at an alchemy bench.",
                &["adventure rank", "ar 45", "resin", "daily", "progression", "books", "mora"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Genshin, entry));

        // ------------------------------------------------------- Baldur's Gate 3
        [
            e(Category::Items, "Gear, Gold & Loot Sources",
                "• Check every merchant's stock each act — they restock and scale with your level.\n• Lockpicking (Thieves' Tools + Sleight of Hand) opens most chests; knock spells work on doors.\n• Act bosses and quest rewards carry the best uniques — explore side content before pushing the main quest.\n• Hirelings (Withers) can hold/duplicate gear — and Withers can respec anyone for cheap.\n• Vendor prices improve with Attitude — barter with a high-Charisma character or gift items first.",
                &["gear", "loot", "merchant", "gold", "lockpick", "chest", "vendor", "unique"]),
            e(Category::Mechanics, "Rests, Saves & Party Resources",
                "• Long rests advance story timers — some quests expire, so short-rest when possible.\n• Short rests restore actions + half HP; Warlocks and Monks live off them.\n• Save before every conversation and lockpick — dice rolls can ruin plans.\n• Concentration spells break when the caster is hit — protect your caster or recast.\n• Environmental fights: barrels, fire, water + lightning — the terrain is a free weapon.",
                &["rest", "save", "party", "spell", "concentration", "environment", "dice"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::BaldursGate3, entry));

        // ----------------------------------------------------------- Cyberpunk
        [
            e(Category::Items, "Crafting, Iconics & Cyberware",
                "• Crafting components come from dismantling junk loot — break down everything you don't use.\n• Iconic weapons upgrade to Legendary with the right perk — don't scrap them!\n• Cyberware slots at Ripperdocs: front-line builds want a second heart (Second Heart/Chipware), netrunners want OS decks.\n• Specs/blueprints drop from NCPD scanner hustles and certain bosses.\n• Tier scales with Street Cred — most top-tier items unlock at Cred 30+.",
                &["craft", "iconic", "cyberware", "ripperdoc", "component", "weapon", "spec", "street cred"]),
            e(Category::Progression, "Eddies & Street Cred Farming",
                "• NCPD scanner hustles (blue skull icons) pay cash + Cred and clear fast.\n• Gig contracts from fixers are the best money-per-minute in each district.\n• Sell crafted grenades/monowire mods instead of junk loot for extra eddies.\n• Car/motorcycle races and Delivery side jobs add quick cash.\n• Perk respec costs nothing at a Ripperdoc — experiment freely.",
                &["eddie", "money", "street cred", "gig", "ncpd", "farm", "perk", "level"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Cyberpunk, entry));

        // ---------------------------------------------------------- The Witcher 3
        [
            e(Category::Items, "Witcher Gear Diagrams",
                "• Griffin/Cat/Wolf/Bear (Ursine) sets need diagrams — find maps from vendors or quests, then locate the diagrams.\n• Craft at any Armorer/Blacksmith with the right level (Enhanced → Superior → Mastercrafted → Grandmaster).\n• Mastercrafted tiers need a Master armorer/blacksmith (level ~master tier craftsman in Novigrad/Skellige).\n• Glyphs and runes slot into gear — socket higher-tier ones for big damage/sign boosts.\n• Set bonuses (3/6 pieces) reward full-set play — pick one school per build.",
                &["witcher gear", "diagram", "griffin", "cat", "bear", "wolf", "craft", "armor", "rune", "glyph"]),
            e(Category::Mechanics, "Oils, Decoctions & Alchemy",
                "• Oils apply to swords for +damage vs specific monster types — apply before every fight.\n• Decoctions are long-duration potions brewing from mutagens — drop rates scale with your level.\n• Meditate to refill potions/charges (alcohol auto-consumes).\n• Signs: Quen (shield) is the noob-safe pick; Igni melts grouped humans.\n• White Honey clears toxicity — drink it between decoction-heavy fights.",
                &["oil", "decoction", "alchemy", "potion", "meditate", "sign", "quen", "toxicity"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Witcher3, entry));

        // ------------------------------------------------------------ Diablo IV
        [
            e(Category::Items, "Legendaries, Aspects & Materials",
                "• Legendary Aspects extract at the Occultist from any legendary — imprint onto better bases.\n• Nightmare Sigils run endgame dungeons — Obol vendors sell random sigils for Murmuring Obols.\n• Salvage everything at the Blacksmith — materials matter more than gold.\n• Gem crafts scale with tier — craft higher gems at the Jeweler.\n• Unique items drop from specific bosses (uber uniques from Duriel/Andariel) — target-farm with summit materials.",
                &["legendary", "aspect", "occultist", "sigil", "obol", "salvage", "unique", "boss", "craft"]),
            e(Category::Progression, "World Tiers & Capstone",
                "• World Tier 1/2 for leveling; the Capstone Dungeon (Cathedral of Light) unlocks Tier 3.\n• Tier 3 unlocks sacred items, Nightmare dungeons, and Helltide events.\n• Tier 4 (Fallen Temple capstone) unlocks ancestral items and the real endgame.\n• Renown (explored areas, waypoints, altars of Lilith) grants skill points + potions — collect altars early.\n• Seasonal realms reset characters — altars carry over, so farm them once.",
                &["world tier", "capstone", "nightmare", "helltide", "renown", "altar", "level", "progression"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Diablo4, entry));

        // ----------------------------------------------------------- Destiny 2
        [
            e(Category::Items, "Exotics & Upgrade Materials",
                "• Exotics: raid/dungeon drops, Xur (weekend vendor — bring an Exotic Cipher), and questlines.\n• Nightfall weeks reward Pinnacle exotics on high difficulty.\n• Upgrade Modules come from Banshee-44 daily bounties and vendor rank-ups.\n• Ascendant Alloys (exotic catalysts) drop from weekly raid challenges and Wellspring.\n• Deepsight weapons: extract patterns by attuning red-border drops — craft at the Enclave.",
                &["exotic", "xur", "nightfall", "upgrade module", "ascendant alloy", "catalyst", "deepsight", "craft"]),
            e(Category::Mechanics, "Power Level & Seasonal Loop",
                "• Powerful/Pinnacle sources list in the director — do pinnacles (raids, dungeons, 100k nightfalls) first each week.\n• Prime Engrams drop once you're near cap — auto-bump your level.\n• Seasonal challenges give big XP — battle-pass track levels fast with them.\n• Bounties pre-stack before a season starts — instant early levels.\n• Fireteam power: your lowest gear piece sets your base power — swap before turning in bounties.",
                &["power", "pinnacle", "prime", "season", "bounty", "xp", "level"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Destiny2, entry));

        // ---------------------------------------------------------- Overwatch 2
        [
            e(Category::Mechanics, "Ultimate Economy & Value",
                "• Don't pop ults when your team is dead — wait to sync with a push.\n• Combo pairs: Grav + Dragonstrike, Zarya bubble + Genji blade, Halt + Hook.\n• Track enemy ults by sound — a whiffed enemy ult is a free fight for you.\n• Support ults save fights; tank ults win space — use them on YOUR tempo, not in panic.\n• One-fight ults (Dva bomb, High Noon) are gamble plays — fine when behind.",
                &["ultimate", "ult", "combo", "economy", "sync", "team"]),
            e(Category::Strategy, "Positioning by Role",
                "• Tank: take ground slowly — space is the win condition, not kills.\n• DPS: play off-angle (a second sightline) — force the enemy to choose targets.\n• Support: stay behind cover with an escape route; ping flankers for your team.\n• Never peek the same angle twice in a row.\n• Group up AFTER a lost fight — staggering = feeding kills.",
                &["positioning", "tank", "dps", "support", "off angle", "cover", "group"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Overwatch, entry));

        // ------------------------------------------------------------ Warframe
        [
            e(Category::Items, "Frame Farming & Mods",
                "• Warframe parts drop from specific bosses — check the codex/wiki for the frame you want; Nekros (Desecrate) and Hydroid (Pilfering Swarm) boost drop rates.\n• Prime frames come from Void Relics — open Lith/Meso/Neo/Axi relics in corresponding fissure missions.\n• Mods ARE your power: rank key mods (Serration, Vitality, Redirection) before new frames.\n• Orokin Catalysts/Reactors (potatoes) double mod capacity — farm alerts/invasions or buy with platinum.\n• Platinum comes from trading prime parts with other players — never buy weapons with it early.",
                &["farm", "relic", "prime", "mod", "platinum", "catalyst", "reactor", "boss", "drop"]),
            e(Category::Progression, "Star Chart & Junctions",
                "• Clear junctions to unlock planets — each junction has simple objectives (kill Vay Hek, craft X).\n• Rank up your Mastery by leveling new weapons/frames — MR gates good gear.\n• Do the main quests (Second Dream onward) — they unlock essential systems.\n• Nightwave challenges give free slots and aura mods — do the easy dailies.\n• Extractor drones passively farm resources on unlocked planets.",
                &["junction", "planet", "mastery", "quest", "nightwave", "progression", "slot"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Warframe, entry));

        // ---------------------------------------------------------- Subnautica
        [
            e(Category::Items, "Blueprints & Resources by Biome",
                "• Blueprints come from scanning fragments — Seamoth in the shallows/grass plateaus, Cyclops in wrecks (mushroom forest, floaters island).\n• Prawn Suit fragments sit in deeper wrecks — bring a Seamoth with a depth module first.\n• Silver (Argentite) is in the Kelp Forest caves; Gold in the Grand Reef/Sea Treader's Path; Diamond in the Bulb Zone/Deep Grand Reef.\n• Upgrade modules craft at the Modification Station — depth modules unlock deeper zones.\n• Always carry 2 medkits, food, water, and a repair tool in the vehicle.",
                &["blueprint", "fragment", "seamoth", "cyclops", "prawn", "silver", "gold", "diamond", "depth", "scan"]),
            e(Category::Progression, "Depth Progression Route",
                "• Shallows (safe) → Kelp Forest (Seaglide) → Grass Plateaus (Seamoth) → deeper biomes with depth upgrades.\n• Every vehicle depth module = one more stage of exploration — craft before pushing deeper.\n• Reinforced dive suit + radiation suit gate the Aurora and lava zones.\n• Cured food (peeper + salt) never spoils — stock it for deep trips.\n• Beacon everything: wrecks, your base, the Aurora — getting lost is the real killer.",
                &["depth", "vehicle", "progression", "aurora", "suit", "beacon", "explore"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Subnautica, entry));

        // ----------------------------------------------------------------- ARK
        [
            e(Category::Items, "Taming, Kibble & Resources",
                "• Tame with knockouts: tranq arrows/darts + narcotics to keep torpor up; feed its preferred food.\n• Kibble from dino eggs tames WAY faster — build a small egg farm of easy dinos first (Dodos, Parasaur).\n• Metal is in mountain rocks and river nodes — an Ankylosaurus doubles metal weight/harvest.\n• Obsidian and crystal live on peaks; oil in the ocean or snow biome.\n• A Tamed Argentavis or Pteranodon is your first big travel/metal-haul unlock.",
                &["tame", "kibble", "tranq", "narcotic", "metal", "obsidian", "crystal", "anky", "argentavis", "resource"]),
            e(Category::Mechanics, "Base & Engram Planning",
                "• Engram points are limited — learn only what you need; mindwipe tonic respecs later.\n• Stone foundations + metal spike walls deter early raptors — build on high ground.\n• Preserving bins + sparkpowder keep meat long; refrigerators need power.\n• Idle tamed dinos in a pen with troughs — wild dinos WILL attack them.\n• Server PVE vs PVP changes everything: on PVP, hide your base with thatch/stone tricks and turrets.",
                &["engram", "base", "preserving", "trough", "pen", "pvp", "turret", "build"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::ARK, entry));

        // --------------------------------------------------------- No Man's Sky
        [
            e(Category::Items, "Units, Nanites & Crafting Chains",
                "• Chlorine farming: buy chlorine, add oxygen in a refiner, sell the multiplied output — the classic unit printer.\n• Activated Indium farms (mineral extractors on activated indium planets) print passive units — nerfed over the years but still solid.\n• Nanites: run abandoned buildings, submit curiosities (Runaway Mould duplicates in refiners!), and cook food for sale.\n• Salvaged Data (buried tech modules) unlock base blueprints — farm with the terrain scanner.\n• Navigation Data from space stations buys buried-tech maps from the cartographer — instant blueprint hunts.",
                &["units", "nanite", "chlorine", "indium", "refiner", "salvaged data", "farm", "craft", "money"]),
            e(Category::Mechanics, "Freighter & Base Basics",
                "• Your first freighter (free after rescuing one from a pirate ambush) stores everything and warps with you.\n• Build a base on a calm planet near a trade route; claim with a Base Computer.\n• Power: electromagnetic hotspots give free infinite power — find them with the analysis visor.\n• The terrain manipulator mines everything — upgrade its modules for speed.\n• Follow the Artemis/Apollo questline for the core unlocks (teleporters, portals).",
                &["freighter", "base", "power", "electromagnetic", "terrain", "quest", "teleport"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::NoMansSky, entry));

        // ----------------------------------------------------------------- RDR2
        [
            e(Category::Items, "Legendary Animals & Perfect Pelts",
                "• Legendary animals spawn at fixed spots (map marks after the first/trapper intro) — they never flee, but bring a good rifle.\n• Perfect pelts need the RIGHT weapon: small game → varmint rifle, deer → rifle with scope, elites → improved arrows.\n• Study the animal first (binoculars) — it shows pelt quality; a 3-star clean kill drops a perfect pelt.\n• Legendary pelts sell ONLY to the Trapper (and fence for legendary items) — don't lose them to horse death.\n• Satchel upgrades (Pearson's) need perfect pelts — upgrade early for more carrying capacity.",
                &["legendary", "animal", "pelt", "trapper", "hunt", "rifle", "satchel", "perfect"]),
            e(Category::Mechanics, "Money & Honor Basics",
                "• Rob trains/stores, sell pelts and gold bars (railroad bridge cave, wrecked ship) for fast cash.\n• Honor affects store discounts, story endings, and bounty sizes.\n• Pay bounties at the post office or they spawn bounty hunters forever.\n• Horse bonding: feed/pat your horse; level 4 bonding unlocks skidding + drift.\n• Dead Eye upgrades with kills — mark targets (tag) for guaranteed multi-kills.",
                &["money", "honor", "bounty", "horse", "dead eye", "rob", "sell"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::RDR2, entry));

        // -------------------------------------------------------- Phasmophobia
        [
            e(Category::Items, "Gear Tiers & Loadout Planning",
                "• Tier priorities: EMF reader, Spirit Box, UV flashlight, Video camera (ghost orbs) — those four IDs most ghosts.\n• Bring a thermometer — freezing temps confirm specific ghosts instantly.\n• Smudge sticks + lighter interrupt hunts and reset the ghost's activity.\n• Sanity pills at the truck keep investigations long — low sanity = more hunts.\n• Upgrade equipment (Tier 2/3) massively boosts detection range — prioritize the basics before fancy items.",
                &["gear", "emf", "spirit box", "uv", "thermometer", "smudge", "sanity", "loadout"]),
            e(Category::Strategy, "Evidence & Hunt Survival",
                "• Cross off ghosts by evidence — 3 items narrow 30 ghosts to one.\n• Hunts: lights flicker + front door opens — hide behind furniture, break line of sight, stay silent (mic!).\n• Crucifix prevents hunts in range — place it NEAR the ghost room, not on top of you.\n• Ghost events lower sanity hard; use the truck as a sanity recharger zone.\n• Photo rewards: ghost events, dirty water, bone — snap them for bonus cash.",
                &["evidence", "hunt", "crucifix", "hide", "ghost", "photo", "sanity"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Phasmophobia, entry));

        // ------------------------------------------------------- Lethal Company
        [
            e(Category::Items, "Scrap Value & Quota Strategy",
                "• Two-handed items (big scrap) pay the most — coordinate who carries and who guards.\n• Apparatus (the glowing core) is worth a fortune but blacks out the facility — grab it LAST.\n• Scan (right-click) highlights scrap values — grab 100+ value items first on short days.\n• Sell early to hit quota, then bank surplus — overselling wastes profit (quota scales).\n• Buy: flashlight first, then stun grenade/shovel for the hostile monsters.",
                &["scrap", "quota", "value", "apparatus", "sell", "flashlight", "shovel", "farm"]),
            e(Category::Mechanics, "Monsters & Moon Selection",
                "• Experimentation/Assurance are safe moons; Titan pays huge but spawns the deadliest enemies.\n• Coil-heads: freeze when watched — one player watches, one loots.\n• Brackens stalk silently — DON'T stare at them (eye contact enrages them); face away and back off to break line of sight.\n• Jesters wind up and explode into a chase — leave the building when you hear the wind-up.\n• Doors + dropships are safe zones — lure monsters into the ship door and close it.",
                &["monster", "coil head", "bracken", "jester", "moon", "titan", "safety", "door"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::LethalCompany, entry));

        // ---------------------------------------------------------------- Hades
        [
            e(Category::Items, "Boons, Keepsakes & Mirror",
            "• Boons from gods stack by rarity — focus 2-3 gods per run for Duo/Legacy boons.\n• Keepsakes: equip one per god to force their first boons; keep Skelly's / Cerberus' for early value.\n• Mirror of Night upgrades (Death Defiance, Greater Reflex) persist forever — buy these first.\n• Chthonic Keys unlock the Mirror; Titan Blood upgrades weapon Aspects; Diamonds buy House Contractor perks (Greater Keepsake Collection first).\n• Obols spent at Charon's shop — the Pom of Power (boon levels) is usually the best buy.",
                &["boon", "keepsake", "mirror", "titan blood", "diamond", "obol", "charon", "pom", "level"]),
            e(Category::Strategy, "Weapon Aspects & Run Building",
                "• Each weapon has Aspects unlocked with Titan Blood — Aspect of Zeus shield / Zag bow are beginner-friendly.\n• Heat (Pact of Punishment) raises rewards — add heat gradually as you get comfortable.\n• Build around ONE core boon (e.g., Aphrodite attack + Artemis crit) and stack supports.\n• Hermes boons (dashes/speed) stack with everything — take them whenever offered.\n• Priority rooms: Daedalus hammers (weapon upgrades) > Poms > Gold.",
                &["weapon", "aspect", "heat", "hammer", "hermes", "build", "run", "reward"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Hades, entry));

        // -------------------------------------------------------- Hollow Knight
        [
            e(Category::Items, "Charms, Mask Shards & Vessel Fragments",
                "• Charms define builds: Quick Slash, Shaman Stone, Mark of Pride are top-tier — notch limits gate combos.\n• Charm notches: buy from Salubra (5), find others via bosses/challenges.\n• Mask Shards (4 = +1 HP) hide behind paths/puzzles — check the map vendor's hints.\n• Vessel Fragments (3 = +1 soul) similarly extend your spell reserves.\n• Geo farming: the Trial of the Warrior or Lost Kin / Husk Guards routes pay big.",
                &["charm", "notch", "mask shard", "vessel", "geo", "soul", "salubra", "farm"]),
            e(Category::Strategy, "Exploration & Ability Gating",
                "• Abilities gate areas: Mothwing Cloak (dash), Mantis Claw (wall cling), Monarch Wings (double jump), Isma's Tear (acid swim).\n• Stuck? Revisit old zones with the new ability — the map is full of one-way locks.\n• Buy the map from Cornifer in each area (or hear his humming) and pin everything with Iselda.\n• Benches are your save points — rest before every boss and re-tune charms.\n• Dream Nail on spirits/ghosts gives extra lore + soul to fuel spell spam.",
                &["ability", "dash", "claw", "wings", "bench", "map", "cornifer", "explore", "spell"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::HollowKnight, entry));

        // ------------------------------------------------------- Final Fantasy XIV
        [
            e(Category::Items, "Tomestones & Gear Acquisition",
                "• Current tomestones (Allagan/Aesthetics) buy best-in-slot gear — cap them weekly (450/2000 limits).\n• Raid gear drops from Savage tiers; crafted gear from market board is a strong catch-up.\n• Trade old tomestones at Auriana (Mor Dhona) for items that sell — early-week money.\n• Retainers run ventures — send them on 18h expeditions for materials and quick profit.\n• Glamour + dyes come from raids/events — the Glamour Dresser in inns stores everything.",
                &["tomestone", "gear", "raid", "retainer", "venture", "market board", "glamour", "farm"]),
            e(Category::Progression, "MSQ, Roulette & Leveling",
                "• Main Scenario Quest (MSQ) is the spine — it gates everything; follow it to max level.\n• Duty Roulette: Leveling/Trial/Main Story give huge daily XP bonuses.\n• Palace of the Dead (1-60), Heaven-on-High (61-70), and Bozja (71-90) level alt jobs fast.\n• Food gives +3% XP for 30 min — always eat something.\n• Road to 80 buff (on preferred servers) doubles XP below max level — pick it when starting fresh.",
                &["msq", "roulette", "level", "potd", "hoh", "food", "xp", "job"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::FinalFantasy14, entry));

        // -------------------------------------------------------- Sea of Thieves
        [
            e(Category::Items, "Treasure, Emissary & Trading",
                "• Gold Hoarders: X-maps/riddles → chests; Order of Souls: bounty skulls; Merchant Alliance: commodity runs.\n• Emissary flags boost loot value up to 2.5x — but make you a target; raise after stacking quests, lower before logging off.\n• Chest of Sorrows leaks your ship — keep it on the rowboat, off the deck.\n• Stronghold loot from Skeleton Forts pays the most per hour when they spawn near you.\n• Storage crates from shipwrecks haul extra loot — stack them before a big haul.",
                &["treasure", "emissary", "chest", "skull", "merchant", "gold", "fort", "loot", "sell"]),
            e(Category::Strategy, "Ship Combat & Survival",
                "• Always repair below-deck holes FIRST — water is the real killer, not cannons.\n• Chainshots kill masts — an immobile ship is a free sink.\n• Firebombs + blunderbombs disrupt boarding parties; grapeshot clears decks before boarding.\n• Anchor-turn maneuver: drop anchor mid-turn for a sharp dodge/fight angle.\n• Sell often — dying with 40 chests onboard is the classic Sea of Thieves lesson.",
                &["ship", "cannon", "chainshot", "repair", "anchor", "sell", "combat"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::SeaOfThieves, entry));

        // ------------------------------------------------------- Monster Hunter
        [
            e(Category::Items, "Carving, Parts & Crafting",
                "• Break parts (head/wings/tail) BEFORE the kill — breaking gives extra carves and unlocks gear parts.\n• Cut tails with slicing damage to carve one extra slot — tails are key weapon materials.\n• Capturing (Shock/Pitfall trap + tranq bombs) yields different rewards than carving — some parts only come from capture.\n• Investigations with gold/silver rewards pay the best material drops — refresh them often.\n• Canteen meals before every hunt — the right skill food (sharpness/hunger) matters.",
                &["carve", "part", "break", "capture", "trap", "investigation", "material", "craft", "canteen"]),
            e(Category::Mechanics, "Weapon Mastery & Armor Skills",
                "• Pick ONE weapon and learn it — each is a separate game-with-a-game.\n• Armor skills (Attack Boost, Critical Eye, Weakness Exploit) scale builds — build around a 3-skill core.\n• Hit weak points (glowing orange spots after damage) — WEX + Crit Boost multiplies damage.\n• Max Potions + Mega Potions + Astera Jerky = your heal kit; craft before leaving.\n• Palico gadgets (Flashfly cage, Coral Orchestra) add free buffs and heals.",
                &["weapon", "armor", "skill", "weakness", "potion", "palico", "damage", "build"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::MonsterHunter, entry));

        // ------------------------------------------------------------------ Generic
        [
            e(Category::Mechanics, "Improving Your Aim",
                "• Pick ONE sensitivity and never change it — muscle memory needs consistency.\n• 15 minutes of daily aim trainers (or DM warmups) beats hours of unfocused matches.\n• Crosshair placement: keep the crosshair at head level and pre-aim corners.\n• Lower sensitivity generally = more consistent for beginners (aim with your arm, fine-tune with wrist).\n• Record your misses: are you flicking early or late? Fix one error at a time.",
                &["aim", "sens", "practice", "crosshair", "warmup", "sensitivity"]),
            e(Category::Strategy, "Communication Basics",
                "• Short, factual callouts: location + what + how many. 'Two pushing B' > 'they're everywhere!'.\n• Use the ping system — it's language-free and faster than typing.\n• Stay positive: teams that talk tilt lose more games than teams with bad aim.\n• Call what you KNOW vs what you GUESS — mark guesses as guesses.\n• Play the same games with a consistent duo/group — chemistry is a real skill multiplier.",
                &["team", "communicate", "ping", "callout", "duo", "tilt"]),
            e(Category::Strategy, "Tilt Control",
                "• 2-loss rule: after two losses in a row, take a 15-minute break — save your rating.\n• Review, don't rage: watch one replay of a loss and find ONE fixable mistake.\n• Physical resets: water, stretch, screen brightness down — tilt is physical too.\n• Mute toxic players instantly; their comms cost you more than the info is worth.\n• End sessions on a win or a fun game — your brain remembers how sessions end.",
                &["tilt", "mental", "break", "replay", "toxic", "mute"]),
            e(Category::Progression, "Hardware & Settings",
                "• FPS stability > raw FPS: cap your frame rate slightly below your max for consistent frametimes.\n• Lower graphics presets in competitive games — more FPS AND fewer visual distractions.\n• Mouse: 800-1600 DPI in the driver, low in-game sens is the standard pro setup.\n• Monitor: anything 120Hz+ feels dramatically smoother for reaction-based games.\n• Update GPU drivers before blaming the game — half of stutter issues are driver bugs.",
                &["settings", "fps", "hardware", "dpi", "monitor", "driver", "graphics"]),
        ]
        .into_iter()
        .for_each(|entry| kb.add(GameType::Generic, entry));

        kb
    }

    fn add(&mut self, game: GameType, entry: KnowledgeEntry) {
        self.entries.push((game, entry));
    }

    /// All entries for a game, in insertion order.
    pub fn entries_for_game(&self, game: &GameType) -> Vec<&KnowledgeEntry> {
        self.entries
            .iter()
            .filter(|(g, _)| g == game)
            .map(|(_, e)| e)
            .collect()
    }

    /// Number of entries stored for a game.
    pub fn count_for_game(&self, game: &GameType) -> usize {
        self.entries.iter().filter(|(g, _)| g == game).count()
    }

    /// Score-based keyword search over a game's entries.
    /// Returns up to `limit` best matches, strongest first.
    pub fn search(&self, game: &GameType, query: &str, limit: usize) -> Vec<&KnowledgeEntry> {
        let lower = query.to_lowercase();
        let words: Vec<&str> = lower
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()))
            .filter(|w| w.len() > 1 && !STOPWORDS.contains(w))
            .collect();
        if words.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(usize, &KnowledgeEntry)> = self
            .entries
            .iter()
            .filter(|(g, _)| g == game)
            .filter_map(|(_, entry)| {
                let title = entry.title.to_lowercase();
                let content = entry.content.to_lowercase();
                let mut score = 0usize;
                for w in &words {
                    if entry.tags.iter().any(|t| matches_word(t, w)) {
                        score += 4;
                    }
                    if title.contains(w) {
                        score += 3;
                    }
                    if content.contains(w) {
                        score += 1;
                    }
                }
                if score > 0 {
                    Some((score, entry))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.title.cmp(b.1.title)));
        scored.truncate(limit);
        scored.into_iter().map(|(_, entry)| entry).collect()
    }

    /// Fallback search across ALL games — used ONLY in General Chat mode,
    /// where no single game is selected. A selected game never receives
    /// another game's knowledge.
    pub fn search_any(&self, query: &str, limit: usize) -> Vec<(GameType, &KnowledgeEntry)> {
        let lower = query.to_lowercase();
        let words: Vec<&str> = lower
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()))
            .filter(|w| w.len() > 1 && !STOPWORDS.contains(w))
            .collect();
        if words.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(usize, GameType, &KnowledgeEntry)> = self
            .entries
            .iter()
            .filter_map(|(game, entry)| {
                let title = entry.title.to_lowercase();
                let content = entry.content.to_lowercase();
                let mut score = 0usize;
                for w in &words {
                    if entry.tags.iter().any(|t| matches_word(t, w)) {
                        score += 4;
                    }
                    if title.contains(w) {
                        score += 3;
                    }
                    if content.contains(w) {
                        score += 1;
                    }
                }
                if score > 0 {
                    Some((score, game.clone(), entry))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.title.cmp(b.2.title)));
        scored.truncate(limit);
        scored.into_iter().map(|(_, game, entry)| (game, entry)).collect()
    }
}

/// Format knowledge base hits as a chat answer.
pub fn format_answer(hits: &[&KnowledgeEntry]) -> String {
    let mut out = String::from("📚 From the ROCKET Knowledge Base:\n\n");
    for (i, hit) in hits.iter().enumerate() {
        out.push_str(&format!("{} — {}\n{}\n", hit.category.label(), hit.title, hit.content));
        if i + 1 < hits.len() {
            out.push_str("\n────────────────────\n\n");
        }
    }
    out
}

/// Chat-flavored answer — the same curated content, framed like a friend
/// talking instead of a database dump. Used by the chat pipeline.
pub fn format_chat_answer(hits: &[&KnowledgeEntry]) -> String {
    let mut out = String::new();
    for (i, hit) in hits.iter().enumerate() {
        if i == 0 {
            out.push_str(&format!("{} — {}\n{}\n", hit.category.label(), hit.title, hit.content));
        } else {
            out.push_str(&format!("\nAlso useful — {} {}:\n{}\n", hit.category.label(), hit.title, hit.content));
        }
    }
    out.push_str("\n🔥 Say \"more\" for another tip on this.");
    out
}

/// Format cross-game hits, labeling which game each guide is from.
pub fn format_cross_answer(hits: &[(GameType, &KnowledgeEntry)]) -> String {
    let mut out = String::from("📚 From the ROCKET Knowledge Base (other games):\n\n");
    for (i, (game, hit)) in hits.iter().enumerate() {
        out.push_str(&format!(
            "{} — {} (from {})\n{}\n",
            hit.category.label(),
            hit.title,
            game.get_name(),
            hit.content
        ));
        if i + 1 < hits.len() {
            out.push_str("\n────────────────────\n\n");
        }
    }
    out
}

/// Universal item knowledge — answers "how do I get/craft/find X" for ANY
/// item in ANY game, even when nothing curated matches. Returns None when
/// the question isn't an item-acquisition question.
pub fn universal_item_answer(game: &GameType, query: &str) -> Option<String> {
    let lower = query.to_lowercase();
    const TRIGGERS: &[&str] = &[
        "how do i get",
        "how to get",
        "how can i get",
        "how do you get",
        "where do i get",
        "where to get",
        "where do i find",
        "where to find",
        "where can i find",
        "how do i craft",
        "how to craft",
        "how do you craft",
        "how do i make",
        "how to make",
        "how do i farm",
        "how to farm",
        "best way to get",
        "best way to farm",
        "best way to find",
        "recipe for",
    ];
    let trigger = TRIGGERS.iter().find(|t| lower.contains(*t))?;
    let pos = lower.find(trigger)?;
    let after = &lower[pos + trigger.len()..];

    const FILLER: &[&str] = &[
        "in", "the", "a", "an", "on", "at", "my", "this", "game", "fast", "faster", "quick",
        "quickly", "easy", "easily", "more", "some", "it", "them", "good", "better", "started",
        "into", "out", "any",
    ];
    let item: String = after
        .split(|c: char| c == '?' || c == '.' || c == '!' || c == ',' || c == ';')
        .next()
        .unwrap_or(after)
        .split_whitespace()
        .filter(|w| !FILLER.contains(w))
        .collect::<Vec<_>>()
        .join(" ");
    if item.len() < 2 {
        return None;
    }

    let item_display: String = item
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let genre_tip: &str = match game.get_category() {
        GameFilter::Sandbox => "🧱 Sandbox/crafting route: check your crafting menu or the right station (workbench → furnace → anvil…), then gather the raw materials — core items usually come from a specific depth, biome, or time of day.",
        GameFilter::Shooter => "🔫 Shooter route: this kind of item usually comes from loot spawns, chests/supply drops, boss rewards, or the shop/battle pass — learn the fixed spawn spots on your favorite maps and prioritize them.",
        GameFilter::Rpg => "🐉 RPG route: expect boss drops, quest rewards, vendor stock, and crafting from gathered materials — the item's tooltip/recipe usually names exactly where its pieces come from.",
        GameFilter::Mmo => "🌐 MMO route: check the auction house/marketboard first (often cheaper than farming), then farm the specific dungeon/zone/boss that drops it — and mind daily/weekly lockouts.",
        GameFilter::Horror => "👻 Horror route: gear and key items sit on fixed spawns or get bought between runs — bring the right equipment for the right map and learn which rooms spawn what.",
        GameFilter::OpenWorld => "🗺️ Open-world route: check the in-game codex/encyclopedia entry — it usually lists sources; otherwise expect vendors, camps/outpost drops, and settlement crafting stations.",
        GameFilter::Simulation => "🚜 Simulation route: items come from progression chains — each tier of money/level unlocks the next equipment tier, so max out what you have before pushing up.",
        GameFilter::Platformer => "🏃 Platformer route: key items hide behind optional paths — backtrack old areas with new abilities, break suspicious walls, and check reward shrines.",
        GameFilter::Moba => "⚔️ MOBA route: in-match items follow a build order from the store; meta items (skins, battle-pass goods) come from events and the shop.",
        GameFilter::Fighting => "🥊 Fighting route: gear and costumes come from arcade/story modes, challenges, or the shop — in-match loadouts are picked before the round.",
        GameFilter::Strategy => "♟️ Strategy route: check the tech tree — the item/unit needs the right building and tech level first; queue prerequisite structures before farming its cost.",
        GameFilter::Sports | GameFilter::Racing => "🏁 Sports/racing route: earn the in-game currency from events, then buy from the store/marketplace — the best gear is gated behind seasons or tiers.",
        GameFilter::Party => "🎉 Party route: cosmetics and unlocks come from playing matches and event rewards — check the unlock conditions on the item's menu entry.",
        _ => "🎮 Universal route: check the item's in-game tooltip/codex — it almost always lists where it drops or what it's crafted from.",
    };

    Some(format!(
        "🔎 How to get {} in {} — the 6 universal routes, in the order to try them:\n\n\
         1. 🛠️ Craft it — check crafting menus/stations for a recipe that needs it or produces it.\n\
         2. ⛏️ Gather the raw materials — ores, plants and loot usually live in one specific biome, depth, or zone.\n\
         3. 👹 Kill the source — many items only drop from specific enemies, bosses, or rare spawns.\n\
         4. 📦 Open containers — chests, crates, barrels and supply drops are common unique sources.\n\
         5. 🛒 Buy it — vendors, auction houses and shops often sell what's tedious to farm.\n\
         6. 🎯 Quest for it — story/side quests and achievements reward unique items outright.\n\n{}\n\n\
         💡 Search the exact item name in the 📚 Knowledge panel too — curated guides cover the most popular games, and 🤖 Online AI mode can answer about literally any item.",
        item_display,
        game.get_name(),
        genre_tip
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netherite_question_finds_netherite_entry() {
        let kb = KnowledgeBase::new();
        let hits = kb.search(&GameType::Minecraft, "how to get netherite", 2);
        assert!(!hits.is_empty(), "no KB hits for 'how to get netherite'");
        assert!(
            hits[0].title.to_lowercase().contains("netherite"),
            "top hit was '{}' — expected the Netherite entry",
            hits[0].title
        );
    }

    #[test]
    fn diamond_question_finds_diamond_entry() {
        let kb = KnowledgeBase::new();
        let hits = kb.search(&GameType::Minecraft, "how do I find diamonds", 2);
        assert!(!hits.is_empty());
        assert!(hits[0].title.to_lowercase().contains("diamond"));
    }

    #[test]
    fn aim_question_finds_generic_aim_entry() {
        let kb = KnowledgeBase::new();
        let hits = kb.search(&GameType::Generic, "how to improve my aim", 2);
        assert!(!hits.is_empty());
        assert!(hits[0].title.to_lowercase().contains("aim"));
    }

    #[test]
    fn search_respects_game_filter() {
        let kb = KnowledgeBase::new();
        let hits = kb.search(&GameType::Valorant, "netherite", 5);
        assert!(hits.is_empty(), "Minecraft entries must not leak into Valorant");
    }

    #[test]
    fn cross_game_search_finds_other_games() {
        let kb = KnowledgeBase::new();
        let hits = kb.search_any("netherite", 2);
        assert!(!hits.is_empty(), "cross-game search missed the netherite guide");
        assert_eq!(hits[0].0, GameType::Minecraft);
    }

    #[test]
    fn new_game_entries_are_searchable() {
        let kb = KnowledgeBase::new();
        assert!(!kb.search(&GameType::Terraria, "hellstone", 2).is_empty());
        assert!(!kb.search(&GameType::EldenRing, "smithing stone", 2).is_empty());
        assert!(!kb.search(&GameType::Valheim, "iron", 2).is_empty());
        assert!(!kb.search(&GameType::Palworld, "sphere", 2).is_empty());
    }

    #[test]
    fn universal_item_answer_triggers_on_any_item() {
        let answer = universal_item_answer(&GameType::CS2, "how do I get a dragon lore");
        assert!(answer.is_some());
        assert!(answer.unwrap().contains("Dragon Lore"));

        assert!(universal_item_answer(&GameType::Palworld, "where to find sulfur").is_some());
        assert!(universal_item_answer(&GameType::Hades, "how to farm titan blood").is_some());

        // Non item-questions must not trigger it.
        assert!(universal_item_answer(&GameType::Generic, "what is your favorite color").is_none());
        assert!(universal_item_answer(&GameType::Generic, "hello").is_none());
    }
}
