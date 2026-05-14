#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{card_code_types, card_codes};

pub type CardCodeTypeRecord = card_code_types::Model;
pub type CardCodeRecord = card_codes::Model;
