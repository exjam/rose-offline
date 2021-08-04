pub struct ObjectVariables {
    pub variables: Vec<i32>,
}

impl ObjectVariables {
    pub fn new(count: usize) -> Self {
        let mut variables = Vec::with_capacity(count);
        variables.resize(count, 0);
        Self { variables }
    }
}
