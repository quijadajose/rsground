// src/auth/mod.rs
pub mod handlers;

// Reexportar para facilitar el uso
pub use handlers::{oauth, auth_callback, guest_jwt, health};
