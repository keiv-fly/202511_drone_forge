use crate::drones::{Drone, DroneStatus};
use crate::tasks::{apply_task, TaskManager};
use crate::world::World;

#[derive(Debug)]
pub struct Engine {
	pub world: World,
	pub drones: Vec<Drone>,
	pub tasks: TaskManager,
}

impl Engine {
	pub fn new(world: World, drones: Vec<Drone>) -> Self {
		Self { world, drones, tasks: TaskManager::new() }
	}

	// Processes a single step:
	// - Pick an idle drone
	// - If a task is pending, move to Thinking -> Working
	// - Apply the task immediately for Milestone 1
	// - Mark drone Finished and then back to Idle
	pub fn tick(&mut self) {
		// Find an available drone
		let drone_idx = self.drones.iter().position(|d| matches!(d.status, DroneStatus::Idle | DroneStatus::Finished));
		if drone_idx.is_none() {
			return;
		}
		let idx = drone_idx.unwrap();

		// Start a task if any
		if self.tasks.any_pending() {
			let next = self.tasks.start_next();
			if let Some(task) = next {
				self.drones[idx].status = DroneStatus::Thinking;
				self.drones[idx].current_task = Some(task.clone());
				// In Milestone 1 we immediately execute
				self.drones[idx].status = DroneStatus::Working;
				let _tiles = apply_task(&mut self.world, &task);
				self.tasks.complete_current(&task);
				self.drones[idx].status = DroneStatus::Finished;
				self.drones[idx].current_task = None;
				// Reset to Idle for next frame
				self.drones[idx].status = DroneStatus::Idle;
			}
		}
	}
}

#[cfg(test)]
mod tests {
use super::*;
use crate::coords::{TileBox3, TileCoord3};
use crate::tasks::Task;
use crate::tile::TileKind;

	#[test]
	fn engine_executes_task() {
		let world = World::new(2, 2, 1, TileKind::Stone);
		let mut engine = Engine::new(world, vec![Drone { id: 1, status: DroneStatus::Idle, current_task: None }]);
		let t = Task::MineBox(TileBox3::new(TileCoord3::new(0,0,0), TileCoord3::new(1,1,0)));
		engine.tasks.push(t);
		engine.tick();
		assert_eq!(engine.world.resources.stone, 4);
	}
}


