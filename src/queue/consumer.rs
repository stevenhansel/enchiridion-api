use std::sync::{Arc, Mutex};

pub struct Consumer {
    client: Arc<Mutex<redis::Connection>>,
    queue_name: String,
}
