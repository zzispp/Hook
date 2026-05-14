use std::sync::Arc;

use crate::application::CardCodeUseCase;

#[derive(Clone)]
pub struct CardCodeApiState {
    pub card_codes: Arc<dyn CardCodeUseCase>,
}

impl CardCodeApiState {
    pub fn new(card_codes: Arc<dyn CardCodeUseCase>) -> Self {
        Self { card_codes }
    }
}
