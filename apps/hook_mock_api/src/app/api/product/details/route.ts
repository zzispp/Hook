import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _products } from 'src/_mock/_product';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * Get product details
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const productId = searchParams.get('productId');

    const products = _products();

    const product = products.find((productItem) => productItem.id === productId);

    if (!product) {
      return response({ message: 'Product not found!' }, STATUS.NOT_FOUND);
    }

    logger('[Product] details', product.id);

    return response({ product }, STATUS.OK);
  } catch (error) {
    return handleError('Product - Get details', error);
  }
}
