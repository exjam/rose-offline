pub struct WorldRates {
    pub xp_rate: i32,
}

impl WorldRates {
    pub fn new() -> Self {
        Self { xp_rate: 300 }
    }
}
