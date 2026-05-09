#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::api_tokens;

pub type ApiTokenRecord = api_tokens::Model;
