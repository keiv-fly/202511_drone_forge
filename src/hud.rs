use crate::drones::Drone;
use crate::resources::Resources;
use crate::tasks::{TaskManager, TaskState};

pub const TOOL_STRIP_LABELS: [&str; 4] = ["Select", "Mine Area", "Build Warrior", "Cancel"];
pub const CONSOLE_HINT: &str = "Describe task… (MVP: uses selected area to create mine_box)";
pub const CONSOLE_SUBMIT_LABEL: &str = "Submit";
pub const DRONE_PANEL_HEADING: &str = "Drones";
pub const TASK_PANEL_HEADING: &str = "Tasks";

pub struct HudControls {
    pub z_readout: String,
    pub z_up_label: &'static str,
    pub z_down_label: &'static str,
    pub pause_label: String,
}

pub fn format_hud(resources: &Resources, wave_label: &str, core_hp: (u32, u32)) -> String {
    format!(
        "Stone: {} | Iron: {} | {} | Core {}/{}",
        resources.stone, resources.iron, wave_label, core_hp.0, core_hp.1
    )
}

pub fn hud_controls(current_z: i32, paused: bool) -> HudControls {
    HudControls {
        z_readout: format!("Z: {}", current_z),
        z_up_label: "Z▲",
        z_down_label: "Z▼",
        pause_label: if paused { "Resume" } else { "Pause" }.to_string(),
    }
}

pub fn format_side_panel(drones: &[Drone], tasks: &TaskManager) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!("[{}]", DRONE_PANEL_HEADING));
    for d in drones {
        let status = match d.status {
            crate::drones::DroneStatus::Idle => "Idle",
            crate::drones::DroneStatus::Thinking => "Thinking...",
            crate::drones::DroneStatus::Working => "Working",
            crate::drones::DroneStatus::Finished => "Finished",
        };
        let task = d
            .current_task
            .as_ref()
            .map(|t| t.description())
            .unwrap_or_else(|| "None".to_string());
        out.push(format!("Drone #{} – {} – {}", d.id, status, task));
    }
    out.push(format!("[{}]", TASK_PANEL_HEADING));
    for (t, s) in &tasks.tasks {
        let state = match s {
            TaskState::Pending => "Pending",
            TaskState::InProgress => "InProgress",
            TaskState::Done => "Done",
        };
        out.push(format!("{} – {}", t.description(), state));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::{TileBox3, TileCoord3};
    use crate::drones::{Drone, DroneStatus};
    use crate::tasks::Task;

    #[test]
    fn hud_format() {
        let r = Resources { stone: 3, iron: 5 };
        let s = format_hud(&r, "Wave 1 in 01:23", (100, 100));
        assert!(s.contains("Stone: 3"));
        assert!(s.contains("Iron: 5"));
        assert!(s.contains("Wave 1 in 01:23"));
        assert!(s.contains("Core 100/100"));
    }

    #[test]
    fn hud_controls_match_design() {
        let controls = hud_controls(0, false);
        assert_eq!(controls.z_readout, "Z: 0");
        assert_eq!(controls.z_up_label, "Z▲");
        assert_eq!(controls.z_down_label, "Z▼");
        assert_eq!(controls.pause_label, "Pause");

        let paused = hud_controls(2, true);
        assert_eq!(paused.pause_label, "Resume");
    }

    #[test]
    fn side_panel_lists_drones_and_tasks() {
        let mut tasks = TaskManager::new();
        let t = Task::MineBox(TileBox3::new(
            TileCoord3::new(0, 0, 0),
            TileCoord3::new(1, 1, 0),
        ));
        tasks.push(t.clone());
        let drones = vec![Drone {
            id: 1,
            status: DroneStatus::Idle,
            current_task: Some(t),
        }];
        let lines = format_side_panel(&drones, &tasks);
        assert!(lines.iter().any(|l| l.contains("Drone #1")));
        assert!(lines.iter().any(|l| l.contains(TASK_PANEL_HEADING)));
    }
}
