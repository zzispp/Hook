import { _mock } from './_mock';
import { _tags } from './assets';

// ----------------------------------------------------------------------

const COLORS = [
  '#FF4842',
  '#1890FF',
  '#FFC0CB',
  '#00AB55',
  '#FFC107',
  '#7F00FF',
  '#000000',
  '#FFFFFF',
];

const DESCRIPTION = `
<h6>Specifications</h6>
<table>
  <tbody>
    <tr>
      <td>Category</td>
      <td>Mobile</td>
    </tr>
    <tr>
      <td>Manufacturer</td>
      <td>Apple</td>
    </tr>
    <tr>
      <td>Warranty</td>
      <td>12 Months</td>
    </tr>
    <tr>
      <td>Serial number</td>
      <td>358607726380311</td>
    </tr>
    <tr>
      <td>Ships from</td>
      <td>United States</td>
    </tr>
  </tbody>
</table>

<h6>Product details</h6>
<ul>
  <li>
    <p>The foam sockliner feels soft and comfortable</p>
  </li>
  <li>
    <p>Pull tab</p>
  </li>
  <li>
    <p>Not intended for use as Personal Protective Equipment</p>
  </li>
  <li>
    <p>Colour Shown: White/Black/Oxygen Purple/Action Grape</p>
  </li>
  <li>
    <p>Style: 921826-109</p>
  </li>
  <li>
    <p>Country/Region of Origin: China</p>
  </li>
</ul>
<h6>Benefits</h6>
<ul>
  <li>
    <p>Mesh and synthetic materials on the upper keep the fluid look of the OG while adding comfort</p>
    and durability.
  </li>
  <li>
    <p>Originally designed for performance running, the full-length Max Air unit adds soft, comfortable cushio</p>
    ning underfoot.
  </li>
  <li>
    <p>The foam midsole feels springy and soft.</p>
  </li>
  <li>
    <p>The rubber outsole adds traction and durability.</p>
  </li>
</ul>
<h6>Delivery and returns</h6>
<p>Your order of $200 or more gets free standard delivery.</p>
<ul>
  <li>
    <p>Standard delivered 4-5 Business Days</p>
  </li>
  <li>
    <p>Express delivered 2-4 Business Days</p>
  </li>
</ul>
<p>Orders are processed and delivered Monday-Friday (excluding public holidays)</p>

`;

const generateAttachments = () =>
  Array.from({ length: 20 }, (_, index) => _mock.image.product(index));

const generateReviews = () => {
  const attachments = generateAttachments();

  return Array.from({ length: 8 }, (_, index) => ({
    id: _mock.id(index),
    name: _mock.fullName(index),
    postedAt: _mock.time(index),
    comment: _mock.sentence(index),
    isPurchased: _mock.boolean(index),
    rating: _mock.number.rating(index),
    avatarUrl: _mock.image.avatar(index),
    helpful: _mock.number.nativeL(index),
    attachments:
      (index === 1 && attachments.slice(0, 1)) ||
      (index === 3 && attachments.slice(2, 4)) ||
      (index === 5 && attachments.slice(5, 8)) ||
      [],
  }));
};

const getColorSliceForIndex = (index: number) => {
  if (index === 0) return COLORS.slice(0, 2);
  if (index === 1) return COLORS.slice(1, 3);
  if (index === 2) return COLORS.slice(2, 4);
  if (index === 3) return COLORS.slice(3, 6);
  if (index === 4 || index === 16 || index === 19) return COLORS.slice(4, 6);
  if (index === 5 || index === 17) return COLORS.slice(5, 6);
  if (index === 6 || index === 18) return COLORS.slice(0, 2);
  if (index === 7) return COLORS.slice(4, 6);
  if (index === 8) return COLORS.slice(2, 4);
  if (index === 9 || index === 11) return COLORS.slice(2, 6);
  if (index === 10) return COLORS.slice(3, 6);
  if (index === 12) return COLORS.slice(2, 7);
  if (index === 13) return COLORS.slice(4, 7);
  if (index === 14) return COLORS.slice(0, 2);
  if (index === 15) return COLORS.slice(5, 8);
  return COLORS.slice(2, 6); // Default case
};

const generateRatings = () =>
  Array.from({ length: 5 }, (_, index) => ({
    name: `${index + 1} Star`,
    starCount: _mock.number.nativeL(index),
    reviewCount: _mock.number.nativeL(index + 1),
  }));

const generateImages = () => Array.from({ length: 8 }, (_, index) => _mock.image.product(index));

// ----------------------------------------------------------------------

export const _products = () =>
  Array.from({ length: 20 }, (_, index) => {
    const reviews = generateReviews();
    const images = generateImages();
    const ratings = generateRatings();

    const publish = index % 3 ? 'published' : 'draft';

    const category = (index % 2 && 'Shose') || (index % 3 && 'Apparel') || 'Accessories';

    const gender = (index % 2 && ['Men']) || (index % 3 && ['Women', 'Kids']) || ['Kids'];

    const available = (index % 2 && 72) || (index % 3 && 10) || 0;

    const inventoryType = (index % 2 && 'in stock') || (index % 3 && 'low stock') || 'out of stock';

    const priceSale = index % 3 ? null : _mock.number.price(index);

    return {
      id: _mock.id(index),
      gender,
      images,
      reviews,
      publish,
      ratings,
      category,
      available,
      priceSale,
      taxes: 10,
      quantity: 80,
      inventoryType,
      tags: _tags.slice(0, 5),
      code: `38BEE27${index}`,
      description: DESCRIPTION,
      sku: `WW75K521${index}YW/SV`,
      createdAt: _mock.time(index),
      name: _mock.productName(index),
      price: _mock.number.price(index),
      coverUrl: _mock.image.product(index),
      colors: getColorSliceForIndex(index),
      totalRatings: _mock.number.rating(index),
      totalSold: _mock.number.nativeM(index + 1),
      totalReviews: _mock.number.nativeL(index + 1),
      newLabel: { enabled: [1, 2, 3].includes(index), content: 'NEW' },
      saleLabel: { enabled: [4, 5].includes(index), content: 'SALE' },
      sizes: ['6', '7', '8', '8.5', '9', '9.5', '10', '10.5', '11', '11.5', '12', '13'],
      subDescription:
        'Featuring the original ripple design inspired by Japanese bullet trains, the Nike Air Max 97 lets you push your style full-speed ahead.',
    };
  });
