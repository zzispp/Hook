import type { NextRequest } from 'next/server';

import { sign } from 'src/utils/jwt';
import { STATUS, response, handleError } from 'src/utils/response';

import { _users, JWT_SECRET, JWT_EXPIRES_IN } from 'src/_mock/_auth';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/**
 * This API is used for demo purpose only
 * You should use a real database
 * You should hash the password before saving to database
 * You should not save the password in the database
 * You should not expose the JWT_SECRET in the client side
 */

export async function POST(req: NextRequest) {
  try {
    const { email, password } = await req.json();

    const currentUser = _users.find((user) => user.email === email);

    if (!currentUser) {
      return response(
        { message: 'There is no user corresponding to the email address.' },
        STATUS.UNAUTHORIZED
      );
    }

    if (currentUser?.password !== password) {
      return response({ message: 'Wrong password' }, STATUS.UNAUTHORIZED);
    }

    const accessToken = await sign({ userId: currentUser?.id }, JWT_SECRET, {
      expiresIn: JWT_EXPIRES_IN,
    });

    return response({ user: currentUser, accessToken }, 200);
  } catch (error) {
    return handleError('Auth - Sign in', error);
  }
}
