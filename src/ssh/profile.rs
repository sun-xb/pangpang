use serde_derive::{Serialize, Deserialize};



#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String
}