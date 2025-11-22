use serde::{Deserialize, Serialize};

use crate::coords::{TileBox3, TileCoord3};
use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Task {
    MineBox(TileBox3),
}

impl Task {
    pub fn description(&self) -> String {
        match self {
            Task::MineBox(b) => format!(
                "Mine box (({},{},{})->({},{},{}))",
                b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z
            ),
        }
    }
}

#[derive(Debug, Default)]
pub struct TaskManager {
    pub tasks: Vec<(Task, TaskState)>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push((task, TaskState::Pending));
    }

    pub fn any_pending(&self) -> bool {
        self.tasks.iter().any(|(_, s)| *s == TaskState::Pending)
    }

    pub fn start_next(&mut self) -> Option<Task> {
        if let Some((_, (task, state))) = self
            .tasks
            .iter_mut()
            .enumerate()
            .find(|(_, (_, s))| *s == TaskState::Pending)
        {
            *state = TaskState::InProgress;
            return Some(task.clone());
        }
        None
    }

    pub fn complete_current(&mut self, t: &Task) {
        if let Some((_, state)) = self.tasks.iter_mut().find(|(task, _)| task == t) {
            *state = TaskState::Done;
        }
    }
}

pub fn apply_task(world: &mut World, task: &Task) -> u32 {
    match task {
        Task::MineBox(b) => {
            let mut count = 0u32;
            for c in b.iter_tiles() {
                if world
                    .mine_tile(TileCoord3 {
                        x: c.x,
                        y: c.y,
                        z: c.z,
                    })
                    .is_some()
                {
                    count = count.saturating_add(1);
                }
            }
            count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::TileKind;
    use crate::world::World;

    #[test]
    fn task_description() {
        let t = Task::MineBox(TileBox3::new(
            TileCoord3::new(0, 0, 0),
            TileCoord3::new(1, 1, 0),
        ));
        assert!(t.description().contains("Mine box"));
    }

    #[test]
    fn task_manager_flow() {
        let mut tm = TaskManager::new();
        let t = Task::MineBox(TileBox3::new(
            TileCoord3::new(0, 0, 0),
            TileCoord3::new(0, 0, 0),
        ));
        tm.push(t.clone());
        assert!(tm.any_pending());
        let started = tm.start_next().unwrap();
        assert_eq!(started, t);
        tm.complete_current(&started);
        assert!(!tm.any_pending());
    }

    #[test]
    fn apply_mine_task_counts_mined_tiles() {
        let mut world = World::new(2, 2, 1, TileKind::Stone);
        let t = Task::MineBox(TileBox3::new(
            TileCoord3::new(0, 0, 0),
            TileCoord3::new(1, 1, 0),
        ));
        let mined = apply_task(&mut world, &t);
        assert_eq!(mined, 4);
        assert_eq!(world.resources.stone, 4);
    }
}
