import type { NextRequest } from 'next/server';

import { sign } from 'src/utils/jwt';
import { STATUS, response, handleError } from 'src/utils/response';

import { _users, JWT_SECRET, JWT_EXPIRES_IN } from 'src/_mock/_auth';

// ----------------------------------------------------------------------

/**
 * This API is used for demo purpose only
 * You should use a real database
 * You should hash the password before saving to database
 * You should not save the password in the database
 * You should not expose the JWT_SECRET in the client side
 */

export const runtime = 'edge';

export async function POST(req: NextRequest) {
  try {
    const { email, password, firstName, lastName } = await req.json();

    const userExists = _users.find((user) => user.email === email);

    if (userExists) {
      return response(
        { message: 'There already exists an account with the given email address.' },
        STATUS.CONFLICT
      );
    }

    const newUser = {
      id: _users[0].id,
      displayName: `${firstName} ${lastName}`,
      email,
      password,
      photoURL: '',
      phoneNumber: '',
      country: '',
      address: '',
      state: '',
      city: '',
      zipCode: '',
      about: '',
      role: 'user',
      isPublic: true,
    };

    const accessToken = await sign({ userId: newUser.id }, JWT_SECRET, {
      expiresIn: JWT_EXPIRES_IN,
    });

    // Push new user to database
    _users.push(newUser);

    return response({ user: newUser, accessToken }, STATUS.OK);
  } catch (error) {
    return handleError('Auth - Sign up', error);
  }
}
