export const STATUS = {
  OK: 200,
  ERROR: 500,
  CONFLICT: 409,
  NOT_FOUND: 404,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
};

export function response(data: string | Record<string, unknown>, status: number): Response {
  const value = typeof data === 'string' ? data : JSON.stringify(data);

  return new Response(value, {
    status,
    headers: {
      'Content-Type': 'application/json',
    },
  });
}

export function handleError(message: string, error: unknown): Response {
  console.error(`[${message}]: `, error);
  return response({ message: 'Internal server error' }, STATUS.ERROR);
}
