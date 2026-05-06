import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

// ----------------------------------------------------------------------

export const runtime = 'edge';

const DEFAULT_PAGE = 1;
const DEFAULT_PER_PAGE = 10;
const TOTAL_PRODUCTS = 50;

const _products = Array.from({ length: TOTAL_PRODUCTS }, (_, index) => ({
  id: `id-${index + 1}`,
  name: `product-${index + 1}`,
  category: (index % 2 && 'Accessories') || (index % 3 && 'Shoes') || 'Clothing',
}));

type Products = typeof _products;

/** **************************************
 * Products with pagination and filters
 *************************************** */
export async function GET(req: NextRequest) {
  const { searchParams } = req.nextUrl;

  const pageParam = searchParams.get('page') ?? `${DEFAULT_PAGE}`;
  const perPageParam = searchParams.get('perPage') ?? `${DEFAULT_PER_PAGE}`;

  const page = parseInt(pageParam, 10);
  const perPage = parseInt(perPageParam, 10);
  const searchQuery = searchParams.get('search')?.trim().toLowerCase() ?? '';
  const category = searchParams.get('category')?.trim() ?? '';

  try {
    const filteredProducts = filterProducts(_products, searchQuery, category);
    const paginatedProducts = paginateProducts(filteredProducts, page, perPage);

    const totalPages = Math.ceil(filteredProducts.length / perPage);
    const totalItems = filteredProducts.length;

    logger('[Product] filtered-products', filteredProducts.length);

    return response(
      {
        products: paginatedProducts,
        totalPages,
        totalItems,
        categoryOptions: Array.from(
          new Set(_products.map(({ category: c_category }) => c_category))
        ), // Remove duplicate categories
      },
      STATUS.OK
    );
  } catch (error) {
    return handleError('Pagination - Get list of products', error);
  }
}

// ----------------------------------------------------------------------

function paginateProducts(products: Products, page: number, perPage: number) {
  const startIndex = (page - 1) * perPage;
  const endIndex = startIndex + perPage;

  return products.slice(startIndex, endIndex);
}

function filterProducts(products: Products, searchQuery: string, category: string) {
  return products.filter(({ id, name, category: prodCategory }) => {
    // Accept search by id or name
    const matchesSearch = searchQuery
      ? id.includes(searchQuery) || name.toLowerCase().includes(searchQuery)
      : true;
    const matchesCategory = category ? prodCategory === category : true;

    return matchesSearch && matchesCategory;
  });
}
