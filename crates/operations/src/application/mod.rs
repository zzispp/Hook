mod error;
mod ports;
mod service;
mod validation;

pub use error::{OperationsError, OperationsResult};
pub use ports::{OperationsRepository, OperationsUseCase, TicketCaptchaVerifier, TicketEmail, TicketMailer};
pub use service::{OperationsService, is_admin_role};
