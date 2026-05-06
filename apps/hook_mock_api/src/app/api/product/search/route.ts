import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _products } from 'src/_mock/_product';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Search products
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const query = searchParams.get('query')?.trim().toLowerCase();

    if (!query) {
      return response({ results: [] }, STATUS.OK);
    }

    const products = _products();

    // Accept search by name or sku
    const results = products.filter(
      ({ name, sku }) => name.toLowerCase().includes(query) || sku?.toLowerCase().includes(query)
    );

    logger('[Product] search-results', results.length);

    return response({ results }, STATUS.OK);
  } catch (error) {
    return handleError('Product - Get search', error);
  }
}
