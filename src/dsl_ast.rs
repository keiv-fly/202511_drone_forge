use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coords::{TileBox3, TileCoord3};
use crate::tasks::Task;

#[derive(Debug, Error)]
pub enum CompileError {
	#[error("Invalid program root")]
	InvalidRoot,
	#[error("Unsupported node: {0}")]
	UnsupportedNode(String),
	#[error("Unknown variable: {0}")]
	UnknownVar(String),
	#[error("Invalid argument")]
	InvalidArg,
	#[error("Schema error: {0}")]
	SchemaError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
	pub version: u32,
	pub node: String,
	pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "node")]
pub enum Statement {
	Let {
		name: String,
		ty: String,
		value: Expr,
	},
	ExprStmt {
		expr: Expr,
	},
	ForIn {
		var: Var,
		#[serde(rename = "iter")]
		iter_expr: Expr,
		body: Vec<Statement>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Var {
	pub name: String,
	pub ty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "node")]
pub enum Expr {
	TileBoxFromCoords {
		min: Coord,
		max: Coord,
	},
	TileCoord {
		x: i32,
		y: i32,
		#[serde(default)]
		z: i32,
	},
	VarRef {
		name: String,
	},
	Call {
		func: String,
		args: Vec<Expr>,
	},
	IterTiles {
		#[serde(rename = "box")]
		r#box: Box<Expr>,
	},
	IntLiteral {
		value: i64,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coord {
	pub node: String,
	pub x: i32,
	pub y: i32,
	#[serde(default)]
	pub z: i32,
}

#[derive(Default)]
struct Scope {
	vars: std::collections::HashMap<String, Expr>,
}

fn expr_to_box3(e: &Expr, scope: &Scope) -> Result<TileBox3, CompileError> {
	match e {
		Expr::TileBoxFromCoords { min, max } => {
			if min.node != "TileCoord" || max.node != "TileCoord" {
				return Err(CompileError::SchemaError("TileBoxFromCoords needs TileCoord".into()));
			}
			Ok(TileBox3::new(
				TileCoord3 { x: min.x, y: min.y, z: min.z },
				TileCoord3 { x: max.x, y: max.y, z: max.z },
			))
		}
		Expr::VarRef { name } => {
			let bound = scope.vars.get(name).ok_or_else(|| CompileError::UnknownVar(name.clone()))?;
			expr_to_box3(bound, scope)
		}
		_ => Err(CompileError::InvalidArg),
	}
}

pub fn compile_program_to_tasks(p: &Program) -> Result<Vec<Task>, CompileError> {
	if p.node != "Program" {
		return Err(CompileError::InvalidRoot);
	}
	let mut scope = Scope::default();
	let mut tasks = Vec::new();
	for stmt in &p.statements {
		match stmt {
			Statement::Let { name, ty, value } => {
				if ty != "TileBox" {
					return Err(CompileError::UnsupportedNode(format!("Let type {}", ty)));
				}
				scope.vars.insert(name.clone(), value.clone());
			}
			Statement::ExprStmt { expr } => {
				match expr {
					Expr::Call { func, args } if func == "mine_box" => {
						if args.len() != 1 {
							return Err(CompileError::InvalidArg);
						}
						let b = expr_to_box3(&args[0], &scope)?;
						tasks.push(Task::MineBox(b));
					}
					_ => return Err(CompileError::UnsupportedNode("Only mine_box supported in M1".into())),
				}
			}
			Statement::ForIn { .. } => {
				return Err(CompileError::UnsupportedNode("ForIn not supported in M1".into()));
			}
		}
	}
	Ok(tasks)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn parse_and_compile_mine_box() {
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
						"min": { "node": "TileCoord", "x": 0, "y": 0, "z": 0 },
						"max": { "node": "TileCoord", "x": 1, "y": 1, "z": 0 }
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
		let prog: Program = serde_json::from_value(program_json).unwrap();
		let tasks = compile_program_to_tasks(&prog).unwrap();
		assert_eq!(tasks.len(), 1);
		match &tasks[0] {
			Task::MineBox(b) => {
				assert_eq!(b.width(), 2);
				assert_eq!(b.height(), 2);
			}
		}
	}
}


