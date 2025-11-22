use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use droneforge::*;
use droneforge::world::World as GameWorld;

// ---------- Constants ----------
const TILE_SIZE: f32 = 16.0;
const INITIAL_Z_LEVEL: i32 = 0;
const WORLD_WIDTH: i32 = 64;
const WORLD_HEIGHT: i32 = 64;
const WORLD_LEVELS: i32 = 1;
const RNG_SEED: u64 = 42;

// ---------- Components ----------
#[derive(Component)]
struct TilePos {
	x: i32,
	y: i32,
	z: i32,
}

#[derive(Component)]
struct TilesLayer; // Marker to despawn/rebuild when Z changes

#[derive(Component)]
struct SelectionOverlay; // Marker for selection rectangle overlay

// ---------- Resources ----------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
	Select,
	MineArea,
	BuildWarrior,
}

#[derive(Resource)]
struct UiState {
	console_input: String,
	console_log: Vec<String>,
	focus_console: bool,
	paused: bool,
	current_z: i32,
	request_rebuild_tiles: bool,
	current_tool: Tool,
	toast: Option<(String, f32)>, // (message, remaining_seconds)
	core_hp: (u32, u32),
}

#[derive(Resource, Default)]
struct SelectionState {
	is_dragging: bool,
	start_world: Vec2,
	current_world: Vec2,
	last_box: Option<TileBox3>,
}

#[derive(Resource)]
struct GameEngine {
	engine: Engine,
}

// ---------- Entry ----------
fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Droneforge GUI (MVP)".to_string(),
				resolution: (1280, 800).into(),
				..Default::default()
			}),
			..Default::default()
		}))
		.add_plugins(EguiPlugin::default())
		// Resources
		.insert_resource(UiState {
			console_input: String::new(),
			console_log: vec!["Welcome to Droneforge GUI (MVP)".to_string()],
			focus_console: false,
			paused: false,
			current_z: INITIAL_Z_LEVEL,
			request_rebuild_tiles: true,
			current_tool: Tool::Select,
			toast: None,
			core_hp: (100, 100),
		})
		.insert_resource(SelectionState::default())
		.insert_resource(GameEngine {
			engine: Engine::new(
				GameWorld::from_seed_with_distribution(WORLD_WIDTH, WORLD_HEIGHT, WORLD_LEVELS, RNG_SEED),
				vec![Drone::new(1)],
			),
		})
		// Setup
		.add_systems(Startup, setup_camera)
		// Frame systems
		.add_systems(
			Update,
			(
				handle_pan_zoom,
				handle_selection_input,
				build_tiles_when_needed,
				update_tile_colors_from_world,
				tick_engine_when_running,
				update_toast_timer,
			),
		)
		.add_systems(EguiPrimaryContextPass, draw_ui)
		.run();
}

// ---------- Setup ----------
fn setup_camera(mut commands: Commands) {
        let center_x = (WORLD_WIDTH as f32) * TILE_SIZE * 0.5;
        let center_y = (WORLD_HEIGHT as f32) * TILE_SIZE * 0.5;
        commands.spawn((Camera2d, Transform::from_xyz(center_x, center_y, 1000.0)));
}

// ---------- Utilities ----------
fn tile_color_for_kind(k: TileKind) -> Color {
	match k {
		TileKind::Air => Color::srgb(0.06, 0.06, 0.08),
		TileKind::Stone => Color::srgb(0.6, 0.6, 0.65),
		TileKind::Iron => Color::srgb(0.8, 0.45, 0.2),
		TileKind::Wall => Color::srgb(0.15, 0.15, 0.18),
		TileKind::Floor => Color::srgb(0.25, 0.25, 0.28),
	}
}

fn world_to_tile_coord(p: Vec2) -> (i32, i32) {
	let x = (p.x / TILE_SIZE).floor() as i32;
	let y = (p.y / TILE_SIZE).floor() as i32;
	(x, y)
}

fn screen_to_world_2d(camera_q: &Query<(&Camera, &GlobalTransform)>, screen_pos: Vec2) -> Option<Vec2> {
	let (camera, camera_transform) = camera_q.single().ok()?;
	camera.viewport_to_world_2d(camera_transform, screen_pos).ok()
}

fn set_toast(ui: &mut ResMut<UiState>, msg: impl Into<String>) {
	ui.toast = Some((msg.into(), 2.0));
}

// ---------- Systems: Map Rendering ----------
fn build_tiles_when_needed(
	mut commands: Commands,
	mut ui: ResMut<UiState>,
	engine: Res<GameEngine>,
	existing_layers: Query<Entity, With<TilesLayer>>,
) {
	if !ui.request_rebuild_tiles {
		return;
	}
	// Clear previous tile layer
	for e in &existing_layers {
		commands.entity(e).despawn();
	}
	// Build current z layer tiles
	let z = ui.current_z;
        for y in 0..engine.engine.world.height() {
                for x in 0..engine.engine.world.width() {
                        let k = engine.engine.world.get_tile(TileCoord3 { x, y, z }).unwrap_or(TileKind::Air);
                        let color = tile_color_for_kind(k);
                        let pos = Vec3::new(
                                x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                                y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                                0.0,
                        );
                        let _id = commands
                                .spawn((
                                        Sprite::from_color(color, Vec2::new(TILE_SIZE, TILE_SIZE)),
                                        Transform::from_translation(pos),
                                        Visibility::Visible,
                                        ViewVisibility::HIDDEN,
                                        TilePos { x, y, z },
                                        TilesLayer,
                                ))
                                .id();
                }
        }
	// Done
	ui.request_rebuild_tiles = false;
}

fn update_tile_colors_from_world(
	engine: Res<GameEngine>,
	mut q: Query<(&TilePos, &mut Sprite)>,
) {
	if !engine.is_changed() {
		// We still refresh, but this early return could be enabled if we track diffs
	}
	for (pos, mut sprite) in &mut q {
		if let Some(k) = engine.engine.world.get_tile(TileCoord3 { x: pos.x, y: pos.y, z: pos.z }) {
			let new_color = tile_color_for_kind(k);
			sprite.color = new_color;
		}
	}
}

// ---------- Systems: Camera Pan/Zoom ----------
fn handle_pan_zoom(
	mut ev_motion: EventReader<bevy::input::mouse::MouseMotion>,
	mut ev_wheel: EventReader<bevy::input::mouse::MouseWheel>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	mut q_cam: Query<(&mut Projection, &mut Transform), With<Camera>>,
) {
	let (mut proj, mut cam_transform) = if let Ok(v) = q_cam.single_mut() { v } else { return };

	if mouse_buttons.pressed(MouseButton::Middle) {
		let mut delta = Vec2::ZERO;
		for m in ev_motion.read() {
			delta += m.delta;
		}
		if delta.length_squared() > 0.0 {
			// Pan opposite of mouse drag direction (screen to world)
			cam_transform.translation.x -= delta.x;
			cam_transform.translation.y += delta.y;
		}
	}

	for w in ev_wheel.read() {
		let scroll = w.y;
		let factor = 1.0 - scroll * 0.1;
		if let Projection::Orthographic(ortho) = &mut *proj {
			let new_scale = (ortho.scale * factor).clamp(0.2, 10.0);
			ortho.scale = new_scale;
		}
	}
}

// ---------- Systems: Selection ----------
fn handle_selection_input(
	mut commands: Commands,
	windows: Query<&Window, With<PrimaryWindow>>,
	q_cam: Query<(&Camera, &GlobalTransform)>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	mut selection: ResMut<SelectionState>,
	ui: Res<UiState>,
	mut q_overlay: Query<Entity, With<SelectionOverlay>>,
) {
	// Only active in MineArea mode
	if ui.current_tool != Tool::MineArea {
		// Clear overlay if present
		for e in &mut q_overlay {
			commands.entity(e).despawn();
		}
		selection.is_dragging = false;
		return;
	}

	let window = if let Ok(w) = windows.single() { w } else { return };
	let cursor = if let Some(p) = window.cursor_position() { p } else { return };

	let world_pos = if let Some(wp) = screen_to_world_2d(&q_cam, cursor) { wp } else { return };

	if mouse_buttons.just_pressed(MouseButton::Left) {
		selection.is_dragging = true;
		selection.start_world = world_pos;
		selection.current_world = world_pos;
		// Remove existing overlay
		for e in &mut q_overlay {
			commands.entity(e).despawn();
		}
	}

	if selection.is_dragging && mouse_buttons.pressed(MouseButton::Left) {
		selection.current_world = world_pos;
		// Draw/update a simple debug rectangle as overlay
		let min = selection.start_world.min(selection.current_world);
		let size = (selection.current_world - selection.start_world).abs();
		let center = min + size * 0.5;

		// Recreate overlay fresh (simple approach)
		for e in &mut q_overlay {
			commands.entity(e).despawn();
		}
                let id = commands
                        .spawn((
                                Sprite {
                                        color: Color::srgba(0.2, 0.6, 1.0, 0.15),
                                        custom_size: Some(Vec2::new(size.x.max(1.0), size.y.max(1.0))),
                                        ..Default::default()
                                },
                                Transform::from_translation(Vec3::new(center.x, center.y, 10.0)),
                                SelectionOverlay,
                        ))
                        .id();
                let _border = commands
                        .spawn((
                                Sprite {
                                        color: Color::srgba(0.2, 0.6, 1.0, 0.45),
                                        custom_size: Some(Vec2::new(size.x.max(1.0), size.y.max(1.0))),
                                        ..Default::default()
                                },
                                Transform::from_translation(Vec3::new(center.x, center.y, 11.0)),
                                SelectionOverlay,
                        ))
                        .id();
                let _ = (id, _border);
	}

	if selection.is_dragging && mouse_buttons.just_released(MouseButton::Left) {
		selection.is_dragging = false;
		// Compute snapped tile box at current Z
		let start = selection.start_world;
		let end = selection.current_world;
		let min = start.min(end);
		let max = start.max(end);
		let (min_x, min_y) = world_to_tile_coord(min);
		let (max_x, max_y) = world_to_tile_coord(max);
		let b = TileBox3::new(
			TileCoord3::new(min_x, min_y, ui.current_z),
			TileCoord3::new(max_x, max_y, ui.current_z),
		);
		selection.last_box = Some(b);
		// Overlay remains; console will auto-focus on submit
	}
}

// ---------- Systems: Engine ----------
fn tick_engine_when_running(mut eng: ResMut<GameEngine>, ui: Res<UiState>) {
	if !ui.paused {
		eng.engine.tick();
	}
}

// ---------- Systems: Toast ----------
fn update_toast_timer(time: Res<Time>, mut ui: ResMut<UiState>) {
	if let Some((_, ref mut remaining)) = ui.toast {
		*remaining -= time.delta_secs();
		if *remaining <= 0.0 {
			ui.toast = None;
		}
	}
}

// ---------- Systems: UI ----------
fn draw_ui(
	mut egui_ctx: EguiContexts,
	mut ui: ResMut<UiState>,
	mut eng: ResMut<GameEngine>,
	mut selection: ResMut<SelectionState>,
	mut commands: Commands,
	mut q_overlay: Query<Entity, With<SelectionOverlay>>,
) {
	if let Ok(ctx) = egui_ctx.ctx_mut() {

	// Top HUD
	egui::TopBottomPanel::top("top_hud").show(&*ctx, |ui_top| {
		ui_top.horizontal(|ui_row| {
			let wave_label = "Wave TBD"; // Placeholder
			let hud_text = format_hud(&eng.engine.world.resources, wave_label, ui.core_hp);
			let controls = hud_controls(ui.current_z, ui.paused);
			ui_row.label(hud_text);
			ui_row.separator();
			ui_row.label(&controls.z_readout);
			if ui_row.button(controls.z_up_label).clicked() {
				ui.current_z = (ui.current_z + 1).min(eng.engine.world.levels() - 1);
				ui.request_rebuild_tiles = true;
			}
			if ui_row.button(controls.z_down_label).clicked() {
				ui.current_z = (ui.current_z - 1).max(0);
				ui.request_rebuild_tiles = true;
			}
			ui_row.separator();
			if ui_row.button(&controls.pause_label).clicked() {
				ui.paused = !ui.paused;
			}
			if let Some((ref msg, _)) = ui.toast {
				ui_row.separator();
				ui_row.colored_label(egui::Color32::YELLOW, msg);
			}
		});
	});

	// Right panel (Drones / Tasks)
	egui::SidePanel::right("right_panel")
		.resizable(true)
		.default_width(280.0)
		.show(&*ctx, |ui_right| {
			ui_right.heading(DRONE_PANEL_HEADING);
			egui::ScrollArea::vertical().show(ui_right, |ui_scroll| {
				for d in &eng.engine.drones {
					let status = match d.status {
						DroneStatus::Idle => "Idle",
						DroneStatus::Thinking => "Thinking",
						DroneStatus::Working => "Working",
						DroneStatus::Finished => "Finished",
					};
					let task = d
						.current_task
						.as_ref()
						.map(|t| t.description())
						.unwrap_or_else(|| "None".to_string());
					if ui_scroll.button(format!("Drone #{} — {} — {}", d.id, status, task)).clicked() {
						set_toast(&mut ui, "Centering on drone is not implemented in M1");
					}
				}
			});
			ui_right.separator();
			ui_right.heading(TASK_PANEL_HEADING);
			egui::ScrollArea::vertical().show(ui_right, |ui_scroll| {
				for (t, s) in &eng.engine.tasks.tasks {
					let state = match s {
						TaskState::Pending => "Pending",
						TaskState::InProgress => "InProgress",
						TaskState::Done => "Done",
					};
					let label = format!("{} — {}", t.description(), state);
					if ui_scroll.button(label).clicked() {
						if let Task::MineBox(b) = t {
							selection.last_box = Some(*b);
							set_toast(&mut ui, "Highlighted task area (visual overlay TBD)");
						}
					}
				}
			});
		});

	// Bottom console
	egui::TopBottomPanel::bottom("bottom_console").resizable(true).show(&*ctx, |ui_bottom| {
		ui_bottom.horizontal(|ui_row| {
			let edit = egui::TextEdit::singleline(&mut ui.console_input)
				.hint_text(CONSOLE_HINT);
			let mut response = ui_row.add(edit);
			if ui.focus_console {
				response.request_focus();
				ui.focus_console = false;
			}
			let submit_clicked = ui_row.button(CONSOLE_SUBMIT_LABEL).clicked();
			let enter_pressed = response.lost_focus() && response.ctx.input(|i| i.key_pressed(egui::Key::Enter));
			if submit_clicked || enter_pressed {
				let entered = ui.console_input.trim().to_string();
				if let Some(b) = selection.last_box {
					let program = dsl_ast_program_for_mine_box(b);
					match compile_program_to_tasks(&program) {
						Ok(tasks) => {
							for t in tasks {
								eng.engine.tasks.push(t);
							}
							ui.console_log.push(format!("> {}", entered));
							ui.console_log.push("OK: Created task mine_box".to_string());
							ui.console_input.clear();
						}
						Err(e) => {
							ui.console_log.push(format!("> {}", entered));
							ui.console_log.push(format!("Error: {}", e));
						}
					}
				} else {
					ui.console_log.push(format!("> {}", entered));
					ui.console_log.push("No selection area; drag an area in Mine Area mode".to_string());
				}
			}
		});
		ui_bottom.separator();
		egui::ScrollArea::vertical().stick_to_bottom(true).show(ui_bottom, |ui_logs| {
			for line in &ui.console_log {
				ui_logs.label(line);
			}
		});
	});

	// Tool strip (top-left over map)
	egui::Area::new("tool_strip".into()).fixed_pos(egui::pos2(12.0, 80.0)).show(&*ctx, |ui_area| {
		egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 64)).show(ui_area, |ui_tools| {
			ui_tools.horizontal(|ui_row| {
				let sel = ui.current_tool == Tool::Select;
				if ui_row.selectable_label(sel, TOOL_STRIP_LABELS[0]).clicked() {
					ui.current_tool = Tool::Select;
				}
				let sel = ui.current_tool == Tool::MineArea;
				if ui_row.selectable_label(sel, TOOL_STRIP_LABELS[1]).clicked() {
					ui.current_tool = Tool::MineArea;
					// Prompt flow: area drag first, then console
					ui.focus_console = false;
				}
				let sel = ui.current_tool == Tool::BuildWarrior;
				if ui_row.selectable_label(sel, TOOL_STRIP_LABELS[2]).clicked() {
					ui.current_tool = Tool::BuildWarrior;
					set_toast(&mut ui, "Build Warrior not implemented in M1");
				}
				if ui_row.button(TOOL_STRIP_LABELS[3]).clicked() {
					ui.current_tool = Tool::Select;
					selection.is_dragging = false;
					selection.last_box = None;
					// Remove overlays
					for e in &mut q_overlay {
						commands.entity(e).despawn();
					}
				}
			});
		});
	});

	} // end if Ok(ctx)
}

fn dsl_ast_program_for_mine_box(b: TileBox3) -> Program {
        use serde_json::json;
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
					"min": { "node": "TileCoord", "x": b.min.x, "y": b.min.y, "z": b.min.z },
					"max": { "node": "TileCoord", "x": b.max.x, "y": b.max.y, "z": b.max.z }
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
        serde_json::from_value(program_json).expect("valid program json")
}

#[cfg(test)]
mod tests {
        use super::*;
        use bevy::prelude::{MinimalPlugins, TransformPlugin, Visibility, ViewVisibility};

        #[test]
        fn map_with_iron_and_stone_is_visible() {
                let mut app = App::new();

                app.add_plugins((MinimalPlugins, TransformPlugin));

                let mut world = GameWorld::new(2, 1, 1, TileKind::Stone);
                world.set_tile(TileCoord3 { x: 1, y: 0, z: 0 }, TileKind::Iron);

                app.insert_resource(GameEngine { engine: Engine::new(world, vec![Drone::new(1)]) });
                app.insert_resource(UiState {
                        console_input: String::new(),
                        console_log: vec!["test".to_string()],
                        focus_console: false,
                        paused: false,
                        current_z: INITIAL_Z_LEVEL,
                        request_rebuild_tiles: true,
                        current_tool: Tool::Select,
                        toast: None,
                        core_hp: (100, 100),
                });

                app.add_systems(Startup, (setup_camera, build_tiles_when_needed));

                app.update();
                for _ in 0..3 {
                        app.update();
                }

                assert_tilemap_visible(app.world_mut());
        }

        fn assert_tilemap_visible(world: &mut bevy::prelude::World) {
                let mut tile_query = world.query::<(
                        &TilePos,
                        &GlobalTransform,
                        Option<&Visibility>,
                        Option<&ViewVisibility>,
                )>();

                let camera_transform = camera_transform(world);

                let mut view_query = world.query_filtered::<(&GlobalTransform, &mut ViewVisibility), With<TilesLayer>>();
                for (transform, mut view_visibility) in view_query.iter_mut(world) {
                        if is_in_front_of_camera(transform, &camera_transform) {
                                view_visibility.set();
                        }
                }

                let mut tiles_count = 0;
                let mut stone_visible = false;
                let mut iron_visible = false;

                for (pos, transform, visibility, view_visibility) in tile_query.iter(world) {
                        tiles_count += 1;
                        let visibility_ok = visibility
                                .map(|v| matches!(v, Visibility::Inherited | Visibility::Visible))
                                .unwrap_or(true);
                        let view_vis_ok = view_visibility.map(|vv| vv.get()).unwrap_or(false);
                        let in_front = is_in_front_of_camera(transform, &camera_transform);

                        if visibility_ok && view_vis_ok && in_front {
                                match (pos.x, pos.y) {
                                        (0, 0) => stone_visible = true,
                                        (1, 0) => iron_visible = true,
                                        _ => (),
                                }
                        }
                }

                assert!(tiles_count >= 2, "Expected at least two tiles to be spawned");
                assert!(stone_visible, "Stone tile is not visible to the camera");
                assert!(iron_visible, "Iron tile is not visible to the camera");
        }

        fn camera_transform(world: &mut bevy::prelude::World) -> GlobalTransform {
                let mut cam_query = world.query_filtered::<&GlobalTransform, With<Camera>>();
                *cam_query.single(world).expect("Main camera exists")
        }

        fn is_in_front_of_camera(tile: &GlobalTransform, camera: &GlobalTransform) -> bool {
                let tile_z = tile.translation().z;
                let cam_z = camera.translation().z;
                tile_z < cam_z
        }
}


