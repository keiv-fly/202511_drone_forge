pub mod tile;
pub mod coords;
pub mod resources;
pub mod world;
pub mod drones;
pub mod tasks;
pub mod dsl_ast;
pub mod hud;
pub mod engine;

// Re-exports for convenience in tests and integration users.
pub use tile::TileKind;
pub use coords::{TileCoord3, TileBox3};
pub use resources::Resources;
pub use world::World;
pub use drones::{Drone, DroneStatus};
pub use tasks::{Task, TaskManager, TaskState};
pub use dsl_ast::{Program, compile_program_to_tasks};
pub use hud::{format_hud, format_side_panel};
pub use engine::Engine;


