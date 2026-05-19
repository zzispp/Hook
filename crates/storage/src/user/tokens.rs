use sea_orm::Set;

use super::{
    PasswordResetTokenRecord, RegistrationEmailVerificationRecord, password_reset_tokens,
    password_reset_tokens::ActiveModel as PasswordResetTokenActiveModel, registration_email_verifications,
    registration_email_verifications::ActiveModel as RegistrationEmailVerificationActiveModel,
};

pub(super) fn password_reset_token_record(record: password_reset_tokens::Model) -> PasswordResetTokenRecord {
    PasswordResetTokenRecord {
        id: record.id,
        user_id: record.user_id,
        token_hash: record.token_hash,
        expires_at: record.expires_at,
        consumed_at: record.consumed_at,
        created_at: record.created_at,
    }
}

pub(super) fn password_reset_token_active_model(record: PasswordResetTokenRecord) -> PasswordResetTokenActiveModel {
    PasswordResetTokenActiveModel {
        id: Set(record.id),
        user_id: Set(record.user_id),
        token_hash: Set(record.token_hash),
        expires_at: Set(record.expires_at),
        consumed_at: Set(record.consumed_at),
        created_at: Set(record.created_at),
    }
}

pub(super) fn registration_email_verification_record(record: registration_email_verifications::Model) -> RegistrationEmailVerificationRecord {
    RegistrationEmailVerificationRecord {
        id: record.id,
        email: record.email,
        code_hash: record.code_hash,
        expires_at: record.expires_at,
        consumed_at: record.consumed_at,
        created_at: record.created_at,
    }
}

pub(super) fn registration_email_verification_active_model(
    record: RegistrationEmailVerificationRecord,
) -> RegistrationEmailVerificationActiveModel {
    RegistrationEmailVerificationActiveModel {
        id: Set(record.id),
        email: Set(record.email),
        code_hash: Set(record.code_hash),
        expires_at: Set(record.expires_at),
        consumed_at: Set(record.consumed_at),
        created_at: Set(record.created_at),
    }
}
