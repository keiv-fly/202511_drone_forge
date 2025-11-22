use droneforge::*;
use serde_json::json;

#[test]
fn end_to_end_mining_from_ast() {
    // Build a small world with stone
    let world = World::new(3, 3, 1, TileKind::Stone);
    let mut engine = Engine::new(world, vec![Drone::new(1)]);

    // AST program: let area; mine_box(area)
    let program_json = json!({
        "version": 1,
        "node": "Program",
        "statements": [
            {
                "node": "Let",
                "name": "area",
                "ty": "TileBox",
                "value": {
                    "node": "TileBoxFromCoords",
                    "min": { "node": "TileCoord", "x": 0, "y": 0, "z": 0 },
                    "max": { "node": "TileCoord", "x": 1, "y": 1, "z": 0 }
                }
            },
            {
                "node": "ExprStmt",
                "expr": {
                    "node": "Call",
                    "func": "mine_box",
                    "args": [{ "node": "VarRef", "name": "area" }]
                }
            }
        ]
    });
    let prog: Program = serde_json::from_value(program_json).unwrap();
    let tasks = compile_program_to_tasks(&prog).unwrap();
    assert_eq!(tasks.len(), 1);
    for t in tasks {
        engine.tasks.push(t);
    }

    // Run one engine tick (M1 applies task immediately)
    engine.tick();

    // Verify resources and UI strings
    assert_eq!(engine.world.resources.stone, 4);
    let (core_hp, core_hp_max) = engine.world.core_hp();
    let hud = format_hud(
        &engine.world.resources,
        "Wave 1 in 01:23",
        (core_hp, core_hp_max),
    );
    assert!(hud.contains("Stone: 4"));
    assert!(hud.contains("Core HP"));
    let side = format_side_panel(&engine.drones, &engine.tasks);
    assert!(side.iter().any(|l| l.contains("Drones")));
    assert!(side.iter().any(|l| l.contains("Tasks")));
}
