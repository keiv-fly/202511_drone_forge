use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl TileCoord3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileBox3 {
    pub min: TileCoord3, // inclusive
    pub max: TileCoord3, // inclusive
}

impl TileBox3 {
    pub fn new(min: TileCoord3, max: TileCoord3) -> Self {
        assert!(
            min.x <= max.x && min.y <= max.y && min.z <= max.z,
            "Invalid TileBox3 bounds"
        );
        Self { min, max }
    }

    pub fn contains(&self, c: TileCoord3) -> bool {
        c.x >= self.min.x
            && c.x <= self.max.x
            && c.y >= self.min.y
            && c.y <= self.max.y
            && c.z >= self.min.z
            && c.z <= self.max.z
    }

    pub fn width(&self) -> i32 {
        self.max.x - self.min.x + 1
    }

    pub fn height(&self) -> i32 {
        self.max.y - self.min.y + 1
    }

    pub fn levels(&self) -> i32 {
        self.max.z - self.min.z + 1
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = TileCoord3> {
        let min = self.min;
        let max = self.max;
        (min.z..=max.z).flat_map(move |z| {
            (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| TileCoord3 { x, y, z }))
        })
    }

    pub fn border_tiles(&self) -> impl Iterator<Item = TileCoord3> {
        let min = self.min;
        let max = self.max;
        let has_x_thickness = min.x != max.x;
        let has_y_thickness = min.y != max.y;
        let has_z_thickness = min.z != max.z;
        self.iter_tiles().filter(move |c| {
            (has_x_thickness && (c.x == min.x || c.x == max.x))
                || (has_y_thickness && (c.y == min.y || c.y == max.y))
                || (has_z_thickness && (c.z == min.z || c.z == max.z))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_and_contains() {
        let b = TileBox3::new(TileCoord3::new(1, 2, 3), TileCoord3::new(2, 3, 3));
        assert!(b.contains(TileCoord3::new(1, 2, 3)));
        assert!(b.contains(TileCoord3::new(2, 3, 3)));
        assert!(!b.contains(TileCoord3::new(0, 2, 3)));
    }

    #[test]
    fn sizes() {
        let b = TileBox3::new(TileCoord3::new(0, 0, 0), TileCoord3::new(2, 3, 4));
        assert_eq!(b.width(), 3);
        assert_eq!(b.height(), 4);
        assert_eq!(b.levels(), 5);
    }

    #[test]
    fn iter_counts() {
        let b = TileBox3::new(TileCoord3::new(0, 0, 0), TileCoord3::new(1, 1, 0));
        let tiles: Vec<_> = b.iter_tiles().collect();
        assert_eq!(tiles.len(), 4);
    }

    #[test]
    fn border_includes_edges() {
        let b = TileBox3::new(TileCoord3::new(0, 0, 0), TileCoord3::new(2, 2, 0));
        let all: Vec<_> = b.iter_tiles().collect();
        let border: Vec<_> = b.border_tiles().collect();
        assert!(border.len() < all.len());
        assert!(
            border
                .iter()
                .all(|c| c.x == 0 || c.x == 2 || c.y == 0 || c.y == 2 || c.z == 0)
        );
    }
}
