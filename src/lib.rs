pub mod coords;
pub mod drones;
pub mod dsl_ast;
pub mod engine;
pub mod hud;
pub mod resources;
pub mod tasks;
pub mod tile;
pub mod world;

// Re-exports for convenience in tests and integration users.
pub use coords::{TileBox3, TileCoord3};
pub use drones::{Drone, DroneStatus};
pub use dsl_ast::{Program, compile_program_to_tasks};
pub use engine::Engine;
pub use hud::{format_hud, format_side_panel};
pub use resources::Resources;
pub use tasks::{Task, TaskManager, TaskState};
pub use tile::TileKind;
pub use world::World;
