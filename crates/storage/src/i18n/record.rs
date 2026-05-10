#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{translation_entries, translation_languages};

pub type TranslationEntryRecord = translation_entries::Model;
pub type TranslationLanguageRecord = translation_languages::Model;
