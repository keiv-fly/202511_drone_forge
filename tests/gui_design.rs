use droneforge::*;

#[test]
fn top_hud_matches_design_outline() {
    let resources = Resources { stone: 8, iron: 3 };
    let wave_label = "Wave 2 in 02:30";
    let hud_line = format_hud(&resources, wave_label, (75, 100));
    assert!(hud_line.contains("Stone: 8"));
    assert!(hud_line.contains("Iron: 3"));
    assert!(hud_line.contains(wave_label));
    assert!(hud_line.contains("Core 75/100"));

    let controls = hud_controls(1, false);
    assert_eq!(controls.z_readout, "Z: 1");
    assert_eq!(controls.z_up_label, "Z▲");
    assert_eq!(controls.z_down_label, "Z▼");
    assert_eq!(controls.pause_label, "Pause");
}

#[test]
fn mouse_first_controls_are_exposed() {
    for expected in ["Select", "Mine Area", "Build Warrior", "Cancel"] {
        assert!(TOOL_STRIP_LABELS.contains(&expected));
    }
}

#[test]
fn console_and_panels_have_named_entries() {
    assert_eq!(
        CONSOLE_HINT,
        "Describe task… (MVP: uses selected area to create mine_box)"
    );
    assert_eq!(CONSOLE_SUBMIT_LABEL, "Submit");
    assert_eq!(DRONE_PANEL_HEADING, "Drones");
    assert_eq!(TASK_PANEL_HEADING, "Tasks");
}
