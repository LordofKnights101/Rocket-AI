//! Rocket's re-explainer — the lore behind `remind me …` on the desk.
//!
//! (The former Rite of First Entry overlay was retired in v0.2.4; its
//! knowledge now lives on the 📖 Info page, and here for desk reminders.)

/// Re-explain any taught topic, on demand. Returns None when the text isn't
/// a reminder, so the desk falls through to normal handling.
pub fn re_explain(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let has_any = ["remind me", "show me again", "teach me", "how do i", "who are you"]
        .iter()
        .any(|k| lower.contains(k));
    if !has_any {
        return None;
    }
    if ["tree", "atelier", "commission", "create"].iter().any(|k| lower.contains(k))
        && ["minimize", "restore", "tab", "switch", "multiple", "another"]
            .iter()
            .any(|k| lower.contains(k))
    {
        return Some(
            "🌳 **Minimizing a tree and starting another** — on this desk: `minimize tree` folds the focused tree into its tab (state kept, still accruing). Then simply `create tree for <game> …` again — the new tree pins beside the minimized one as its own tab. Restore with `open tree <name>`, by clicking the tab, or `trees` to survey them all. The ＋ button in the atelier does it too.".into(),
        );
    }
    if ["tree", "atelier", "commission", "quest tree", "create"].iter().any(|k| lower.contains(k)) {
        return Some(
            "🌳 **Commissioning a Quest Tree** — press the 🌳 **Create Tree** button in the atelier and answer two questions, or address me: `create tree [archetype] for <game> [and <game2>] [depth N] [breadth N] [gentle|steep] [flavor: …]`. Example: `create tree branching for terraria depth 4 steep`. Archetypes: Linear Mastery · Branching Web · Ordeal Ladder · Comparative Network · Restoration · Timed Trials · Narrative Chronicle · Edge-Case Gauntlet — each with its rank gate. The graph appears as its own tab; click nodes to read them and **Mark Done** when their moment comes.".into(),
        );
    }
    if ["channel", "main hall", "side", "desk", "dual"].iter().any(|k| lower.contains(k)) {
        return Some(
            "🕯️ **The dual channels** — the **Main Hall** (Chat page input) is for scholarly inquiry into the grimoires: every grounded answer earns pages. **This desk** is Rocket's side channel: practical tasks, quests, audits, chronicles — never XP-earning scholarship.".into(),
        );
    }
    if ["xp", "rank", "page", "progress", "level", "standing"].iter().any(|k| lower.contains(k)) {
        return Some(
            "📈 **Progression** — grounded answers mint pages (= coins, never spent levels); pages raise your level and Order rank (Novice Scribe → Grand Archivist). Your identity card glows with the tally; 👤 Profile and `where stand I` show the full standing.".into(),
        );
    }
    if ["rocket", "familiar", "who are you", "help"].iter().any(|k| lower.contains(k)) {
        return Some(
            "🪶 **Rocket** — your familiar and the Order's agent of practical action: quest reports, trials, grimoire audits and gap lists, comparative tables, chronicle export, tree commissioning and refinement. Address me on this desk; `help` recites my arts.".into(),
        );
    }
    Some(
        "🕯️ The Codex has seven arts: the dual channels, asking and earning, my office, commissioning trees, the art of many tabs, your compass (Profile / `where stand I`), and this — my door, always open. Name any of them and I shall unfold it again. The full sheet lives on the 📖 Info page.".into(),
    )
}
