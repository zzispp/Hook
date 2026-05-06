import { SignJWT, jwtVerify } from 'jose';

// ----------------------------------------------------------------------

type Token = {
  userId?: string;
  iat?: number;
  exp?: number;
};

/**
 * SignJWT
 * https://github.com/panva/jose/blob/main/docs/classes/jwt_sign.SignJWT.md
 */
export async function sign(
  payload: Token,
  secret: string,
  options: { expiresIn: string | number }
): Promise<string> {
  const iat = Math.floor(Date.now() / 1000);

  return new SignJWT({ ...payload })
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setIssuedAt(iat)
    .setExpirationTime(options.expiresIn)
    .sign(new TextEncoder().encode(secret));
}

/**
 * jwtVerify
 * https://github.com/panva/jose/blob/HEAD/docs/functions/jwt_verify.jwtVerify.md
 */
export async function verify(token: string, secret: string): Promise<Token> {
  const { payload } = await jwtVerify(token, new TextEncoder().encode(secret));

  return payload;
}
