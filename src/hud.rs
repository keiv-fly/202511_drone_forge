use crate::drones::{Drone, DroneStatus};
use crate::resources::Resources;
use crate::tasks::{TaskManager, TaskState};

pub const HUD_SEPARATOR: &str = " • ";
pub const HUD_Z_UP_LABEL: &str = "Z▲";
pub const HUD_Z_DOWN_LABEL: &str = "Z▼";
pub const HUD_PAUSE_LABEL: &str = "Pause";

pub fn format_hud(resources: &Resources, wave_label: &str, core_hp: (u32, u32)) -> String {
    let (core_hp_current, core_hp_max) = core_hp;
    format!(
        "Stone: {}{}Iron: {}{}{}{}Core HP: {}/{}",
        resources.stone,
        HUD_SEPARATOR,
        resources.iron,
        HUD_SEPARATOR,
        wave_label,
        HUD_SEPARATOR,
        core_hp_current,
        core_hp_max,
    )
}

pub fn format_side_panel(drones: &[Drone], tasks: &TaskManager) -> Vec<String> {
    let mut out = Vec::new();
    out.push("[Drones]".to_string());
    for d in drones {
        let status = match d.status {
            DroneStatus::Idle => "Idle",
            DroneStatus::Thinking => "Thinking...",
            DroneStatus::Working => "Working",
            DroneStatus::Finished => "Finished",
        };
        let task = d
            .current_task
            .as_ref()
            .map(|t| t.description())
            .unwrap_or_else(|| "None".to_string());
        out.push(format!("Drone #{} - {} - {}", d.id, status, task));
    }
    out.push("[Tasks]".to_string());
    for (t, s) in &tasks.tasks {
        let state = match s {
            TaskState::Pending => "Pending",
            TaskState::InProgress => "InProgress",
            TaskState::Done => "Done",
        };
        out.push(format!("{} - {}", t.description(), state));
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
    fn hud_format_includes_design_tokens() {
        let r = Resources { stone: 3, iron: 5 };
        let s = format_hud(&r, "Wave 1 in 01:23", (90, 100));
        assert!(s.contains("Stone: 3"));
        assert!(s.contains("Iron: 5"));
        assert!(s.contains("Wave 1 in 01:23"));
        assert!(s.contains("Core HP: 90/100"));
        assert!(s.contains(HUD_SEPARATOR));
    }

    #[test]
    fn hud_controls_match_design() {
        assert_eq!(HUD_Z_UP_LABEL, "Z▲");
        assert_eq!(HUD_Z_DOWN_LABEL, "Z▼");
        assert_eq!(HUD_PAUSE_LABEL, "Pause");
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
        assert!(lines.iter().any(|l| l.contains("Tasks")));
    }
}
