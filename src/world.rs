use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::coords::TileCoord3;
use crate::resources::Resources;
use crate::tile::{ResourceYield, TileKind};

#[derive(Debug, Clone)]
pub struct World {
    width: i32,
    height: i32,
    levels: i32,
    tiles: Vec<TileKind>,
    pub resources: Resources,
    core_hp: u32,
    core_hp_max: u32,
}

impl World {
    pub fn new(width: i32, height: i32, levels: i32, fill: TileKind) -> Self {
        let size = (width as usize) * (height as usize) * (levels as usize);
        let core_hp_max = 100;
        Self {
            width,
            height,
            levels,
            tiles: vec![fill; size],
            resources: Resources::default(),
            core_hp: core_hp_max,
            core_hp_max,
        }
    }

    pub fn from_seed_with_distribution(width: i32, height: i32, levels: i32, seed: u64) -> Self {
        let mut world = Self::new(width, height, levels, TileKind::Air);
        let mut rng = StdRng::seed_from_u64(seed);
        for z in 0..levels {
            for y in 0..height {
                for x in 0..width {
                    let roll: f32 = rng.r#gen();
                    let kind = if roll < 0.10 {
                        TileKind::Iron
                    } else if roll < 0.55 {
                        TileKind::Stone
                    } else {
                        TileKind::Air
                    };
                    world.set_tile(TileCoord3 { x, y, z }, kind);
                }
            }
        }
        world
    }

    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn levels(&self) -> i32 {
        self.levels
    }
    pub fn core_hp(&self) -> (u32, u32) {
        (self.core_hp, self.core_hp_max)
    }

    fn index(&self, c: TileCoord3) -> Option<usize> {
        if c.x < 0
            || c.y < 0
            || c.z < 0
            || c.x >= self.width
            || c.y >= self.height
            || c.z >= self.levels
        {
            return None;
        }
        let idx = ((c.z * self.height + c.y) * self.width + c.x) as usize;
        Some(idx)
    }

    pub fn get_tile(&self, c: TileCoord3) -> Option<TileKind> {
        self.index(c).map(|i| self.tiles[i])
    }

    pub fn set_tile(&mut self, c: TileCoord3, k: TileKind) {
        if let Some(i) = self.index(c) {
            self.tiles[i] = k;
        }
    }

    pub fn mine_tile(&mut self, c: TileCoord3) -> Option<ResourceYield> {
        let i = self.index(c)?;
        let k = self.tiles[i];
        if let Some(y) = k.mined_yield() {
            self.tiles[i] = TileKind::Air;
            match y {
                ResourceYield::Stone(n) => self.resources.add_stone(n),
                ResourceYield::Iron(n) => self.resources.add_iron(n),
            }
            Some(y)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_bounds() {
        let w = World::new(3, 3, 2, TileKind::Air);
        assert!(w.index(TileCoord3 { x: 0, y: 0, z: 0 }).is_some());
        assert!(w.index(TileCoord3 { x: 2, y: 2, z: 1 }).is_some());
        assert!(w.index(TileCoord3 { x: -1, y: 0, z: 0 }).is_none());
        assert!(w.index(TileCoord3 { x: 0, y: 3, z: 0 }).is_none());
        assert!(w.index(TileCoord3 { x: 0, y: 0, z: 2 }).is_none());
    }

    #[test]
    fn mining_updates_resources() {
        let mut w = World::new(2, 1, 1, TileKind::Air);
        let c = TileCoord3 { x: 0, y: 0, z: 0 };
        w.set_tile(c, TileKind::Stone);
        let y = w.mine_tile(c);
        assert!(y.is_some());
        assert_eq!(w.get_tile(c), Some(TileKind::Air));
        assert_eq!(w.resources.stone, 1);
        assert_eq!(w.resources.iron, 0);
    }
}
