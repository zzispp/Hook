import { fSub } from 'src/utils/set-date';

import { CONFIG } from 'src/global-config';

import {
  _id,
  _ages,
  _roles,
  _prices,
  _emails,
  _ratings,
  _nativeS,
  _nativeM,
  _nativeL,
  _percents,
  _booleans,
  _sentences,
  _lastNames,
  _fullNames,
  _tourNames,
  _jobTitles,
  _taskNames,
  _fileNames,
  _postTitles,
  _firstNames,
  _eventNames,
  _courseNames,
  _fullAddress,
  _companyNames,
  _productNames,
  _descriptions,
  _phoneNumbers,
  _countryNames,
} from './assets';

// ----------------------------------------------------------------------

export const _mock = {
  id: (index: number) => _id[index],
  // time: (index: number) => `2024-06-${(index + 1).toString().padStart(2, '0')}T23:00:00.000Z`,
  time: (index: number) => fSub({ days: index, hours: index }),
  boolean: (index: number) => _booleans[index],
  role: (index: number) => _roles[index],
  // Text
  courseNames: (index: number) => _courseNames[index],
  fileNames: (index: number) => _fileNames[index],
  eventNames: (index: number) => _eventNames[index],
  taskNames: (index: number) => _taskNames[index],
  postTitle: (index: number) => _postTitles[index],
  jobTitle: (index: number) => _jobTitles[index],
  tourName: (index: number) => _tourNames[index],
  productName: (index: number) => _productNames[index],
  sentence: (index: number) => _sentences[index],
  description: (index: number) => _descriptions[index],
  // Contact
  email: (index: number) => _emails[index],
  phoneNumber: (index: number) => _phoneNumbers[index],
  fullAddress: (index: number) => _fullAddress[index],
  // Name
  firstName: (index: number) => _firstNames[index],
  lastName: (index: number) => _lastNames[index],
  fullName: (index: number) => _fullNames[index],
  companyNames: (index: number) => _companyNames[index],
  countryNames: (index: number) => _countryNames[index],
  // Number
  number: {
    percent: (index: number) => _percents[index],
    rating: (index: number) => _ratings[index],
    age: (index: number) => _ages[index],
    price: (index: number) => _prices[index],
    nativeS: (index: number) => _nativeS[index],
    nativeM: (index: number) => _nativeM[index],
    nativeL: (index: number) => _nativeL[index],
  },
  // Image
  image: {
    cover: (index: number) => `${CONFIG.basePath}/assets/images/cover/cover-${index + 1}.webp`,
    avatar: (index: number) => `${CONFIG.basePath}/assets/images/avatar/avatar-${index + 1}.webp`,
    travel: (index: number) => `${CONFIG.basePath}/assets/images/travel/travel-${index + 1}.webp`,
    course: (index: number) => `${CONFIG.basePath}/assets/images/course/course-${index + 1}.webp`,
    company: (index: number) =>
      `${CONFIG.basePath}/assets/images/company/company-${index + 1}.webp`,
    product: (index: number) =>
      `${CONFIG.basePath}/assets/images/m-product/product-${index + 1}.webp`,
    portrait: (index: number) =>
      `${CONFIG.basePath}/assets/images/portrait/portrait-${index + 1}.webp`,
  },
};
