# Repository Guidelines

## Project Structure & Modules
- `src/lib.rs` exposes the core modules; keep new exports minimal.
- Gameplay core: `src/world.rs` (grid + tile access), `src/tile.rs` (tile kinds), `src/resources.rs` (resource counters), `src/drones.rs` (drone state), `src/tasks.rs` (task queue + execution), `src/engine.rs` (tick loop). DSL parsing lives in `src/dsl_ast.rs`; HUD formatting in `src/hud.rs`.
- Frontend: `src/bin/gui.rs` hosts the Bevy + egui MVP. It builds the world, draws tiles, and wires mouse selection to DSL-generated tasks.
- Tests: unit tests co-located in modules; integration coverage in `tests/integration_end_to_end.rs`.
- Design notes and mocks: `design/*.md`. Target features should align with these docs before coding.

## Build, Test, and Development Commands
- `cargo fmt` — format the Rust sources.
- `cargo clippy --all-targets --all-features -- -D warnings` — lint and fail on warnings.
- `cargo test` — run unit and integration tests (headless).
- `cargo run --bin gui` — launch the Bevy/egui GUI; useful for smoke-testing input and tile rendering.

## Coding Style & Naming Conventions
- Follow `rustfmt` defaults (4-space indentation, max line width 100). Do not hand-edit whitespace; rerun `cargo fmt`.
- Modules/files use `snake_case`; types and traits use `CamelCase`; functions, methods, and variables use `snake_case`.
- Prefer small, composable functions; keep `Engine::tick`-style flows readable with early returns over deep nesting.
- Add comments sparingly to explain non-obvious state transitions or coordinate math; avoid restating the code.

## Testing Guidelines
- Add unit tests near new logic; mirror existing patterns that construct small `World` instances and assert tile/resource changes.
- Integration tests belong under `tests/`; prefer end-to-end flows similar to `integration_end_to_end.rs`, validating DSL → tasks → engine results.
- When fixing bugs, add a regression test that fails pre-fix. Aim for deterministic setups (seeded worlds, fixed task queues).

## Commit & Pull Request Guidelines
- Commits: concise, imperative subject lines (e.g., “Add HUD formatting for task list”), scoped to one logical change. Reference issue IDs when applicable.
- PRs: include a short summary of behavior changes, test results (`cargo fmt`, `cargo clippy`, `cargo test`), and any GUI notes/screens where visuals change.
- Keep PRs focused: gameplay logic, DSL changes, and rendering tweaks should be separate where possible to ease review.
