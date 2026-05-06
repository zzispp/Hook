'use client';

import type { IPostItem } from 'src/types/blog';

import { paths } from 'src/routes/paths';

import { DashboardContent } from 'src/layouts/dashboard';

import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { PostCreateEditForm } from '../post-create-edit-form';

// ----------------------------------------------------------------------

type Props = {
  post?: IPostItem;
};

export function PostEditView({ post }: Props) {
  return (
    <DashboardContent>
      <CustomBreadcrumbs
        heading="Edit"
        backHref={paths.dashboard.post.root}
        links={[
          { name: 'Dashboard', href: paths.dashboard.root },
          { name: 'Blog', href: paths.dashboard.post.root },
          { name: post?.title },
        ]}
        sx={{ mb: { xs: 3, md: 5 } }}
      />

      <PostCreateEditForm currentPost={post} />
    </DashboardContent>
  );
}
