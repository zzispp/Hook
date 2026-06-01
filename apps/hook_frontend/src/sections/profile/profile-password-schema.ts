import * as z from 'zod';

import { trimCredential, PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH } from 'src/auth/context/jwt/validation';

import { profileEmailCodeSchema } from './profile-email-code-schema';

type CommonT = (key: string, options?: Record<string, unknown>) => string;

export type ProfilePasswordSchemaType = z.infer<ReturnType<typeof profilePasswordSchema>>;

export function profilePasswordSchema(t: CommonT) {
  return profileEmailCodeSchema(t)
    .extend({
      password: passwordSchema(t),
      confirmPassword: z.string().transform(trimCredential),
    })
    .refine((value) => value.password === value.confirmPassword, {
      path: ['confirmPassword'],
      error: () => t('profile.validation.passwordMismatch'),
    });
}

export function profilePasswordWithCurrentSchema(t: CommonT) {
  return z
    .object({
      currentPassword: passwordSchema(t),
      password: passwordSchema(t),
      confirmPassword: z.string().transform(trimCredential),
    })
    .refine((value) => value.password === value.confirmPassword, {
      path: ['confirmPassword'],
      error: () => t('profile.validation.passwordMismatch'),
    });
}

export type ProfilePasswordWithCurrentSchemaType = z.infer<
  ReturnType<typeof profilePasswordWithCurrentSchema>
>;

export function profilePasswordDefaultValues(email: string) {
  return {
    email,
    code: '',
    password: '',
    confirmPassword: '',
  };
}

export function profilePasswordWithCurrentDefaultValues() {
  return {
    currentPassword: '',
    password: '',
    confirmPassword: '',
  };
}

function passwordSchema(t: CommonT) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(PASSWORD_MIN_LENGTH, { error: () => passwordLengthMessage(t) })
        .max(PASSWORD_MAX_LENGTH, { error: () => passwordLengthMessage(t) })
    );
}

function passwordLengthMessage(t: CommonT) {
  return t('profile.validation.passwordLength', {
    min: PASSWORD_MIN_LENGTH,
    max: PASSWORD_MAX_LENGTH,
  });
}
