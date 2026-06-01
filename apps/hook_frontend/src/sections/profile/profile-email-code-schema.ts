import * as z from 'zod';

import { trimCredential } from 'src/auth/context/jwt/validation';

const EMAIL_CODE_LENGTH = 6;
const EMAIL_CODE_PATTERN = /^\d{6}$/;

type CommonT = (key: string, options?: Record<string, unknown>) => string;

export type ProfileEmailCodeSchemaType = z.infer<ReturnType<typeof profileEmailCodeSchema>>;

export function profileEmailCodeSchema(t: CommonT) {
  return z.object({
    email: z.string(),
    code: emailCodeSchema(t),
  });
}

export function profileEmailCodeDefaultValues(email: string): ProfileEmailCodeSchemaType {
  return {
    email,
    code: '',
  };
}

function emailCodeSchema(t: CommonT) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z.string().regex(EMAIL_CODE_PATTERN, {
        error: () => t('profile.validation.emailCodeInvalid', { length: EMAIL_CODE_LENGTH }),
      })
    );
}
