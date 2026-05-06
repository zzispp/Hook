import type { NextRequest } from 'next/server';

import { NextResponse } from 'next/server';

import { CONFIG } from './global-config';

// ----------------------------------------------------------------------

export function middleware(req: NextRequest) {
  const origin = req.headers.get('origin') || '';

  const { methods, allowedOrigins } = CONFIG.cors;

  let allowedOrigin = allowedOrigins.length === 0 ? '*' : '';

  if (allowedOrigins.includes(origin)) {
    allowedOrigin = origin;
  }

  const res = NextResponse.next();
  res.headers.set('Access-Control-Allow-Origin', allowedOrigin);
  res.headers.set('Access-Control-Allow-Methods', methods.join(','));
  res.headers.set('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  res.headers.set('Access-Control-Allow-Credentials', 'true');

  if (req.method === 'OPTIONS') {
    return new Response(null, { status: 204, headers: res.headers });
  }

  return res;
}

export const config = {
  matcher: '/api/:path*',
};
