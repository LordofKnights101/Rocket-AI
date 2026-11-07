//! Auto game detection — watches running processes and matches known game
//! executables so ROCKET switches context the moment you launch a game.
//!
//! Windows: uses the built-in `tasklist` CSV output (no extra crates).
//! Other platforms: returns `None` (manual selection still works).

use crate::GameType;

const PREFS_FILE: &str = "rocket_detect.json";

/// Is auto-detection enabled? (Defaults to ON.)
pub fn load_enabled() -> bool {
    std::fs::read(PREFS_FILE)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|v| v["enabled"].as_bool())
        .unwrap_or(true)
}

/// Persist the auto-detection toggle.
pub fn save_enabled(enabled: bool) {
    let _ = std::fs::write(
        PREFS_FILE,
        serde_json::json!({ "enabled": enabled }).to_string(),
    );
}

/// Known game executables → ROCKET game. Order matters when several run
/// at once (e.g. clients + the game itself map to the same game anyway).
/// Known game executables → ROCKET game (public for docs_gen).
pub const GAME_PROCESSES: &[(&str, GameType)] = &[
    ("robloxplayerbeta.exe", GameType::Roblox),
    ("minecraftlauncher.exe", GameType::Minecraft),
    ("minecraft.exe", GameType::Minecraft),
    ("valorant-win64.exe", GameType::Valorant),
    ("valorant.exe", GameType::Valorant),
    ("cs2.exe", GameType::CS2),
    ("fortniteclient-win64-shipping.exe", GameType::Fortnite),
    ("cod.exe", GameType::CallOfDuty),
    ("call of duty.exe", GameType::CallOfDuty),
    ("apexlegends.exe", GameType::ApexLegends),
    ("tslgame.exe", GameType::PUBG),
    ("rocketleague.exe", GameType::RocketLeague),
    ("thesims4.exe", GameType::TheSims),
    ("stardew valley.exe", GameType::StardewValley),
    ("skyrimse.exe", GameType::Skyrim),
    ("skyrim.exe", GameType::Skyrim),
    ("gta5.exe", GameType::GTA),
    ("gta5enhanced.exe", GameType::GTA),
    ("among us.exe", GameType::AmongUs),
    ("fallguys_client.exe", GameType::FallGuys),
    ("wow.exe", GameType::WorldOfWarcraft),
    ("wowclassic.exe", GameType::WorldOfWarcraft),
    ("dota2.exe", GameType::Dota2),
    ("league of legends.exe", GameType::LeagueOfLegends),
    ("leagueclient.exe", GameType::LeagueOfLegends),
    // ---- newer catalog entries with known executables ----
    ("eldenring.exe", GameType::EldenRing),
    ("witcher3.exe", GameType::Witcher3),
    ("valheim.exe", GameType::Valheim),
    ("terraria.exe", GameType::Terraria),
    ("rustclient.exe", GameType::Rust),
    ("shootergame.exe", GameType::ARK),
    ("overwatch.exe", GameType::Overwatch),
    ("destiny2.exe", GameType::Destiny2),
    ("haloinfinite.exe", GameType::Halo),
    ("cyberpunk2077.exe", GameType::Cyberpunk),
    ("bg3_dx11.exe", GameType::BaldursGate3),
    ("bg3.exe", GameType::BaldursGate3),
    ("hogwartslegacy.exe", GameType::Hogwarts),
    ("palworld-win64-shipping.exe", GameType::Palworld),
    ("palworld.exe", GameType::Palworld),
    ("phasmophobia.exe", GameType::Phasmophobia),
    ("sekiro.exe", GameType::Sekiro),
    ("rdr2.exe", GameType::RDR2),
    ("monsterhunterworld.exe", GameType::MonsterHunter),
    ("ffxiv_dx11.exe", GameType::FinalFantasy14),
    ("ffxiv.exe", GameType::FinalFantasy14),
];

/// Match a single process image name (e.g. "cs2.exe") to a game.
pub fn match_process(image_name: &str) -> Option<GameType> {
    let name = image_name.trim().to_lowercase();
    GAME_PROCESSES
        .iter()
        .find(|(proc_name, _)| *proc_name == name)
        .map(|(_, game)| game.clone())
}

/// Scan the running processes for a known game.
pub fn scan_running_game() -> Option<GameType> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines().find_map(|line| {
            let first = line.split(',').next()?.trim_matches('"');
            match_process(first)
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_games_are_matched_case_insensitively() {
        assert_eq!(match_process("cs2.exe"), Some(GameType::CS2));
        assert_eq!(match_process("CS2.EXE"), Some(GameType::CS2));
        assert_eq!(match_process("Among Us.exe"), Some(GameType::AmongUs));
        assert_eq!(
            match_process("VALORANT-Win64.exe"),
            Some(GameType::Valorant)
        );
        assert_eq!(
            match_process("LeagueClient.exe"),
            Some(GameType::LeagueOfLegends)
        );
    }

    #[test]
    fn unknown_processes_match_nothing() {
        assert_eq!(match_process("spotify.exe"), None);
        assert_eq!(match_process("chrome.exe"), None);
        assert_eq!(match_process(""), None);
    }

    #[test]
    fn every_mapping_has_a_game() {
        for (proc_name, game) in GAME_PROCESSES {
            assert_eq!(
                match_process(proc_name),
                Some(game.clone()),
                "mapping broken for {}",
                proc_name
            );
            assert!(!proc_name.is_empty());
        }
    }
}
