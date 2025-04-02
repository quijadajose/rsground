use std::sync::{Arc, Mutex};
use crate::models::*;

#[derive(Clone)]
pub struct AppState {
    pub(crate) manager: Arc<Mutex<ProjectManager>>,
}
