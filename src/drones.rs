use serde::{Deserialize, Serialize};

use crate::tasks::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneStatus {
	Idle,
	Thinking,
	Working,
	Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drone {
	pub id: u32,
	pub status: DroneStatus,
	pub current_task: Option<Task>,
}

impl Drone {
	pub fn new(id: u32) -> Self {
		Self { id, status: DroneStatus::Idle, current_task: None }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn drone_init() {
		let d = Drone::new(1);
		assert_eq!(d.id, 1);
		assert_eq!(d.status, DroneStatus::Idle);
		assert!(d.current_task.is_none());
	}
}


