import { headers } from 'next/headers';

import { verify } from 'src/utils/jwt';
import { STATUS, response, handleError } from 'src/utils/response';

import { _users, JWT_SECRET } from 'src/_mock/_auth';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/**
 * This API is used for demo purpose only
 * You should use a real database
 * You should hash the password before saving to database
 * You should not save the password in the database
 * You should not expose the JWT_SECRET in the client side
 */

export async function GET() {
  try {
    const headersList = headers();
    const authorization = headersList.get('authorization');

    if (!authorization || !authorization.startsWith('Bearer ')) {
      return response({ message: 'Authorization token missing or invalid' }, STATUS.UNAUTHORIZED);
    }

    const accessToken = `${authorization}`.split(' ')[1];
    const data = await verify(accessToken, JWT_SECRET);

    const currentUser = _users.find((user) => user.id === data.userId);

    if (!currentUser) {
      return response({ message: 'Invalid authorization token' }, STATUS.UNAUTHORIZED);
    }

    return response({ user: currentUser }, 200);
  } catch (error) {
    return handleError('[Auth] - Me', error);
  }
}
