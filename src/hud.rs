use crate::drones::Drone;
use crate::resources::Resources;
use crate::tasks::{TaskState, TaskManager};

pub fn format_hud(resources: &Resources, wave_label: &str) -> String {
	format!(
		"Stone: {} | Iron: {} | {}",
		resources.stone, resources.iron, wave_label
	)
}

pub fn format_side_panel(drones: &[Drone], tasks: &TaskManager) -> Vec<String> {
	let mut out = Vec::new();
	out.push("[Drones]".to_string());
	for d in drones {
		let status = match d.status {
			crate::drones::DroneStatus::Idle => "Idle",
			crate::drones::DroneStatus::Thinking => "Thinking...",
			crate::drones::DroneStatus::Working => "Working",
			crate::drones::DroneStatus::Finished => "Finished",
		};
		let task = d.current_task.as_ref().map(|t| t.description()).unwrap_or_else(|| "None".to_string());
		out.push(format!("Drone #{} – {} – {}", d.id, status, task));
	}
	out.push("[Tasks]".to_string());
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
	use crate::drones::{Drone, DroneStatus};
	use crate::coords::{TileBox3, TileCoord3};

	#[test]
	fn hud_format() {
		let r = Resources { stone: 3, iron: 5 };
		let s = format_hud(&r, "Wave 1 in 01:23");
		assert!(s.contains("Stone: 3"));
		assert!(s.contains("Iron: 5"));
		assert!(s.contains("Wave 1 in 01:23"));
	}

	#[test]
	fn side_panel_lists_drones_and_tasks() {
		let mut tasks = TaskManager::new();
		let t = Task::MineBox(TileBox3::new(TileCoord3::new(0,0,0), TileCoord3::new(1,1,0)));
		tasks.push(t.clone());
		let drones = vec![Drone { id: 1, status: DroneStatus::Idle, current_task: Some(t) }];
		let lines = format_side_panel(&drones, &tasks);
		assert!(lines.iter().any(|l| l.contains("Drone #1")));
		assert!(lines.iter().any(|l| l.contains("Tasks")));
	}
}


