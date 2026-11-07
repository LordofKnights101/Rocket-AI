//! ROCKET's brain — a local gaming AI engine with a knowledge base,
//! a learning memory, and personalization. Runs fully in the browser.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Reply tags (shown as badges on chat bubbles)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Learned,
    Knowledge,
    SmallTalk,
    JustLearned,
    FollowUp,
    TeachMe,
}

impl Tag {
    pub fn badge(self) -> &'static str {
        match self {
            Tag::Learned => "⚡ answered from memory — instant recall",
            Tag::Knowledge => "📚 gaming knowledge base",
            Tag::SmallTalk => "💬 chat",
            Tag::JustLearned => "🎓 learned from you — saved",
            Tag::FollowUp => "🔥 more tips",
            Tag::TeachMe => "🧠 learning mode",
        }
    }
}

pub struct Reply {
    pub text: String,
    pub tag: Tag,
}

// ---------------------------------------------------------------------------
// Static knowledge base
// ---------------------------------------------------------------------------

pub struct Fact {
    keywords: &'static [&'static str],
    text: &'static str,
}

pub struct Topic {
    pub name: &'static str,
    keywords: &'static [&'static str],
    facts: &'static [Fact],
}

const TOPICS: &[Topic] = &[
    Topic {
        name: "Minecraft",
        keywords: &["minecraft", "mc", "creeper", "netherite", "redstone", "villager"],
        facts: &[
            Fact {
                keywords: &["diamond", "diamonds"],
                text: "Diamonds are most common around Y -58/-59. Strip-mine at that level and keep a water bucket handy — there's almost always lava nearby!",
            },
            Fact {
                keywords: &["creeper", "creepers"],
                text: "Creepers start hissing when they're about 3 blocks away and explode ~1.5s later. Back off, or hit-and-retreat with a sword — a shield blocks the whole blast!",
            },
            Fact {
                keywords: &["netherite", "debris", "ancient"],
                text: "For Netherite: mine Ancient Debris around Y 15 in the Nether, smelt it into scraps, craft ingots (4 scraps + 4 gold), then upgrade diamond gear at a Smithing Table.",
            },
            Fact {
                keywords: &["redstone"],
                text: "Redstone 101: a Repeater extends the signal another 15 blocks, Comparators read how full a container is, and Observers detect block changes. Start by building a simple automatic farm!",
            },
            Fact {
                keywords: &["enderman", "endermen"],
                text: "Endermen get angry when you look at their eyes. Wear a carved pumpkin on your head and they won't aggro — great for farming Ender Pearls safely.",
            },
            Fact {
                keywords: &["villager", "villagers", "trading", "trade"],
                text: "Cure a Zombie Villager (Weakness potion + golden apple) and their trades get permanently cheaper. Librarians can sell Mending books — reroll their stock by breaking and replacing the lectern!",
            },
            Fact {
                keywords: &["portal", "nether"],
                text: "A Nether Portal is 4x5 minimum (10 obsidian — corners optional). Light the inside with flint & steel. Remember: 1 block traveled in the Nether = 8 blocks in the Overworld!",
            },
            Fact {
                keywords: &["dragon", "ender dragon", "the end"],
                text: "For the Ender Dragon: destroy the End Crystals on top of the obsidian towers first (snowballs work great!), then attack while it's perched on the central fountain.",
            },
            Fact {
                keywords: &["bed", "beds", "sleep", "spawn"],
                text: "Beds set your spawn point and skip the night — but they EXPLODE in the Nether and the End. Never place one there (unless you're speedrunning bed-bombing a boss).",
            },
            Fact {
                keywords: &["enchant", "enchanting", "enchantment", "enchantments"],
                text: "Enchanting: surround an Enchanting Table with 15 bookshelves for level-30 enchants. Mending is the best all-round enchantment — grab it from Librarian villagers.",
            },
            Fact {
                keywords: &["food", "hunger", "eat"],
                text: "Steak and cooked porkchops are the best early-game foods. Always carry food — when the hunger bar drops low, your health stops regenerating.",
            },
            Fact {
                keywords: &["start", "beginner", "begin", "first day"],
                text: "New world checklist: punch a tree, craft a table, wooden pickaxe → stone tools, then get a shelter up before night 1. Torches stop monsters from spawning inside.",
            },
        ],
    },
    Topic {
        name: "Roblox",
        keywords: &["roblox", "rbx", "blox", "obby"],
        facts: &[
            Fact {
                keywords: &["robux", "free robux"],
                text: "There is NO such thing as free Robux. Generators, 'RBX deals' and login sites are scams that steal your account. Only buy Robux in-app or earn them via Premium payouts.",
            },
            Fact {
                keywords: &["blox fruits", "blox fruit", "sea", "grind"],
                text: "Blox Fruits: pick a fruit that fits your style (Buddha for grinding, Light for travel), do quests at YOUR level instead of jumping islands, and save boosts for 2x XP weekends.",
            },
            Fact {
                keywords: &["obby", "obby"],
                text: "Obby tip: hold jump and learn each obstacle's rhythm. Camera angle matters — zoom out on moving platforms, and test invisible paths by throwing something first.",
            },
            Fact {
                keywords: &["scam", "scammed", "hacked", "account", "password"],
                text: "Account safety: turn on 2-Step Verification, never share your password, and ignore 'I'll give you Robux if you...' messages. Report + block scammers.",
            },
            Fact {
                keywords: &["bloxburg", "money", "job"],
                text: "Bloxburg: the Delivery Person job pays steadily (level it by taking big orders), builder jobs pay more per level, and don't forget the Daily Bonus streak for free money.",
            },
            Fact {
                keywords: &["adopt me", "pets", "pet"],
                text: "Adopt Me: pet value comes from rarity + demand — check a live value list before trading, and never do 'trust trades' (that's always a scam).",
            },
            Fact {
                keywords: &["studio", "script", "scripting", "luau", "coding"],
                text: "Want to make your own game? Learn Luau in Roblox Studio — start by reading and editing scripts inside free models, then follow the official Creator docs. Every big Roblox dev started with a broken obby 😄",
            },
            Fact {
                keywords: &["trade", "trading", "value"],
                text: "Before any trade, check the item's current value — values shift week to week. If someone rushes you to accept, that's your sign to cancel.",
            },
            Fact {
                keywords: &["lag", "laggy", "fps", "slow"],
                text: "Roblox lag: lower Graphics Quality (Esc → Settings), close background tabs, use a wired connection, and prefer servers close to you.",
            },
        ],
    },
    Topic {
        name: "Steam",
        keywords: &["steam", "valve"],
        facts: &[
            Fact {
                keywords: &["refund", "refunds", "money back"],
                text: "Steam refunds: under 2 hours played and within 14 days of purchase = automatic approval. The money goes to your Steam wallet or the original payment method.",
            },
            Fact {
                keywords: &["sale", "sales", "discount", "cheap", "wishlist"],
                text: "Big Steam sales hit in summer & winter (plus spring/autumn). Wishlist games now — Steam emails you the moment they drop in price.",
            },
            Fact {
                keywords: &["free games", "free game", "freebies"],
                text: "Steam has a 'Free to Play' section, and developers occasionally give games away to keep — follow Steam news or deal communities so you never miss one.",
            },
            Fact {
                keywords: &["cs2", "counter strike", "counter-strike"],
                text: "CS2 tip: practice recoil on ONE gun at a time in deathmatch, and learn a few smoke/flash lineups — utility wins more rounds than raw aim.",
            },
            Fact {
                keywords: &["terraria"],
                text: "Terraria progression: wood → ores → bosses. Build a few NPC houses early (they sell great stuff), and craft a Grappling Hook before deep cave exploring.",
            },
            Fact {
                keywords: &["tf2", "team fortress"],
                text: "TF2: the stock weapons are genuinely competitive — learn the fundamentals with stock before unlockables, and play Medic or Heavy to learn maps safely.",
            },
            Fact {
                keywords: &["deck", "controller"],
                text: "On Steam Deck: cap FPS at 40 for way better battery life, and check the 'Deck Verified' badge before buying a game.",
            },
            Fact {
                keywords: &["family", "sharing", "share games"],
                text: "Steam Families lets you share your library with family — but only one person can play a shared game at a time.",
            },
        ],
    },
    Topic {
        name: "Terraria",
        keywords: &["terraria"],
        facts: &[
            Fact {
                keywords: &["progression", "order", "bosses", "beat"],
                text: "Progression: pick a class (melee/ranged/magic/summon) and follow the boss ladder — King Slime → Eye of Cthulhu → Eater/Brain of Cthulhu → Skeletron → Queen Bee → Wall of Flesh, then the Hardmode mech bosses → Plantera → Golem → Moon Lord.",
            },
            Fact {
                keywords: &["wall of flesh", "hardmode", "voodoo"],
                text: "Entering Hardmode: kill the Wall of Flesh by tossing a Guide Voodoo Doll into Underworld lava. Build a LONG platform bridge across hell first — you need the running room, and the Hungry will eat you without it.",
            },
            Fact {
                keywords: &["hardmode", "ores", "cobalt", "mythril", "adamantite", "chlorophyte"],
                text: "Hardmode ore ladder: smash Demon Altars with the Pwnhammer to spawn Cobalt/Palladium → Mythril/Orichalcum → Adamantite/Titanium. After mech bosses, Chlorophyte grows in Jungle mud — leave jungle mud intact for it.",
            },
            Fact {
                keywords: &["npc", "house", "homes", "housing"],
                text: "NPCs move into valid housing: an enclosed 30+ block room with a light source, a flat-surface item (table), a comfort item (chair), and an entrance. Neighboring NPCs who like each other = happier = cheaper shop prices.",
            },
            Fact {
                keywords: &["hook", "grapple", "grappling", "mobility", "boots"],
                text: "Mobility wins runs: grab Hermes Boots + Cloud in a Bottle from surface chests early, and craft a Grappling Hook as soon as you can (hook + 3 chains). A hook saves you in caves AND dodges boss dashes.",
            },
            Fact {
                keywords: &["fishing", "crates", "angler"],
                text: "Fishing is underrated money: crates give free ores/potions, the Angler hands out daily rewards, and Sonar Potions stop you wasting bait on junk fish. Fish during Blood Moons for rare drops.",
            },
            Fact {
                keywords: &["invasion", "blood moon", "eclipse", "goblin", "pirate"],
                text: "Events are loot piñatas if you prepare: wall off doors during Blood Moons, and build a lava-trap kill corridor with your summons out for Goblin/Pirate invasions — the drops (Shadowflame weapons, discount cards) are worth it.",
            },
            Fact {
                keywords: &["moon lord", "final boss", "endgame"],
                text: "Moon Lord: build a boxed arena with Heart Lanterns + Campfires, fight ranged or summon so his laser eyes can't corner you, and keep the Nurse nearby to heal between phases. Beat him and you've 'beaten' Terraria — then the fun really starts.",
            },
        ],
    },
    Topic {
        name: "Palworld",
        keywords: &["palworld", "pals"],
        facts: &[
            Fact {
                keywords: &["base", "build", "territory", "where"],
                text: "First base: flat land near a Fast Travel point and water. Pals auto-assign to jobs matching their work suitability — Kindling Pals smelt, Planting Pals farm, Handiwork Pals craft. A base that runs itself is the whole game.",
            },
            Fact {
                keywords: &["breeding", "breed"],
                text: "Breeding is the real endgame: passive skills pass to offspring and stats combine, so a bred Pal beats any wild catch. Build a Breeding Farm, keep it stocked with cake, and lock in strong passives.",
            },
            Fact {
                keywords: &["condensation", "condense", "10"],
                text: "Catching 10 of the same Pal isn't hoarding — Pal Condensation at the Statue of Power fuses them into one permanently stronger Pal. Batch-catch commons early; the stat boost stacks forever.",
            },
            Fact {
                keywords: &["sanity", "food", "feed", "sick", "stress"],
                text: "Keep a Feed Box stocked and beds for every worker. Low-sanity Pals get stressed, sick, and lazy — rest plus better food (not just raw berries) restores them. A sane base is a fast base.",
            },
            Fact {
                keywords: &["raid", "defense", "defend", "attack"],
                text: "Your base WILL get raided: wall the perimeter, funnel attackers through gates, and station combat Pals with defensive passives inside. Keep your best fighters home when you log off.",
            },
            Fact {
                keywords: &["types", "element", "advantage", "fire", "water"],
                text: "Elements decide tough fights: Fire beats Grass & Ice, Water beats Fire, Grass beats Ground & Electric... bring counters to tower bosses and swap Pals mid-fight like Pokémon — type advantage is free damage.",
            },
            Fact {
                keywords: &["spheres", "catch", "legendary"],
                text: "Better Pals need better spheres — upgrade the Sphere Workbench early and carry a mix. Weaken targets first: low HP + status makes Legendary Spheres actually stick on alphas and legendaries.",
            },
            Fact {
                keywords: &["tower", "bosses", "syndicate"],
                text: "Tower bosses are scripted syndicate-leader fights, not catches — you can't pocket them. Bring type counters, swap Pals to dodge cooldowns, and use the arena terrain; they're skill checks, not stat checks.",
            },
        ],
    },
    Topic {
        name: "Pokémon",
        keywords: &["pokemon", "pokémon", "pokedex"],
        facts: &[
            Fact {
                keywords: &["type chart", "types", "weakness", "weaknesses", "super effective"],
                text: "The core triangle is Fire > Grass > Water > Fire, but immunities win more games than weaknesses: Ground is immune to Electric, Flying to Ground, Normal to Ghost and Ghost to Normal. Learn the immunities first.",
            },
            Fact {
                keywords: &["evs", "ev", "effort values", "training"],
                text: "EVs: 510 total per Pokémon, max 252 in one stat, and 4 EVs = +1 point at level 100. Vitamins add EVs fast, and EV-reducing berries undo mistakes. Decide the role (speed? bulk?) before you train.",
            },
            Fact {
                keywords: &["ivs", "iv", "breeding", "destiny knot", "nature"],
                text: "IV breeding: give a parent a Destiny Knot (passes 5 IVs) and an Everstone (locks nature). Hatch with a Flame Body Pokémon in your party to halve egg steps. Perfect IVs are a grind — Masuda-method shinies and comp teams come from the same nursery.",
            },
            Fact {
                keywords: &["catch", "catching", "quick ball", "dusk ball"],
                text: "Catching math is low HP + status + right ball: Quick Balls (turn 1), Dusk Balls (night/caves), Net Balls (water/bug). False Swipe guarantees you never KO the target by accident.",
            },
            Fact {
                keywords: &["shiny", "shinies", "masuda", "odds"],
                text: "Shiny odds: 1/4096 base. Masuda Method (breed with a foreign-language Pokémon) drops it to ~1/683, and adding the Shiny Charm gets ~1/512. Modern games stack outbreaks/sandwich powers way beyond that.",
            },
            Fact {
                keywords: &["team", "competitive", "team building", "sweeper"],
                text: "Team-building 101: every squad needs a fast lead, a defensive wall, and a setup sweeper. Cover your type weaknesses across six slots, and respect entry hazards (Stealth Rock) — chip damage wins long games.",
            },
            Fact {
                keywords: &["starter", "starters", "choose"],
                text: "In-game, your starter barely matters — every story is beatable with a team you love. Pick on vibes; optimize later. In competitive, choose based on what your team is missing, not what's popular.",
            },
        ],
    },
    Topic {
        name: "GTA",
        keywords: &["gta", "grand theft auto"],
        facts: &[
            Fact {
                keywords: &["money", "make money", "money fast", "cash"],
                text: "Online money: heists pay best and Cayo Perico remains the classic solo farm; stack it with an Agency, Terrorbyte client jobs, and a Nightclub for passive income while you're off doing anything else.",
            },
            Fact {
                keywords: &["businesses", "business", "nightclub", "bunker", "mc"],
                text: "Business ladder for steady income: Bunker (fully upgraded, buy supplies — never steal them, your time is worth more) → Nightclub (passively accumulates tied businesses' stock) → Vehicle Cargo (solo-friendly imports/exports).",
            },
            Fact {
                keywords: &["lester", "assassinations", "stock", "stocks", "story"],
                text: "Story mode's biggest money glitch-that-isn't: save ALL of Lester's assassination missions until AFTER the final heist, then invest each character per his company hints before and after the hit — returns in the hundreds of percent.",
            },
            Fact {
                keywords: &["properties", "property", "rent"],
                text: "In story mode, properties pay weekly: the Sonar Collections Dock and the two cinemas are the best passive earners. Online, only buy a property when a business inside it fits your money loop.",
            },
            Fact {
                keywords: &["wanted", "cops", "stars", "police"],
                text: "Losing wanted levels: line-of-sight is everything — break sight, then stay off their radar cone while the search decays. Pay 'n' Sprays clear it instantly, and subway tunnels confuse pursuit pathing.",
            },
            Fact {
                keywords: &["casino", "heist", "diamond", "prep"],
                text: "Diamond Casino Heist: scope EVERY access point and take time on prep — the approach choice (Big Con / Aggressive / Silent & Sneaky) decides your run. A practiced duo on Hard outearns a messy crew of four.",
            },
        ],
    },
    Topic {
        name: "Stardew Valley",
        keywords: &["stardew", "stardew valley"],
        facts: &[
            Fact {
                keywords: &["crops", "seasonal", "profit", "best crops", "crop profit"],
                text: "Seasonal profit plays: buy Strawberry seeds at the Egg Festival (Spring 13), plant Blueberries all Summer, Cranberries all Fall — regrowing crops compound your money. Keep a couple of each crop for the Community Center.",
            },
            Fact {
                keywords: &["sprinklers", "sprinkler", "watering"],
                text: "Sprinklers end the watering grind: Basic at Farming 3 (4 adjacent tiles), Quality at 6 (8 tiles), Iridium at 9 (24 tiles). Until then, upgrade the watering can in Winter when crops don't need you.",
            },
            Fact {
                keywords: &["fishing", "fish", "early"],
                text: "Fishing is the best Year 1 money: lake and ocean fish sell high (catfish after rain!), it costs nothing but time, and you level while crops grow. Eat field snacks to keep casting when energy runs dry.",
            },
            Fact {
                keywords: &["community center", "bundles", "bundle", "joja"],
                text: "Check the Community Center bundles early and stash one of everything (the crafting menu has the list). The Pantry restores the Greenhouse — winter-proof crops and the single biggest money unlock in the game.",
            },
            Fact {
                keywords: &["friendship", "gifts", "birthday", "birthdays"],
                text: "Talk to townsfolk daily, but never miss birthdays (calendar outside Pierre's): a loved gift on a birthday is worth weeks of daily chats. Two gifts a week is the cap — save them for loved items.",
            },
            Fact {
                keywords: &["animals", "barn", "coop", "mayonnaise"],
                text: "Processed beats raw: mayonnaise from chickens and cheese from cows far outsells the base products. Build a Silo BEFORE scything grass so it stocks as hay for winter feeding.",
            },
            Fact {
                keywords: &["mines", "skull cavern", "iridium", "desert"],
                text: "Iridium lives deep in Skull Cavern (desert, bring the skull key): go on high-luck days with bombs, staircase mats, and Spicy Eel. Skipping floors fast beats clearing them — depth is where iridium nodes live.",
            },
        ],
    },
    Topic {
        name: "Once Human",
        keywords: &["once human"],
        facts: &[
            Fact {
                keywords: &["seasons", "season reset", "season end", "season wipe", "wipe"],
                text: "The big one: Once Human runs SEASONS — at season end, characters and territory reset while you keep blueprints, cosmetics, and some progression. Don't over-invest in a base late in a season; the meta is knowing what carries over.",
            },
            Fact {
                keywords: &["territory", "base", "build", "move"],
                text: "Your territory can be moved for free — place it near the resources/missions you're working and relocate as you progress. Keep crafting benches, storage, and beds centralized so moving never breaks your loop.",
            },
            Fact {
                keywords: &["deviations", "deviation", "companion"],
                text: "Deviations are the heart of the game: combat deviations fight with you, while crafting/gathering deviations work your base while you're away. Level them up and assign the right deviation to the right job — a staffed base plays itself.",
            },
            Fact {
                keywords: &["contamination", "pollution", "stardust"],
                text: "Contaminated zones stack debuffs — carry cleansing supplies and gear with Stardust/contamination resistance before diving purple areas. Loot quality follows the danger, so prep before you push polluted territory.",
            },
            Fact {
                keywords: &["prime war", "events", "prime"],
                text: "Prime Wars and world events are the loot engine: rare mods and gear drop there that are painful to craft. They're group-tuned — squad up, and prioritize objectives over kills for the reward tier.",
            },
            Fact {
                keywords: &["weapons", "mods", "build", "gun"],
                text: "Gun builds live on status effects (Burn/Shatter/Burst): mod along ONE status line instead of mixing, and recalibrate gear toward crit or weakness damage. A focused build beats a spreadsheet of half-buffs every time.",
            },
        ],
    },
Topic {
        name: "Performance",
        keywords: &["lag", "fps", "ping", "stutter", "performance"],
        facts: &[
            Fact {
                keywords: &["lag", "fps", "stutter", "slow"],
                text: "Lag fix kit: lower the graphics preset first (biggest win), close background apps, update GPU drivers, and switch from Wi-Fi to Ethernet.",
            },
            Fact {
                keywords: &["ping", "delay", "latency"],
                text: "High ping? Use Ethernet over Wi-Fi, pick the closest server region, and pause downloads/updates while you play.",
            },
        ],
    },
    Topic {
        name: "Fortnite",
        keywords: &["fortnite", "fort", "battle royale", "building", "skins"],
        facts: &[
            Fact {
                keywords: &["building", "edit", "edits"],
                text: "Building: practice editing in Creative first. Build ramps instead of walls for mobility, edit while jumping to keep pressure, and always have a backup piece ready to edit back to.",
            },
            Fact {
                keywords: &["aim", "aiming", "sens", "sensitivity"],
                text: "Aim settings: start with 7-8% sensitivity on console, 0.08-0.12 on mouse. Build a little in Creative to find your sweet spot, then stick with it — constant changes hurt muscle memory.",
            },
            Fact {
                keywords: &["v-bucks", "vbucks", "free vbucks"],
                text: "V-Bucks: Daily Quests give the most steady income. Save up for battle passes (they pay for themselves), and never buy from 'free V-Bucks' sites — they're scams.",
            },
            Fact {
                keywords: &["meta", "best weapons", "meta weapons"],
                text: "The meta shifts each patch, but the shape holds: one long-range option (sniper/marksman), an AR for mid, and a shotgun/SMG for close. Learn whichever shotgun is the current close-range king.",
            },
            Fact {
                keywords: &["ninja", "trick shots", "advanced movement"],
                text: "Movement: slide into jumps to carry momentum, mantle ledges instead of wasting builds, and learn the 'double edit' technique for instant resets. Keep sprint-jumping while you edit to stay unpredictable.",
            },
        ],
    },
    Topic {
        name: "Zelda",
        keywords: &["zelda", "breath of the wild", "botw", "tears of the kingdom", "totk", "switch"],
        facts: &[
            Fact {
                keywords: &["korok seeds", "korok", "100 koroks"],
                text: "Korok Seeds: use the camera sensor to spot them (they make a chime). Prioritize ones near your path — you'll find 80+ just exploring naturally. Don't stress 100%!",
            },
            Fact {
                keywords: &["master sword", "sword", "durability"],
                text: "Master Sword: recharges after 10 minutes of use. Keep it at full hearts with a fairy or by cooking meals. For tough bosses, use ancient weapons instead.",
            },
            Fact {
                keywords: &["hinox", "titan", "boss"],
                text: "Hinox strategy: sneak up, climb onto its chest, and shoot the eye — eye shots stagger it and drop it for melee. Ancient arrows work too; ancient armor plus a strong bow makes farming them trivial.",
            },
            Fact {
                keywords: &["shrine", "shrines", "combat", "puzzle"],
                text: "Shrines: combat ones test your weapon/arrow skills. Puzzles lean on the runes in BotW (Magnesis, Stasis, Cryonis) and Ultrahand/Recall in TotK. Don't forget to check the ceiling and behind objects.",
            },
            Fact {
                keywords: &["tears of the kingdom", "totk", "ultrahand", "building"],
                text: "TOTK building: use Ultrahand to move large objects, then Fuse weapons/vehicles. Zonai devices + vehicles = infinite creativity. Start simple, then go wild!",
            },
        ],
    },
    Topic {
        name: "Elden Ring",
        keywords: &["elden ring", "elden", "souls", " FromSoftware", "boss"],
        facts: &[
            Fact {
                keywords: &["leveling", "stats", "build"],
                text: "Leveling priorities: Vigor > Mind > Endurance > your main damage stat. Level Vigor to 30+ for survivability. Mind gives more FP for spell usage.",
            },
            Fact {
                keywords: &["boss", "bosses", "difficult", "hard"],
                text: "Boss strategy: learn their tells, summon allies (Spirit Ashes), and use consumables. The Mimic Tear is OP for most bosses — just don't get hit by their grab attack.",
            },
            Fact {
                keywords: &["runes", "grace", "level up"],
                text: "Farming runes: beat bosses repeatedly for big payouts, and mix the Flask of Wondrous Physick with a Crimsonburst or Opaline Crystal Tear for hard fights. The Mimic Tear is the strongest all-round ash summon.",
            },
            Fact {
                keywords: &["armor", "gear", "best armor"],
                text: "Armor: prioritize defensive stats over fashion. Heavy armor has poise, light has dodge frames. The Bull-Goat set is great for new players — high defense and poise.",
            },
            Fact {
                keywords: &["magic", "incantations", "sorcery"],
                text: "Magic: Sorcery scales with Intelligence, Incantations with Faith. Carian spells are great for spacing. Comet Azur is OP but requires 60 Intelligence.",
            },
        ],
    },
];

const JOKES: &[&str] = &[
    "Why did the rocket go to school? To get a little brighter before launch! 🚀😄",
    "Why do Creepers never get invited to parties? They always blow up the plans 💥😄",
    "I'd tell you a lag joke… but you'd get it… eventually 🐌😄",
    "Why did the gamer bring a ladder to the boss fight? They heard the boss was *high* level 🪜😄",
];

pub const GREETING: &str = "Hi I'm Rocket, your personal Gaming AI, \
ask and I will answer with the best of my ability";

const TEACH_PROMPT: &str = "Good question — that one's not in my memory banks yet 🤔\n\
Teach me! Type the answer (or \"skip\") and I'll remember it from now on 🧠";

// ---------------------------------------------------------------------------
// Brain state (the part that learns)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Default)]
struct Persisted {
    learned: Vec<(String, String)>,
    name: Option<String>,
    favorite: Option<String>,
    topic_hits: Vec<(String, u32)>,
    custom_games: Vec<String>,
    answered: u32,
    taught: u32,
    corrections: u32,
}

#[derive(Default, Clone)]
pub struct Brain {
    /// Questions you taught ROCKET: normalized question → answer.
    learned: HashMap<String, String>,
    name: Option<String>,
    favorite: Option<String>,
    /// How often each topic comes up — used to show your favorite games.
    topic_hits: HashMap<String, u32>,
    /// Games the user introduced that aren't in the built-in knowledge base.
    custom_games: Vec<String>,
    answered: u32,
    taught: u32,
    corrections: u32,
    last_topic: Option<usize>,
    /// The last question ROCKET answered — the target for `wrong:` corrections.
    last_answered: Option<String>,
    fact_uses: HashMap<&'static str, u32>,
    joke_idx: u32,
}

const STORAGE_KEY: &str = "rocket.brain.v1";
const BRAIN_FILE: &str = "rocket-brain.json";

impl Brain {
    pub fn load_or_new() -> Brain {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            if let Ok(p) = LocalStorage::get::<Persisted>(STORAGE_KEY) {
                return Brain::from_persisted(p);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(bytes) = std::fs::read(BRAIN_FILE) {
                if let Ok(p) = serde_json::from_slice::<Persisted>(&bytes) {
                    return Brain::from_persisted(p);
                }
            }
        }
        Brain::default()
    }

    fn save(&self) {
        // Unit tests must never touch the user's real brain file on disk.
        if cfg!(test) {
            return;
        }
        let p = self.persisted();
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            let _ = LocalStorage::set(STORAGE_KEY, &p);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(bytes) = serde_json::to_vec(&p) {
                let _ = std::fs::write(BRAIN_FILE, bytes);
            }
        }
    }

    pub fn wipe(&mut self) {
        self.learned.clear();
        self.name = None;
        self.favorite = None;
        self.topic_hits.clear();
        self.custom_games.clear();
        self.answered = 0;
        self.taught = 0;
        self.corrections = 0;
        self.last_topic = None;
        self.last_answered = None;
        self.fact_uses.clear();
        self.joke_idx = 0;
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            LocalStorage::delete(STORAGE_KEY);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = std::fs::remove_file(BRAIN_FILE);
        }
    }

    pub fn game_count(&self) -> usize {
        self.custom_games.len()
    }

    pub fn memory_count(&self) -> usize {
        self.learned.len() + self.topic_hits.len()
    }

    fn persisted(&self) -> Persisted {
        Persisted {
            learned: self.learned.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            name: self.name.clone(),
            favorite: self.favorite.clone(),
            topic_hits: self.topic_hits.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            custom_games: self.custom_games.clone(),
            answered: self.answered,
            taught: self.taught,
            corrections: self.corrections,
        }
    }

    fn from_persisted(p: Persisted) -> Brain {
        Brain {
            learned: p.learned.into_iter().collect(),
            name: p.name,
            favorite: p.favorite,
            topic_hits: p.topic_hits.into_iter().collect(),
            custom_games: p.custom_games,
            answered: p.answered,
            taught: p.taught,
            corrections: p.corrections,
            last_topic: None,
            last_answered: None,
            fact_uses: HashMap::new(),
            joke_idx: 0,
        }
    }

    // -- public API used by the UI ------------------------------------------

    pub fn greeting(&self) -> String {
        match &self.name {
            Some(n) => format!(
                "Welcome back, {}! 🚀 Ready for more Minecraft, Roblox and Steam questions — \
                 my memory banks kept everything you taught me.",
                n
            ),
            None => GREETING.to_string(),
        }
    }

    pub fn teach(&mut self, question: &str, answer: &str) {
        self.learned.insert(normalize(question), answer.trim().to_string());
        self.taught += 1;
        self.answered += 1;
        self.save();
    }

    /// Name of the user, if learned (used by the online AI prompt).
    pub fn user_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The user's favorite game, if learned.
    pub fn user_favorite(&self) -> Option<&str> {
        self.favorite.as_deref()
    }

    /// Up to `limit` taught facts (question, answer) — the user's personal
    /// knowledge that gets injected into the online AI's instructions.
    pub fn learned_facts(&self, limit: usize) -> Vec<(String, String)> {
        self.learned
            .iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn stats_line(&self) -> String {
        let mut s = format!(
            "🧠 {} memories · 💬 {} answered · 🔥 top topic: {}",
            self.taught,
            self.answered,
            self.top_topic().unwrap_or("—")
        );
        if self.corrections > 0 {
            s.push_str(&format!(" · 🔧 {} corrections", self.corrections));
        }
        if !self.custom_games.is_empty() {
            s.push_str(&format!(" · 🕹️ {} custom games", self.custom_games.len()));
        }
        if let Some(f) = &self.favorite {
            s.push_str(&format!(" · 🎮 favorite: {}", f));
        }
        s
    }

    fn top_topic(&self) -> Option<&str> {
        self.topic_hits
            .iter()
            .max_by_key(|(_, c)| **c)
            .map(|(k, _)| k.as_str())
    }

    /// Main entry point: turn a raw user message into a reply.
    pub fn answer(&mut self, raw: &str) -> Reply {
        let q = normalize(raw);

        // Corrections come first — they overwrite the last answered fact.
        if let Some(reply) = self.try_correction(&q) {
            return reply;
        }

        if [
            "more", "another", "next", "another tip", "next tip", "tell me more", "more tips",
        ]
        .contains(&q.as_str())
        {
            return self.follow_up();
        }

        if let Some(reply) = self.smalltalk(&q) {
            return reply;
        }

        if let Some(reply) = self.try_captures(&q) {
            return reply;
        }

        // Learned answers are matched FIRST — that's the "faster responses" part.
        // (This also means corrected answers override the built-in facts.)
        if let Some(reply) = self.recall_learned(&q) {
            self.answered += 1;
            self.last_answered = Some(q.clone());
            self.save();
            return reply;
        }

        self.knowledge(&q)
    }

    /// "wrong: <correct answer>" — permanently fixes the last answered fact.
    fn try_correction(&mut self, q: &str) -> Option<Reply> {
        const PREFIXES: &[&str] = &[
            "wrong:", "wrong ", "correction:", "correction ", "thats wrong",
            "that s wrong", "that is wrong", "thats not right",
        ];
        let rest = PREFIXES
            .iter()
            .find_map(|p| q.strip_prefix(p).map(|r| r.trim().trim_start_matches(':').trim()))
            .filter(|r| !r.is_empty());

        let correction = match rest {
            Some(c) => c.to_string(),
            None => return None, // not a correction — normal flow continues
        };

        match self.last_answered.clone() {
            Some(question) => {
                self.learned.insert(question.clone(), correction.to_string());
                self.corrections += 1;
                self.save();
                Some(Reply {
                    text: format!(
                        "You're right — thanks for the correction 🙏 I had that wrong.\n\
                         I've updated my memory for \"{}\" permanently: it now says \"{}\".\n\
                         Ask me again and you'll get the fixed answer first, every time.",
                        question, correction
                    ),
                    tag: Tag::JustLearned,
                })
            }
            None => Some(Reply {
                text: "I don't have a recent answer to correct 🤔 — ask me a question first, \
                       then tell me what I got wrong and I'll lock in the fix."
                    .to_string(),
                tag: Tag::SmallTalk,
            }),
        }
    }

    // -- reply paths ---------------------------------------------------------

    fn follow_up(&mut self) -> Reply {
        let ti = match self.last_topic {
            Some(ti) => ti,
            None => {
                return Reply {
                    text: "I need a topic first — ask me about Minecraft, Roblox or a Steam \
                           game, then say \"more\" 🚀"
                        .to_string(),
                    tag: Tag::SmallTalk,
                }
            }
        };
        let topic = &TOPICS[ti];
        // Serve the least-recently-used fact so tips keep rotating.
        let fact = topic
            .facts
            .iter()
            .min_by_key(|f| self.fact_uses.get(f.text).copied().unwrap_or(0))
            .expect("topics always have facts");
        *self.fact_uses.entry(fact.text).or_insert(0) += 1;
        *self.topic_hits.entry(topic.name.to_string()).or_insert(0) += 1;
        self.answered += 1;
        self.save();
        Reply {
            text: fact.text.to_string(),
            tag: Tag::FollowUp,
        }
    }

    fn smalltalk(&mut self, q: &str) -> Option<Reply> {
        let name = self
            .name
            .as_deref()
            .map(|n| format!(", {}", n))
            .unwrap_or_default();

        const GREETINGS: &[&str] = &[
            "hi", "hello", "hey", "yo", "sup", "hiya", "hola", "good morning", "good evening",
            "hey rocket", "hi rocket", "hello rocket",
        ];
        if GREETINGS.contains(&q)
            || (q.len() <= 12
                && ["hello", "hey", "hi ", "yo ", "sup", "hiya", "hola"]
                    .iter()
                    .any(|p| q.starts_with(p)))
        {
            return Some(Reply {
                text: format!("Hey{}! 🚀 What game are we diving into today?", name),
                tag: Tag::SmallTalk,
            });
        }

        if ["who are you", "what are you", "whats your name", "what is your name", "your name",
            "who is rocket", "what is rocket"]
            .contains(&q)
        {
            return Some(Reply {
                text: "I'm ROCKET 🚀 — a Rust-powered personal gaming AI. I run on Iced right \
                       in your browser, and I learn from every chat we have."
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["help", "what can you do", "what do you know", "commands", "what can i ask",
            "what can you help with", "capabilities"]
            .contains(&q)
        {
            return Some(Reply {
                text: "I can do a lot! 🚀\n\
                       🎮 Answer questions about 10+ games — ask \"what games do you know\"\n\
                       💡 Give tips — ask, then say \"more\" for another\n\
                       🧠 Learn from you — if I don't know it, teach me once and it's saved forever\n\
                       🔧 Accept corrections — say \"wrong: <correct answer>\" and I permanently fix it\n\
                       🕹️ Add games — say \"add game <name>\" and teach me about it\n\
                       ⚡ Recall instantly — learned & corrected answers are checked FIRST\n\
                       📈 Track your favorite topics — check the stat bar below!"
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["what games do you know", "which games do you know", "what games you know",
            "what games", "which games", "games you know", "list games", "your games"]
            .contains(&q)
        {
            let builtin: Vec<&str> = TOPICS
                .iter()
                .filter(|t| t.name != "Performance")
                .map(|t| t.name)
                .collect();
            let mut text = format!(
                "🎮 I have built-in knowledge of: {}.\n",
                builtin.join(", ")
            );
            if self.custom_games.is_empty() {
                text.push_str(
                    "Plus any game YOU introduce — say \"add game <name>\" and teach me; \
                     I'll remember everything permanently.",
                );
            } else {
                text.push_str(&format!(
                    "Plus your games: {} — ask me anything about them, and teach me what \
                     I don't know yet.",
                    self.custom_games.join(", ")
                ));
            }
            return Some(Reply {
                text,
                tag: Tag::SmallTalk,
            });
        }

        if ["how are you", "how are you doing", "how is it going", "hows it going", "you good",
            "how do you feel"]
            .contains(&q)
        {
            return Some(Reply {
                text: "Running at full thrust 💪 — every question you ask makes my memory \
                       banks stronger. How are YOU doing?"
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["thanks", "thank you", "thx", "ty", "appreciate it", "thanks rocket"].contains(&q) {
            return Some(Reply {
                text: format!("Any time{}! That's what I'm here for 🚀", name),
                tag: Tag::SmallTalk,
            });
        }

        if ["bye", "goodbye", "cya", "see you", "see ya", "good night", "gn", "later"]
            .contains(&q)
        {
            return Some(Reply {
                text: format!("See you{} next launch! 👋🚀", name),
                tag: Tag::SmallTalk,
            });
        }

        if ["who made you", "who created you", "who built you", "who programmed you", "your creator"]
            .contains(&q)
        {
            return Some(Reply {
                text: "I was built with Rust + the Iced GUI framework, compiled to \
                       WebAssembly so I can live right here in your browser. My human \
                       teammate keeps training me 😄"
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["are you ai", "are you an ai", "are you a robot", "are you real", "are you human",
            "are you alive"]
            .contains(&q)
        {
            return Some(Reply {
                text: "I'm an AI assistant with a rocket body 🤖🚀 — 100% local, 100% Rust, \
                       and I get smarter every time we talk."
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["joke", "tell me a joke", "make me laugh", "say something funny", "funny"].contains(&q)
        {
            let joke = JOKES[(self.joke_idx as usize) % JOKES.len()];
            self.joke_idx += 1;
            return Some(Reply {
                text: joke.to_string(),
                tag: Tag::SmallTalk,
            });
        }

        if ["what did you learn", "what do you remember", "your memories",
            "what do you know about me", "memory", "what have you learned"]
            .contains(&q)
        {
            return Some(Reply {
                text: self.memory_dump(),
                tag: Tag::SmallTalk,
            });
        }

        if ["forget me", "forget everything", "reset", "wipe memory", "clear memory"].contains(&q) {
            return Some(Reply {
                text: "Use the 🧠 reset button in the top-right corner — it wipes my memory \
                       banks clean (my built-in gaming knowledge stays)."
                    .to_string(),
                tag: Tag::SmallTalk,
            });
        }

        None
    }

    fn memory_dump(&self) -> String {
        let mut lines = vec!["🧠 My memory banks:".to_string()];
        if let Some(n) = &self.name {
            lines.push(format!("• Your name: {}", n));
        }
        if let Some(f) = &self.favorite {
            lines.push(format!("• Favorite game: {}", f));
        }
        lines.push(format!("• {} things you taught me", self.taught));
        lines.push(format!("• {} questions answered so far", self.answered));
        if let Some(t) = self.top_topic() {
            lines.push(format!("• Topic you ask about most: {}", t));
        }
        if !self.learned.is_empty() {
            lines.push("• Things I remember:".into());
            for (i, (k, _)) in self.learned.iter().take(4).enumerate() {
                lines.push(format!("  {}. \"{}\"", i + 1, k));
            }
            if self.learned.len() > 4 {
                lines.push(format!("  … and {} more", self.learned.len() - 4));
            }
        }
        lines.join("\n")
    }

    /// Learns personal facts ("my name is…", "my favorite game is…", "add game…").
    fn try_captures(&mut self, q: &str) -> Option<Reply> {
        for prefix in ["add game", "new game", "lets talk about", "let s talk about"] {
            if let Some(rest) = q.strip_prefix(prefix) {
                let name = rest.trim().trim_end_matches('.').trim();
                if name.is_empty() || name.len() > 30 || name.contains(" and ") {
                    continue;
                }
                let game = capitalize_first(name);
                // Skip if it's already a built-in topic or already registered.
                let builtin = TOPICS
                    .iter()
                    .any(|t| t.name.to_lowercase() == game.to_lowercase());
                if builtin || self.custom_games.iter().any(|g| g.eq_ignore_ascii_case(&game)) {
                    if builtin {
                        return Some(Reply {
                            text: format!(
                                "{} is already in my knowledge base 🎮 — just ask me about it!",
                                game
                            ),
                            tag: Tag::SmallTalk,
                        });
                    }
                    return Some(Reply {
                        text: format!("{} is already on my list 🎮 — ask away!", game),
                        tag: Tag::SmallTalk,
                    });
                }
                self.custom_games.push(game.clone());
                self.save();
                return Some(Reply {
                    text: format!(
                        "{} added to my game library! 🕹️🚀 I don't know much about it yet — \
                         ask me a question and teach me the answers. Every fact you teach me \
                         sticks permanently, and \"wrong: <answer>\" fixes my mistakes.",
                        game
                    ),
                    tag: Tag::JustLearned,
                });
            }
        }

        for prefix in ["my name is", "i am called", "call me", "im called"] {
            if let Some(rest) = q.strip_prefix(prefix) {
                let name = rest.trim().trim_end_matches('.').trim();
                if !name.is_empty() && name.len() <= 24 && !name.contains(" and ") {
                    let n = capitalize_first(name);
                    self.name = Some(n.clone());
                    self.save();
                    return Some(Reply {
                        text: format!("Nice to meet you, {}! 🚀 I'll remember that.", n),
                        tag: Tag::JustLearned,
                    });
                }
            }
        }
        for prefix in ["my favorite game is", "my favourite game is", "i like playing",
            "i love playing", "my favorite is"]
        {
            if let Some(rest) = q.strip_prefix(prefix) {
                let g = rest.trim().trim_end_matches('.').trim();
                if !g.is_empty() && g.len() <= 32 {
                    let game = capitalize_first(g);
                    self.favorite = Some(game.clone());
                    self.save();
                    return Some(Reply {
                        text: format!(
                            "{} — noted! 🎮 I'll keep that in mind whenever we talk games.",
                            game
                        ),
                        tag: Tag::JustLearned,
                    });
                }
            }
        }
        None
    }

    /// Checks taught answers first (the fast, learned path).
    fn recall_learned(&self, q: &str) -> Option<Reply> {
        if self.learned.is_empty() {
            return None;
        }
        let q_tokens = tokenize(q);
        if q_tokens.is_empty() {
            return None;
        }
        let mut best: Option<(f32, String)> = None;
        for (k, v) in &self.learned {
            let k_tokens = tokenize(k);
            if k_tokens.is_empty() {
                continue;
            }
            let inter = k_tokens
                .iter()
                .filter(|t| q_tokens.contains(t))
                .count() as f32;
            let sim = 2.0 * inter / (k_tokens.len() + q_tokens.len()) as f32;
            if sim >= 0.6 && best.as_ref().map_or(true, |(s, _)| sim > *s) {
                best = Some((sim, v.clone()));
            }
        }
        best.map(|(_, v)| Reply {
            text: v,
            tag: Tag::Learned,
        })
    }

    /// Matches the question against the static knowledge base.
    fn knowledge(&mut self, q: &str) -> Reply {
        let mut best: Option<(u32, u32, usize, usize)> = None; // (score, uses, topic, fact)
        let mut topic_matched: Option<(u32, usize)> = None;
        let tokens = tokenize(q);

        for (ti, topic) in TOPICS.iter().enumerate() {
            let t_hits = topic
                .keywords
                .iter()
                .filter(|kw| kw_matches(kw, q, &tokens))
                .count() as u32;
            if t_hits > 0 {
                match topic_matched {
                    Some((h, _)) if h >= t_hits => {}
                    _ => topic_matched = Some((t_hits, ti)),
                }
                *self.topic_hits.entry(topic.name.to_string()).or_insert(0) += 1;
                self.last_topic = Some(ti);
            }
            for (fi, fact) in topic.facts.iter().enumerate() {
                let f_hits = fact
                    .keywords
                    .iter()
                    .filter(|kw| kw_matches(kw, q, &tokens))
                    .count() as u32;
                if f_hits == 0 {
                    continue;
                }
                let score = f_hits + 2 * t_hits; // topic mention boosts game-specific facts
                let uses = self.fact_uses.get(fact.text).copied().unwrap_or(0);
                let take = match best {
                    None => true,
                    Some((bs, bu, _, _)) => score > bs || (score == bs && uses < bu),
                };
                if take {
                    best = Some((score, uses, ti, fi));
                }
            }
        }

        if let Some((_, _, ti, fi)) = best {
            let fact = &TOPICS[ti].facts[fi];
            *self.fact_uses.entry(fact.text).or_insert(0) += 1;
            self.answered += 1;
            self.last_answered = Some(q.to_string());
            self.save();
            return Reply {
                text: fact.text.to_string(),
                tag: Tag::Knowledge,
            };
        }

        if let Some((_, ti)) = topic_matched {
            self.answered += 1;
            self.last_answered = Some(q.to_string());
            self.save();
            let hints: Vec<&str> = TOPICS[ti]
                .facts
                .iter()
                .map(|f| f.keywords[0])
                .collect();
            return Reply {
                text: format!(
                    "{}? Nice pick 🎮 Try asking me about: {}.\nOr just say \"more\" and \
                     I'll fire off a tip 🔥",
                    TOPICS[ti].name,
                    hints.join(", ")
                ),
                tag: Tag::Knowledge,
            };
        }

        Reply {
            text: TEACH_PROMPT.to_string(),
            tag: Tag::TeachMe,
        }
    }
}

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_alphanumeric() || c == ' ' {
            out.push(c);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

/// Multi-word keywords match as a phrase; single words must match whole
/// tokens (so "sea" doesn't fire on "season").
fn kw_matches(kw: &str, q: &str, tokens: &[String]) -> bool {
    if kw.contains(' ') {
        q.contains(kw)
    } else {
        tokens.iter().any(|t| t == kw)
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_punctuation_and_case() {
        assert_eq!(normalize("What's Netherite?!"), "what s netherite");
        assert_eq!(normalize("  HOW   do I find   diamonds  "), "how do i find diamonds");
    }

    #[test]
    fn answers_minecraft_from_knowledge() {
        let mut brain = Brain::default();
        let reply = brain.answer("How do I find diamonds in minecraft?");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("Y -58"), "should mention diamond level: {}", reply.text);
    }

    #[test]
    fn free_robux_warns_about_scams() {
        let mut brain = Brain::default();
        let reply = brain.answer("how do i get free robux");
        assert!(reply.text.to_lowercase().contains("scam"), "{}", reply.text);
    }

    #[test]
    fn topic_menu_when_only_game_mentioned() {
        let mut brain = Brain::default();
        let reply = brain.answer("roblox");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("Roblox"));
        // "more" should then rotate tips on that topic
        let follow = brain.answer("more");
        assert_eq!(follow.tag, Tag::FollowUp);
    }

    #[test]
    fn learns_and_recalls_instantly() {
        let mut brain = Brain::default();
        let reply = brain.answer("what is the best loadout in apex legends");
        assert_eq!(reply.tag, Tag::TeachMe);

        brain.teach("what is the best loadout in apex legends", "Wingman + R301, always");
        let recalled = brain.answer("best loadout in apex legends?");
        assert_eq!(recalled.tag, Tag::Learned);
        assert_eq!(recalled.text, "Wingman + R301, always");
    }

    #[test]
    fn learns_your_name_and_uses_it() {
        let mut brain = Brain::default();
        let reply = brain.answer("my name is wyatt");
        assert_eq!(reply.tag, Tag::JustLearned);
        assert!(reply.text.contains("Wyatt"));

        let hi = brain.answer("hi");
        assert!(hi.text.contains("Wyatt"), "greeting should use learned name: {}", hi.text);
        assert!(brain.greeting().contains("Wyatt"));
    }

    #[test]
    fn remembers_favorite_game() {
        let mut brain = Brain::default();
        brain.answer("my favorite game is terraria");
        assert_eq!(brain.favorite.as_deref(), Some("Terraria"));
        assert!(brain.stats_line().contains("Terraria"));
    }

    #[test]
    fn unknown_things_offer_teaching() {
        let mut brain = Brain::default();
        // Must dodge every built-in keyword pack — Fortnite ("meta" packs,
        // etc.) now route to Tag::Knowledge, so use an unknown topic.
        let reply = brain.answer("how do florphs respawn in blorptopia");
        assert_eq!(reply.tag, Tag::TeachMe);
        assert!(reply.text.contains("Teach me") || reply.text.contains("teach me"));
    }

    // -- new knowledge packs -------------------------------------------------

    #[test]
    fn knows_terraria_progression() {
        let mut brain = Brain::default();
        let reply = brain.answer("what is the boss progression order in terraria");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("Wall of Flesh"), "{}", reply.text);
    }

    #[test]
    fn knows_palworld_base_building() {
        let mut brain = Brain::default();
        let reply = brain.answer("where should i build my first base in palworld");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("Fast Travel"), "{}", reply.text);
    }

    #[test]
    fn knows_pokemon_evs() {
        let mut brain = Brain::default();
        let reply = brain.answer("how do evs work in pokemon");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("510"), "{}", reply.text);
    }

    #[test]
    fn knows_gta_money() {
        let mut brain = Brain::default();
        let reply = brain.answer("how do i make money fast in gta online");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.to_lowercase().contains("cayo"), "{}", reply.text);
    }

    #[test]
    fn knows_stardew_crops() {
        let mut brain = Brain::default();
        let reply = brain.answer("most profitable crops in stardew valley");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("Blueberries"), "{}", reply.text);
    }

    #[test]
    fn knows_once_human_seasons() {
        let mut brain = Brain::default();
        let reply = brain.answer("do seasons reset in once human");
        assert_eq!(reply.tag, Tag::Knowledge);
        assert!(reply.text.contains("SEASONS"), "{}", reply.text);
    }

    // -- corrections -----------------------------------------------------------

    #[test]
    fn corrections_overwrite_last_answer() {
        let mut brain = Brain::default();
        brain.answer("how do i find diamonds in minecraft"); // knowledge answer
        let fix = brain.answer("wrong: they removed diamonds in 1.21, sorry");
        assert_eq!(fix.tag, Tag::JustLearned);
        assert!(fix.text.contains("updated my memory"));

        // Ask again — the correction must win over the built-in fact.
        let after = brain.answer("how do i find diamonds in minecraft");
        assert_eq!(after.tag, Tag::Learned);
        assert!(after.text.contains("removed diamonds"), "{}", after.text);
        assert_eq!(brain.corrections, 1);
    }

    #[test]
    fn correction_without_question_explains() {
        let mut brain = Brain::default();
        let reply = brain.answer("wrong: the sky is green");
        assert_eq!(reply.tag, Tag::SmallTalk);
        assert!(reply.text.contains("ask me a question first"));
    }

    #[test]
    fn normal_questions_containing_wrong_are_not_corrections() {
        let mut brain = Brain::default();
        let reply = brain.answer("what am i doing wrong in terraria");
        assert_ne!(reply.text.contains("updated my memory"), true);
    }

    // -- game registry -----------------------------------------------------------

    #[test]
    fn what_games_do_you_know_lists_them() {
        let mut brain = Brain::default();
        let reply = brain.answer("what games do you know");
        for game in ["Minecraft", "Terraria", "Palworld", "Stardew", "Once Human"] {
            assert!(reply.text.contains(game), "missing {} in {}", game, reply.text);
        }
    }

    #[test]
    fn add_game_registers_custom_game() {
        let mut brain = Brain::default();
        // "Fortnite" is a built-in topic; a custom game must be one ROCKET
        // doesn't already know.
        let reply = brain.answer("add game helldivers 2");
        assert_eq!(reply.tag, Tag::JustLearned);
        assert!(reply.text.contains("Helldivers 2"));

        let list = brain.answer("what games do you know");
        assert!(list.text.contains("Helldivers 2"), "{}", list.text);
        assert!(brain.stats_line().contains("1 custom games"));
    }

    #[test]
    fn add_builtin_game_is_recognized() {
        let mut brain = Brain::default();
        let reply = brain.answer("add game terraria");
        assert!(reply.text.contains("already in my knowledge base"));
        assert!(brain.custom_games.is_empty());
    }
}
