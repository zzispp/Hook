'use client';

import { paths } from 'src/routes/paths';

import { DashboardContent } from 'src/layouts/dashboard';

import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { ProductCreateEditForm } from '../product-create-edit-form';

// ----------------------------------------------------------------------

export function ProductCreateView() {
  return (
    <DashboardContent>
      <CustomBreadcrumbs
        heading="Create a new product"
        links={[
          { name: 'Dashboard', href: paths.dashboard.root },
          { name: 'Product', href: paths.dashboard.product.root },
          { name: 'Create' },
        ]}
        sx={{ mb: { xs: 3, md: 5 } }}
      />

      <ProductCreateEditForm />
    </DashboardContent>
  );
}
