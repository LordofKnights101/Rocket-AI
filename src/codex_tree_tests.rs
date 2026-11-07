//! Tests for the 🌳 CREATE TREE atelier (engine: src/codex_tree.rs).

use super::*;
use crate::codex::Codex;

fn kb() -> KnowledgeBase {
    KnowledgeBase::new()
}

fn game(name: &str) -> GameType {
    Codex::match_game(name).expect("game in registry")
}

fn mk_spec(game: &str, archetype: TreeArchetype, depth: u32) -> Commission {
    Commission {
        game: game.into(),
        game2: None,
        archetype,
        depth,
        breadth: 2,
        curve: Curve::Linear,
        name: None,
        flavor: None,
    }
}

#[test]
fn commission_parsing_accepts_all_directives() {
    // Bare commission.
    let c = parse_commission("for minecraft").unwrap();
    assert_eq!(c.game, "minecraft");
    assert_eq!(c.archetype, TreeArchetype::LinearMastery);
    assert_eq!(c.depth, 3);

    // Full directive set.
    let c = parse_commission(
        "branching for terraria depth 4 breadth 3 steep flavor: The Deep Delve",
    )
    .unwrap();
    assert_eq!(c.game, "terraria");
    assert_eq!(c.archetype, TreeArchetype::BranchingWeb);
    assert_eq!(c.depth, 4);
    assert_eq!(c.breadth, 3);
    assert_eq!(c.curve, Curve::Steep);
    assert_eq!(c.flavor.as_deref(), Some("The Deep Delve"));

    // Comparative pair.
    let c = parse_commission("comparative for minecraft and terraria depth 2").unwrap();
    assert_eq!(c.archetype, TreeArchetype::ComparativeNetwork);
    assert_eq!(c.game2.as_deref(), Some("terraria"));

    // Errors.
    assert!(parse_commission("").is_err());
    assert!(parse_commission("comparative for minecraft").is_err());
}

#[test]
fn generation_grounded_in_grimoire_for_every_archetype() {
    let kb = kb();
    let g1 = game("minecraft");
    for archetype in TreeArchetype::ALL {
        let (spec, r2) = if archetype == TreeArchetype::ComparativeNetwork {
            (
                Commission {
                    game2: Some("terraria".into()),
                    ..mk_spec("minecraft", archetype, 3)
                },
                Some(game("terraria")),
            )
        } else {
            (mk_spec("minecraft", archetype, 3), None)
        };
        let tree = generate_tree(&kb, &spec, "t1".into(), &g1, r2.as_ref())
            .unwrap_or_else(|e| panic!("{:?} failed: {}", archetype, e));
        // Expected node count by shape: chains are depth long; a branching web
        // is root + breadth × middle tiers + confluence.
        let expected = if archetype == TreeArchetype::BranchingWeb {
            if 3 == 1 {
                1
            } else {
                2 + 2 * (3 - 2) // breadth = 2, depth = 3
            }
        } else {
            3
        };
        assert_eq!(tree.nodes.len(), expected, "{:?}", archetype);
        // Traceability: every section exists in the grimoire(s) or is a
        // documented pseudo-section (gaps / ordeal runs).
        let real_cats: Vec<&str> = kb
            .entries_for_game(&g1)
            .iter()
            .map(|e| e.category.label())
            .collect();
        for n in &tree.nodes {
            assert!(!n.title.is_empty());
            assert!(n.xp_reward > 0);
            assert!(n.target() > 0, "{:?}/{:?} has no criteria", archetype, n.id);
            for s in &n.grimoire_sections {
                let pseudo = s.contains("(gaps)") || s.contains("consecutive grounded");
                let cross = archetype == TreeArchetype::ComparativeNetwork;
                assert!(
                    pseudo || real_cats.iter().any(|c| s.contains(c)) || cross,
                    "{:?} node {} cites unknown section {}",
                    archetype,
                    n.id,
                    s
                );
            }
        }
        // Rank gates are declared and consistent with the archetype.
        assert_eq!(tree.rank_gate, archetype.rank_gate());
        assert!(tree.nodes.iter().all(|n| n.rank_required == tree.rank_gate));
    }
}

#[test]
fn generation_refuses_blank_grimoires() {
    let kb = kb();
    // A game with no curated pack (find one in the registry).
    let empty = crate::all_games()
        .into_iter()
        .find(|g| kb.entries_for_game(g).is_empty() && *g != GameType::Generic)
        .expect("registry has uncurated games");
    let spec = mk_spec(empty.get_name(), TreeArchetype::LinearMastery, 3);
    assert!(generate_tree(&kb, &spec, "t".into(), &empty, None).is_err());
}

#[test]
fn prerequisites_gate_progress() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::LinearMastery, 3);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    // Answer with the root's category: only the root progresses.
    let root_cat = trees[0].nodes[0].criteria.category.clone();
    record_answer(&mut trees, &g1, root_cat.as_deref(), true);
    assert_eq!(trees[0].nodes[0].progress, 1);
    assert_eq!(
        trees[0].nodes[1].progress, 0,
        "tier 1 blocked by prerequisite"
    );
    // Finish the root; tier 1 becomes the frontier.
    record_answer(&mut trees, &g1, root_cat.as_deref(), true);
    assert!(trees[0].nodes[0].completed);
    let frontier = trees[0].frontier();
    assert!(frontier.contains(&trees[0].nodes[1].id));
}

#[test]
fn ordeal_nodes_reset_on_faltering() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::OrdealLadder, 2);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    record_answer(&mut trees, &g1, None, true);
    assert_eq!(trees[0].nodes[0].progress, 1);
    record_answer(&mut trees, &g1, None, false); // faltered
    assert_eq!(trees[0].nodes[0].progress, 0, "ordeal run resets");
}

#[test]
fn comparative_nodes_need_both_realms() {
    let kb = kb();
    let spec = Commission {
        game2: Some("terraria".into()),
        ..mk_spec("minecraft", TreeArchetype::ComparativeNetwork, 2)
    };
    let g1 = game("minecraft");
    let g2 = game("terraria");
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, Some(&g2)).unwrap()];
    let realms = trees[0].nodes[0].criteria.comparative_realms.clone();
    assert_eq!(realms.len(), 2);
    // Answer only in realm one.
    record_answer(&mut trees, &g1, None, true);
    assert!(!trees[0].nodes[0].completed);
    // Now realm two — the node seals.
    record_answer(&mut trees, &g2, None, true);
    assert!(trees[0].nodes[0].completed);
    assert!(trees[0].xp_earned > 0);
}

#[test]
fn restoration_nodes_advance_on_teaching() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::RestorationCampaign, 2);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    // Drive teachings until the first node seals.
    for _ in 0..4 {
        let _ = record_teach(&mut trees, &g1);
    }
    assert!(trees[0].nodes[0].completed, "teach events seal restoration nodes");
    assert!(trees[0].xp_earned > 0);
}

#[test]
fn full_cycle_commission_minimize_restore_refine_progress() {
    // The spec's demonstration cycle, end to end through Rocket's desk.
    let dir = std::env::temp_dir().join(format!("rocket-codex-tree-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();

    // 1) COMMISSION via the desk (Linear Mastery is open to a Novice Scribe).
    let reply = codex.desk_reply("create tree for minecraft depth 3 gentle", &kb, 3);
    assert!(reply.contains("Quest Tree commissioned"), "{}", reply);
    assert_eq!(codex.trees.len(), 1);
    let tree_id = codex.trees[0].id.clone();
    assert_eq!(codex.focused_tree.as_deref(), Some(tree_id.as_str()));

    // 2) MINIMIZE — no data loss.
    let reply = codex.desk_reply("minimize tree", &kb, 3);
    assert!(reply.contains("minimized"));
    assert!(!codex.trees[0].active);
    assert_eq!(codex.trees[0].nodes.len(), 3, "state intact while minimized");

    // 3) RESTORE.
    let reply = codex.desk_reply("open tree minecraft", &kb, 3);
    assert!(reply.contains("restored"));
    assert!(codex.trees[0].active);

    // 4) REFINE — expand depth by 2 (5 nodes total).
    let reply = codex.desk_reply("expand tree minecraft depth 2", &kb, 3);
    assert!(reply.contains("grows"), "{}", reply);
    assert_eq!(codex.trees[0].nodes.len(), 5);
    assert!(codex.trees[0].revisions.iter().any(|r| r.directive == "expand"));

    // 5) PROGRESS — grounded answers drive nodes through prerequisites.
    let g1 = game("minecraft");
    let mut total_xp = 0u32;
    for _ in 0..60 {
        let tag = codex.trees[0]
            .frontier()
            .first()
            .and_then(|nid| codex.trees[0].node(nid))
            .and_then(|n| n.criteria.category.clone());
        let out = codex.tree_answer_event(&g1, tag.as_deref(), true);
        total_xp += out.xp;
        if codex.trees[0].completed_sealed {
            break;
        }
    }
    assert!(total_xp > 0, "progression must pay pages");
    assert!(
        codex.trees[0].completed_sealed,
        "the gentle 5-node tree seals under sustained scholarship"
    );
    assert!(codex.trees[0].xp_earned > 0);

    // 6) Progress visible in the atelier listing.
    let report = codex.desk_reply("trees", &kb, 3);
    assert!(report.contains("sealed") || report.contains("🏆"));
}

#[test]
fn tab_semantics_minimize_duplicate_archive_destroy() {
    let dir = std::env::temp_dir().join(format!("rocket-codex-tab-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();
    let _ = codex.desk_reply("create tree for minecraft", &kb, 20);
    let _ = codex.desk_reply("create tree for terraria", &kb, 20);
    assert_eq!(codex.trees.len(), 2);

    // Duplicate is independent.
    let _ = codex.desk_reply("duplicate tree minecraft", &kb, 20);
    assert_eq!(codex.trees.len(), 3);
    assert!(codex.trees[2].name.contains("(copy)"));

    // Rename inscribes a revision.
    let _ = codex.desk_reply("rename tree terraria to Crimson Ledger", &kb, 20);
    assert_eq!(codex.trees[1].name, "Crimson Ledger");
    assert!(codex.trees[1].revisions.iter().any(|r| r.directive == "rename"));

    // Archive hides from the tab bar but keeps the document.
    let _ = codex.desk_reply("archive tree Crimson Ledger", &kb, 20);
    assert!(codex.trees[1].archived);
    let _ = codex.desk_reply("restore tree Crimson Ledger", &kb, 20);
    assert!(!codex.trees[1].archived);

    // Destroy removes permanently and logs the deed.
    let n_before = codex.trees.len();
    let _ = codex.desk_reply("destroy tree (copy)", &kb, 20);
    assert_eq!(codex.trees.len(), n_before - 1);
    assert!(codex.actions.iter().any(|a| a.action == "tree_destroyed"));
}

#[test]
fn rebalance_rescales_economy() {
    let dir = std::env::temp_dir().join(format!("rocket-codex-rebal-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();
    let _ = codex.desk_reply("create tree for minecraft depth 3", &kb, 20);
    let before: u32 = codex.trees[0].nodes.iter().map(|n| n.xp_reward).sum();
    let _ = codex.desk_reply("rebalance tree Minecraft to 600", &kb, 20);
    let after: u32 = codex.trees[0].nodes.iter().map(|n| n.xp_reward).sum();
    assert_eq!(after, 600);
    assert_ne!(before, 600, "natural curve total differs from the forced 600");
    assert!(codex.trees[0].revisions.iter().any(|r| r.directive == "rebalance"));
}

#[test]
fn validation_and_repair_of_stale_nodes() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::LinearMastery, 2);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    // Sabotage: point a node at a section no grimoire holds.
    trees[0].nodes[0].grimoire_sections = vec!["🖋️ Forgotten Discipline".into()];
    let report = validate_trees(&mut trees, &kb);
    assert_eq!(report.get("t1"), Some(&1), "the sabotaged node is stale");
    assert!(trees[0].nodes[0].stale);
    let repaired = repair_tree(&mut trees[0], &kb);
    assert_eq!(repaired, 1);
    assert!(!trees[0].nodes[0].stale);
    assert!(trees[0].nodes[0]
        .grimoire_sections
        .iter()
        .all(|s| !s.contains("Forgotten")));
    assert!(trees[0].revisions.iter().any(|r| r.directive == "repair"));
}

#[test]
fn tree_completion_pays_order_bonus() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::LinearMastery, 2);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    let mut bonus_seen = false;
    for _ in 0..20 {
        let tag = trees[0]
            .frontier()
            .first()
            .and_then(|nid| trees[0].node(nid))
            .and_then(|n| n.criteria.category.clone());
        let out = record_answer(&mut trees, &g1, tag.as_deref(), true);
        if out.lines.iter().any(|l| l.contains("Quest Tree sealed")) {
            bonus_seen = true;
            break;
        }
    }
    assert!(bonus_seen, "sealing every node pays the Order bonus");
    assert!(trees[0].completed_sealed);
}

#[test]
fn manual_completion_gates_and_pays() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::LinearMastery, 3);
    let mut trees = vec![generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap()];
    let root = trees[0].nodes[0].id.clone();
    let deep = trees[0].nodes[2].id.clone();

    // A locked node refuses gently, in character.
    let err = complete_node(&mut trees[0], &deep).unwrap_err();
    assert!(err.contains("prerequisites") || err.contains("not yet within reach"), "{}", err);
    assert!(!trees[0].nodes[2].completed);

    // The root seals and pays exactly its XP.
    let root_xp = trees[0].nodes[0].xp_reward;
    let (xp, lines, lit) = complete_node(&mut trees[0], &root).unwrap();
    assert_eq!(xp, root_xp);
    assert!(trees[0].nodes[0].completed);
    assert!(lines.iter().any(|l| l.contains("Node sealed")));
    // The successor path lights up.
    assert_eq!(lit.len(), 1, "the next chapter unlocks");
    assert!(trees[0].nodes[1].completed == false);
    assert!(trees[0].frontier().contains(&trees[0].nodes[1].id));
    assert!(trees[0].revisions.iter().any(|r| r.directive == "node_sealed"));
}

#[test]
fn graph_layout_is_hierarchical_and_complete() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = Commission {
        breadth: 2,
        ..mk_spec("minecraft", TreeArchetype::BranchingWeb, 3)
    };
    let tree = generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap();
    let laid = layout(&tree);
    assert_eq!(laid.len(), tree.nodes.len(), "every node is placed");
    // Tiers form columns; rows are unique within a tier.
    for tier in laid.iter().map(|n| n.tier).collect::<std::collections::BTreeSet<_>>() {
        let mut rows: Vec<u32> = laid.iter().filter(|n| n.tier == tier).map(|n| n.row).collect();
        rows.sort();
        rows.dedup();
        assert_eq!(rows.len(), laid.iter().filter(|n| n.tier == tier).count());
    }
    // Every edge connects nodes that exist.
    let edges = layout_edges(&tree, &laid);
    assert!(!edges.is_empty());
    for (from, to) in &edges {
        assert!(from < &laid.len() && to < &laid.len());
    }
}

#[test]
fn outline_lists_every_node() {
    let kb = kb();
    let g1 = game("minecraft");
    let spec = mk_spec("minecraft", TreeArchetype::LinearMastery, 3);
    let tree = generate_tree(&kb, &spec, "t1".into(), &g1, None).unwrap();
    let out = outline(&tree);
    for n in &tree.nodes {
        assert!(out.contains(&n.title), "outline lists {}", n.title);
    }
    assert!(out.contains("pages earned"));
}

#[test]
fn commission_conversation_flows_from_button_to_tree() {
    let dir = std::env::temp_dir().join(format!("rocket-commission-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();

    // The button asks THE question.
    let reply = codex.begin_commission(Some("Roblox".into()));
    assert!(reply.contains("What game quest tree do you want me to build?"));
    assert!(codex.commission_draft.is_some());

    // ONE answer — `current` — generates the tree instantly with defaults.
    let reply = codex.desk_reply("current", &kb, 3);
    assert!(reply.contains("Quest Tree commissioned"), "{}", reply);
    assert!(codex.commission_draft.is_none());
    assert_eq!(codex.trees.len(), 1);
    assert_eq!(codex.trees[0].game, "Roblox");
    assert_eq!(codex.trees[0].archetype, TreeArchetype::LinearMastery);

    // A full one-shot commission still carries a shape when asked for.
    let reply2 = codex.desk_reply("create tree branching for terraria depth 4 steep", &kb, 5);
    assert!(reply2.contains("Quest Tree commissioned"), "{}", reply2);
    assert_eq!(codex.trees[1].archetype, TreeArchetype::BranchingWeb);

    // Cancel escapes mid-flow.
    let _ = codex.begin_commission(None);
    let reply = codex.desk_reply("nevermind", &kb, 3);
    assert!(reply.contains("rests unfinished"));
    assert!(codex.commission_draft.is_none());
}

#[test]
fn outline_and_confirm_saved_commands_work_via_desk() {
    let dir = std::env::temp_dir().join(format!("rocket-outline-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();
    let _ = codex.desk_reply("create tree for minecraft depth 2", &kb, 20);

    let out = codex.desk_reply("outline tree minecraft", &kb, 20);
    assert!(out.contains("pages earned"));
    assert!(out.contains("Chapter") || out.contains("Colloquy"));

    let confirm = codex.desk_reply("confirm saved", &kb, 20);
    assert!(confirm.contains("safely written"));
    assert!(confirm.contains("codex_character.json"));

    // Reorder via desk too.
    let _ = codex.desk_reply("create tree for terraria depth 2", &kb, 20);
    let _ = codex.desk_reply("reorder tree terraria up", &kb, 20);
    assert_eq!(codex.trees[0].game, "Terraria");
}

#[test]
fn tab_close_keeps_tree_and_refocuses() {
    let dir = std::env::temp_dir().join(format!("rocket-close-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("ROCKET_CODEX_DIR", &dir);
    let kb = kb();
    let mut codex = Codex::default();
    let _ = codex.desk_reply("create tree for minecraft depth 2", &kb, 20);
    let _ = codex.desk_reply("create tree for terraria depth 2", &kb, 20);
    let skyrim_id = {
        let _ = codex.begin_commission(Some("Skyrim".into()));
        let _ = codex.desk_reply("current", &kb, 20);
        let _ = codex.desk_reply("default", &kb, 20);
        codex.trees[2].id.clone()
    };
    assert_eq!(codex.trees.len(), 3);

    // Closing the focused tab minimizes it — tree kept, focus moves on.
    let was_active = codex.trees.iter().find(|t| t.id == skyrim_id).unwrap().active;
    assert!(was_active);
    codex.tree_tab_close(&skyrim_id);
    let closed = codex.trees.iter().find(|t| t.id == skyrim_id).unwrap();
    assert!(!closed.active, "tab closed");
    assert_eq!(closed.nodes.len(), 3, "tree and progress kept safe");
    assert_ne!(
        codex.focused_tree.as_deref(),
        Some(skyrim_id.as_str()),
        "focus moves to the next open tab"
    );
    // Reopen restores it fully.
    let _ = codex.desk_reply(&format!("open tree {}", skyrim_id), &kb, 20);
    assert!(codex.trees.iter().find(|t| t.id == skyrim_id).unwrap().active);
}
