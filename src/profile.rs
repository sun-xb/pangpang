use serde_derive::{Serialize, Deserialize};



#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Protocol {
    Ssh(super::ssh::Profile)
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub transport: Option<String>,
    pub protocol: Protocol
}

impl Profile {
    pub fn id(&self) -> String {
        match self.protocol {
            Protocol::Ssh(ref p) => {
                format!("ssh://{}@{}:{}", p.username, p.host, p.port)
            }
        }
    }
}