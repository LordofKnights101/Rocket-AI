//! Rocket's personality — its "vibe".
//!
//! A vibe shapes HOW Rocket talks, not WHAT it knows: the greetings it gives
//! when you switch games, the way it accepts missions, how it celebrates
//! your wins, and the occasional banter it appends to answers. Pick a vibe
//! from the Profile page or just tell Rocket "be funny" in chat.

/// One personality. Every list needs at least one line.
pub struct Vibe {
    pub id: &'static str,
    /// Chip label shown on the Profile page.
    pub label: &'static str,
    /// Said when the game context switches ("co-pilot check-in").
    pub game_switch: &'static [&'static str],
    /// Occasional reaction above an answer (kept short).
    pub reactions: &'static [&'static str],
    /// Occasional sign-off after an answer.
    pub tails: &'static [&'static str],
    /// Said when a mission is accepted.
    pub mission_created: &'static [&'static str],
    /// Said when a mission is completed.
    pub mission_done: &'static [&'static str],
    /// Confirmation when this vibe is switched on.
    pub intro: &'static str,
}

pub const BUDDY: Vibe = Vibe {
    id: "buddy",
    label: "🤝 Buddy",
    game_switch: &[
        "🎮 {game} locked in — what are we tackling first?",
        "🎮 {game}? I'm in. Main quest, side content, or just vibes?",
        "🎮 Alright, {game} — you drive, I navigate. Where to?",
    ],
    reactions: &[
        "Good question — let's figure it out together:",
        "Ooh, good one:",
        "On it:",
    ],
    tails: &[
        "We've got this 🤝",
        "Ping me when you're stuck again — that's what I'm here for.",
        "If it goes sideways, we regroup and try the other way. That's the fun part.",
    ],
    mission_created: &[
        "🎯 Mission accepted: {mission}. Here's our opening move:",
        "🎯 On the board: {mission}. Let's break it down:",
        "🎯 Say less — {mission} is our mission now. Game plan:",
    ],
    mission_done: &[
        "🏆 WE DID IT — {mission} complete! That's why we team up.",
        "🏆 {mission}: done and dusted. Great teamwork!",
        "🏆 Mission complete — {mission}. Told you we'd get there!",
    ],
    intro: "🤝 Buddy mode on — I've got your back out there.",
};

pub const CHILL: Vibe = Vibe {
    id: "chill",
    label: "😎 Chill",
    game_switch: &[
        "😎 {game}, huh? Nice. No rush — where do you feel like starting?",
        "🎮 {game} it is. We'll take it slow and easy.",
        "😎 Cozy choice. {game} sessions are best without a speedrun clock anyway.",
    ],
    reactions: &[
        "Easy — here's the deal:",
        "No stress, it's simpler than it looks:",
        "Cool, so here's the thing:",
    ],
    tails: &[
        "No pressure — it'll click when it clicks 😎",
        "Take your time with it. The game isn't going anywhere.",
        "And hey, if today's not the day, tomorrow's save file will still be there.",
    ],
    mission_created: &[
        "🎯 {mission}, sure thing. No speedrun pressure — here's the relaxed route:",
        "🎯 Mission noted: {mission}. We'll get there at our own pace:",
        "😎 {mission}? Easy. One step at a time:",
    ],
    mission_done: &[
        "😎 {mission} — nice and easy. Told you there was no rush.",
        "🏆 {mission}: done. Smooth. Rewarding yourself with a break? Earned.",
        "😎 Clean run — {mission} complete. Chill victory.",
    ],
    intro: "😎 Chill mode on — we play it cool and we get there.",
};

pub const FUNNY: Vibe = Vibe {
    id: "funny",
    label: "😂 Funny",
    game_switch: &[
        "😂 {game}? Excellent choice. I'll carry — emotionally. You carry the loot.",
        "🎮 Welcome to {game}: where your plans are jokes and the physics are the punchline.",
        "😂 {game} session starting! Remember: any death you walk away from is content.",
    ],
    reactions: &[
        "Ha — okay okay, listen:",
        "Oh we're doing THIS today? Love it. Here:",
        "Buckle up, this answer has layers:",
    ],
    tails: &[
        "And if it all goes wrong — that's the game's fault, not mine 😌",
        "Side note: I typed that answer flawlessly. No typos. Just saying.",
        "If you die, avenge me. Wait. You're me. Avenge yourself.",
        "Fun fact: the NPC judging you is also judging your loadout.",
    ],
    mission_created: &[
        "🎯 Mission accepted: {mission}. I've informed the boss (you). Here's the plan:",
        "🎯 {mission} is on the board! The devs don't know what's coming:",
        "🎯 {mission}? Bold. I respect it. Phase one:",
    ],
    mission_done: &[
        "🏆 {mission} COMPLETE! Somewhere, a game developer just felt a disturbance.",
        "🏆 {mission}: done! The final boss would like a rematch. The answer is no.",
        "🏆 We actually did {mission}. I'd like to thank the guide, and me, for writing it.",
    ],
    intro: "😂 Funny mode on — gameplay advice 90% quality, 10% nonsense, 100% fun.",
};

pub const HYPED: Vibe = Vibe {
    id: "hyped",
    label: "🔥 Hyped",
    game_switch: &[
        "🔥 {game}!! LET'S GOOO — what are we destroying first?!",
        "🎮 {game} LET'S GO! Main quest? Side quests? ALL of it?",
        "🔥 Yo {game}!! I'm SO ready for this session!",
    ],
    reactions: &[
        "OOOH okay okay here we go:",
        "THIS is what I'm talking about:",
        "LET'S GO — here's everything you need:",
    ],
    tails: &[
        "NOW GO MAKE IT LOOK EASY 🔥",
        "That's the secret sauce — NOW COOK 🔥",
        "You're gonna crush this. I can FEEL it 🔥",
    ],
    mission_created: &[
        "🔥 MISSION ACCEPTED: {mission}!! Here's the opening play:",
        "🔥 {mission}?! SAY LESS. We're doing this RIGHT NOW. Step one:",
        "🔥 Adding {mission} to the board — LET'S BREAK IT DOWN:",
    ],
    mission_done: &[
        "🏆 {mission}!!! LET'S GOOOO!! Absolute unit of a performance!",
        "🔥 {mission} = DONE. I'm screaming. That was CINEMA!",
        "🏆 MISSION COMPLETE: {mission}!! Name a better duo. I'll wait.",
    ],
    intro: "🔥 HYPED MODE ON — LET'S GOOOO!!",
};

pub const MENTOR: Vibe = Vibe {
    id: "mentor",
    label: "🧙 Mentor",
    game_switch: &[
        "🧙 {game}, wise choice. Tell me — what are we mastering today?",
        "🎮 {game}. Every session is a lesson if you're paying attention. What's the topic?",
        "🧙 Welcome to {game}. I'll teach; you execute. Deal?",
    ],
    reactions: &[
        "A worthy question. Observe:",
        "Listen closely — this is how it's done:",
        "Ah, a classic sticking point. Here's the teaching:",
    ],
    tails: &[
        "Practice it once in a safe fight before you need it for real. Mastery is repetition.",
        "Remember: knowledge applied is skill. Go apply it.",
        "When you understand WHY this works, you won't need my advice anymore. …But keep me around anyway 🧙",
    ],
    mission_created: &[
        "🎯 Mission accepted: {mission}. Lesson one:",
        "🎯 {mission} — a fine goal. Your training begins now:",
        "🧙 {mission}. We break it into teachable steps:",
    ],
    mission_done: &[
        "🏆 {mission} — completed. The student becomes formidable. I'm proud.",
        "🏆 {mission}: done. That wasn't luck — that was everything you learned paying off.",
        "🏆 Mission complete: {mission}. Note this feeling. THIS is why we practice.",
    ],
    intro: "🧙 Mentor mode on — patience, practice, and we beat anything.",
};

/// Every vibe Rocket can wear.
pub const VIBES: &[Vibe] = &[BUDDY, CHILL, FUNNY, HYPED, MENTOR];

pub const DEFAULT_VIBE_ID: &str = "buddy";

/// Random line from a list (lists are non-empty by construction).
pub fn line<'a>(list: &'a [&'static str]) -> &'a str {
    list[rand::random::<usize>() % list.len()]
}

pub fn vibe_by_id(id: &str) -> &'static Vibe {
    VIBES
        .iter()
        .find(|v| v.id == id)
        .unwrap_or(&BUDDY)
}

/// Fill a {game} or {mission} placeholder in a personality line.
pub fn fill(line_text: &str, placeholder: &str, value: &str) -> String {
    line_text.replace(placeholder, value)
}

/// Parse a vibe-switch request ("be funny", "act chill", "vibe: mentor").
/// Returns the vibe id when the message is a vibe command.
pub fn parse_vibe_command(message: &str) -> Option<&'static str> {
    let lower = message.trim().to_lowercase();
    let stripped = [
        "be ", "act ", "go ", "switch to ", "turn on ", "vibe ", "set vibe to ",
        "set your vibe to ", "you can be ",
    ]
    .iter()
    .find_map(|p| lower.strip_prefix(p))?;

    let stripped = stripped.trim().trim_end_matches(['.', '!', '?']).trim();
    let id = match stripped {
        "buddy" | "friend" | "normal" | "default" | "buddy mode" => Some("buddy"),
        "chill" | "chill mode" | "relaxed" | "calm" => Some("chill"),
        "funny" | "funny mode" | "sarcastic" | "silly" | "joke mode" => Some("funny"),
        "hyped" | "hyped mode" | "hype" | "energetic" | "excited" => Some("hyped"),
        "mentor" | "mentor mode" | "wise" | "teacher" | "sensei" => Some("mentor"),
        _ => None,
    }?;

    // Don't hijack questions ("be funny?" is fine to treat as a command).
    if lower.contains("how") || lower.contains("why") || lower.contains("what is") {
        return None;
    }
    Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_vibe_is_complete_and_unique() {
        assert!(VIBES.len() >= 5, "expected at least 5 vibes");
        let mut ids: Vec<&str> = VIBES.iter().map(|v| v.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), VIBES.len(), "vibe ids must be unique");
        for v in VIBES {
            assert!(!v.game_switch.is_empty(), "{} has no game_switch lines", v.id);
            assert!(!v.reactions.is_empty(), "{} has no reactions", v.id);
            assert!(!v.tails.is_empty(), "{} has no tails", v.id);
            assert!(!v.mission_created.is_empty(), "{} has no mission_created", v.id);
            assert!(!v.mission_done.is_empty(), "{} has no mission_done", v.id);
            assert!(!v.intro.is_empty(), "{} has no intro", v.id);
            assert!(v.game_switch[0].contains("{game}"), "{} game_switch must use the game placeholder", v.id);
            assert!(v.mission_created[0].contains("{mission}"), "{} mission_created must use the mission placeholder", v.id);
            assert!(v.mission_done[0].contains("{mission}"), "{} mission_done must use the mission placeholder", v.id);
        }
    }

    #[test]
    fn vibe_lookup_falls_back_to_buddy() {
        assert_eq!(vibe_by_id("funny").id, "funny");
        assert_eq!(vibe_by_id("nonsense").id, "buddy");
    }

    #[test]
    fn vibe_commands_parse() {
        assert_eq!(parse_vibe_command("be funny"), Some("funny"));
        assert_eq!(parse_vibe_command("Be Chill"), Some("chill"));
        assert_eq!(parse_vibe_command("act like a mentor"), None); // not a prefix match
        assert_eq!(parse_vibe_command("switch to hyped!"), Some("hyped"));
        assert_eq!(parse_vibe_command("vibe mentor"), Some("mentor"));
        assert_eq!(parse_vibe_command("be sarcastic"), Some("funny"));
        // Questions must not be hijacked.
        assert_eq!(parse_vibe_command("how can i be funny"), None);
        // Non-commands return None.
        assert_eq!(parse_vibe_command("im going for a mage build"), None);
        assert_eq!(parse_vibe_command("be honest, is skyrim good"), None);
    }

    #[test]
    fn fill_replaces_placeholders() {
        assert_eq!(fill("🎮 {game} time", "{game}", "Skyrim"), "🎮 Skyrim time");
        assert_eq!(fill("{mission} done", "{mission}", "beat the dragon"), "beat the dragon done");
    }
}
