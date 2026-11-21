use std::sync::Arc;

pub type EventEmitter = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;
