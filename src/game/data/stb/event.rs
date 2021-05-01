use crate::game::data::formats::StbFile;

pub struct StbEvent(pub StbFile);

#[allow(dead_code)]
impl StbEvent {
    pub fn lookup_row_name(&self, name: &str) -> Option<usize> {
        self.0.lookup_row_name(name)
    }

    pub fn get_name(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 0)
    }

    pub fn get_type(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 1)
    }

    pub fn get_description(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 2)
    }

    pub fn get_filename(&self, row: usize) -> Option<&str> {
        self.0.try_get(row, 3)
    }
}
