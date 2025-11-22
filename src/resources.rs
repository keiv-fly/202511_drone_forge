#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resources {
    pub stone: u32,
    pub iron: u32,
}

impl Resources {
    pub fn new() -> Self {
        Self { stone: 0, iron: 0 }
    }

    pub fn add_stone(&mut self, amount: u32) {
        self.stone = self.stone.saturating_add(amount);
    }

    pub fn add_iron(&mut self, amount: u32) {
        self.iron = self.iron.saturating_add(amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_add() {
        let mut r = Resources::default();
        r.add_stone(3);
        r.add_iron(2);
        assert_eq!(r.stone, 3);
        assert_eq!(r.iron, 2);
    }
}
