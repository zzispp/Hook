'use client';

import type { IPostItem } from 'src/types/blog';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Divider from '@mui/material/Divider';
import Checkbox from '@mui/material/Checkbox';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';
import AvatarGroup, { avatarGroupClasses } from '@mui/material/AvatarGroup';

import { paths } from 'src/routes/paths';

import { fShortenNumber } from 'src/utils/format-number';

import { Iconify } from 'src/components/iconify';
import { Markdown } from 'src/components/markdown';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { PostItem } from '../post-item';
import { PostCommentList } from '../post-comment-list';
import { PostCommentForm } from '../post-comment-form';
import { PostDetailsHero } from '../post-details-hero';

// ----------------------------------------------------------------------

type Props = {
  post?: IPostItem;
  latestPosts?: IPostItem[];
};

export function PostDetailsHomeView({ post, latestPosts }: Props) {
  return (
    <>
      <PostDetailsHero
        title={post?.title ?? ''}
        author={post?.author}
        coverUrl={post?.coverUrl ?? ''}
        createdAt={post?.createdAt}
      />

      <Container
        maxWidth={false}
        sx={[
          (theme) => ({ py: 3, mb: 5, borderBottom: `solid 1px ${theme.vars.palette.divider}` }),
        ]}
      >
        <CustomBreadcrumbs
          links={[
            { name: 'Home', href: '/' },
            { name: 'Blog', href: paths.post.root },
            { name: post?.title },
          ]}
          sx={{ maxWidth: 720, mx: 'auto' }}
        />
      </Container>

      <Container maxWidth={false}>
        <Stack sx={{ maxWidth: 720, mx: 'auto' }}>
          <Typography variant="subtitle1">{post?.description}</Typography>

          <Markdown children={post?.content} />

          <Stack
            spacing={3}
            sx={[
              (theme) => ({
                py: 3,
                borderTop: `dashed 1px ${theme.vars.palette.divider}`,
                borderBottom: `dashed 1px ${theme.vars.palette.divider}`,
              }),
            ]}
          >
            <Box sx={{ gap: 1, display: 'flex', flexWrap: 'wrap' }}>
              {post?.tags.map((tag) => (
                <Chip key={tag} label={tag} variant="soft" />
              ))}
            </Box>

            <Box sx={{ display: 'flex', alignItems: 'center' }}>
              <FormControlLabel
                label={fShortenNumber(post?.totalFavorites)}
                control={
                  <Checkbox
                    defaultChecked
                    size="small"
                    color="error"
                    icon={<Iconify icon="solar:heart-bold" />}
                    checkedIcon={<Iconify icon="solar:heart-bold" />}
                    slotProps={{
                      input: {
                        id: 'favorite-checkbox',
                        'aria-label': 'Favorite checkbox',
                      },
                    }}
                  />
                }
                sx={{ mr: 1 }}
              />

              <AvatarGroup
                sx={{
                  [`& .${avatarGroupClasses.avatar}`]: {
                    width: 32,
                    height: 32,
                  },
                }}
              >
                {post?.favoritePerson.map((person) => (
                  <Avatar key={person.name} alt={person.name} src={person.avatarUrl} />
                ))}
              </AvatarGroup>
            </Box>
          </Stack>

          <Box sx={{ mb: 3, mt: 5, display: 'flex' }}>
            <Typography variant="h4">Comments</Typography>

            <Typography variant="subtitle2" sx={{ color: 'text.disabled' }}>
              ({post?.comments.length})
            </Typography>
          </Box>

          <PostCommentForm />

          <Divider sx={{ mt: 5, mb: 2 }} />

          <PostCommentList comments={post?.comments} />
        </Stack>
      </Container>

      {!!latestPosts?.length && (
        <Container sx={{ pb: 15 }}>
          <Typography variant="h4" sx={{ mb: 5 }}>
            Recent Posts
          </Typography>

          <Grid container spacing={3}>
            {latestPosts?.slice(latestPosts.length - 4).map((latestPost) => (
              <Grid
                key={latestPost.id}
                size={{
                  xs: 12,
                  sm: 6,
                  md: 4,
                  lg: 3,
                }}
              >
                <PostItem post={latestPost} detailsHref={paths.post.details(latestPost.title)} />
              </Grid>
            ))}
          </Grid>
        </Container>
      )}
    </>
  );
}
