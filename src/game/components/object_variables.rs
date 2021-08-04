pub struct ObjectVariables {
    pub variables: Vec<i32>,
}

impl ObjectVariables {
    pub fn new(count: usize) -> Self {
        Self {
            variables: vec![0; count],
        }
    }
}
