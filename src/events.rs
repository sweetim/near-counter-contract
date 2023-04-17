use near_sdk::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CounterEventLog {
    pub version: String,
    pub event: String,
    pub data: String,
}

impl std::fmt::Display for CounterEventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &near_sdk::serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        ))
    }
}

impl CounterEventLog {
    pub fn create(action: &crate::CounterAction, value: u128) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            event: "perform_action".to_string(),
            data: format!("perform action ({:?}) = {}", action, value).to_string(),
        }
    }
}
