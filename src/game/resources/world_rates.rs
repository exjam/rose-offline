pub struct WorldRates {
    pub xp_rate: i32,
    pub drop_rate: i32,
    pub drop_money_rate: i32,
}

impl WorldRates {
    pub fn new() -> Self {
        Self {
            xp_rate: 300,
            drop_rate: 300,
            drop_money_rate: 300,
        }
    }
}
