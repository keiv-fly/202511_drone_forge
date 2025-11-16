#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TileKind {
	Air,
	Stone,
	Iron,
	Wall,
	Floor,
}

impl TileKind {
	pub fn is_mineable(self) -> bool {
		matches!(self, TileKind::Stone | TileKind::Iron)
	}

	pub fn mined_yield(self) -> Option<ResourceYield> {
		match self {
			TileKind::Stone => Some(ResourceYield::Stone(1)),
			TileKind::Iron => Some(ResourceYield::Iron(1)),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceYield {
	Stone(u32),
	Iron(u32),
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mineable_flags() {
		assert!(TileKind::Stone.is_mineable());
		assert!(TileKind::Iron.is_mineable());
		assert!(!TileKind::Air.is_mineable());
		assert!(!TileKind::Wall.is_mineable());
		assert!(!TileKind::Floor.is_mineable());
	}

	#[test]
	fn mined_yield_values() {
		assert_eq!(TileKind::Stone.mined_yield(), Some(ResourceYield::Stone(1)));
		assert_eq!(TileKind::Iron.mined_yield(), Some(ResourceYield::Iron(1)));
		assert_eq!(TileKind::Air.mined_yield(), None);
	}
}


