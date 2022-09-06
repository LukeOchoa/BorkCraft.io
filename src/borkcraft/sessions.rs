use serde_derive::{Deserialize, Serialize};

#[derive(Default)]
pub struct SessionInformation {
    pub key: String,
    pub session_time: SessionTime,
    pub is_logged_in: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SessionTime {
    pub key: String,
    pub time: TimeTime,
}
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TimeTime {
    pub hour: String,
    pub minute: String,
    pub second: String,
}

impl SessionInformation {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}
