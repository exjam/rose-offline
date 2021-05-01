#[derive(Clone)]
pub struct Monster {
    pub id: u32,
}

impl Monster {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}
