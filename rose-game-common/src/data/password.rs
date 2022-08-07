use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Password {
    Plaintext(String),
    Md5(String),
}

impl Password {
    pub fn to_md5(&self) -> String {
        match self {
            Password::Plaintext(plaintext) => format!("{:x}", md5::compute(plaintext)),
            Password::Md5(md5) => md5.clone(),
        }
    }
}
