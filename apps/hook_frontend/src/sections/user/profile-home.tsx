import type { GridProps } from '@mui/material/Grid';
import type { IUserProfile, IUserProfilePost } from 'src/types/user';

import { useRef } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Fab from '@mui/material/Fab';
import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import InputBase from '@mui/material/InputBase';
import CardHeader from '@mui/material/CardHeader';

import { fNumber } from 'src/utils/format-number';

import { _socials } from 'src/_mock';

import { Iconify } from 'src/components/iconify';

import { ProfilePostItem } from './profile-post-item';

// ----------------------------------------------------------------------

type Props = GridProps & {
  info: IUserProfile;
  posts: IUserProfilePost[];
};

export function ProfileHome({ info, posts, sx, ...other }: Props) {
  const fileRef = useRef<HTMLInputElement>(null);

  const handleAttach = () => {
    if (fileRef.current) {
      fileRef.current.click();
    }
  };

  const renderFollows = () => (
    <Card sx={{ py: 3, textAlign: 'center', typography: 'h4' }}>
      <Stack
        divider={<Divider orientation="vertical" flexItem sx={{ borderStyle: 'dashed' }} />}
        sx={{ flexDirection: 'row' }}
      >
        <Stack sx={{ width: 1 }}>
          {fNumber(info.totalFollowers)}
          <Box component="span" sx={{ color: 'text.secondary', typography: 'body2' }}>
            Follower
          </Box>
        </Stack>

        <Stack sx={{ width: 1 }}>
          {fNumber(info.totalFollowing)}
          <Box component="span" sx={{ color: 'text.secondary', typography: 'body2' }}>
            Following
          </Box>
        </Stack>
      </Stack>
    </Card>
  );

  const renderAbout = () => (
    <Card>
      <CardHeader title="About" />

      <Box
        sx={{
          p: 3,
          gap: 2,
          display: 'flex',
          typography: 'body2',
          flexDirection: 'column',
        }}
      >
        <div>{info.quote}</div>

        <Box sx={{ gap: 2, display: 'flex', lineHeight: '24px' }}>
          <Iconify width={24} icon="mingcute:location-fill" />
          <span>
            Live at
            <Link variant="subtitle2" color="inherit">
              &nbsp;{info.country}
            </Link>
          </span>
        </Box>

        <Box sx={{ gap: 2, display: 'flex', lineHeight: '24px' }}>
          <Iconify width={24} icon="solar:letter-bold" />
          {info.email}
        </Box>

        <Box sx={{ gap: 2, display: 'flex', lineHeight: '24px' }}>
          <Iconify width={24} icon="solar:case-minimalistic-bold" />
          <span>
            {info.role} at
            <Link variant="subtitle2" color="inherit">
              &nbsp;{info.company}
            </Link>
          </span>
        </Box>

        <Box sx={{ gap: 2, display: 'flex', lineHeight: '24px' }}>
          <Iconify width={24} icon="solar:case-minimalistic-bold" />
          <span>
            Studied at
            <Link variant="subtitle2" color="inherit">
              &nbsp;{info.school}
            </Link>
          </span>
        </Box>
      </Box>
    </Card>
  );

  const renderPostInput = () => (
    <Card sx={{ p: 3 }}>
      <InputBase
        multiline
        fullWidth
        rows={4}
        placeholder="Share what you are thinking here..."
        inputProps={{ id: 'post-input' }}
        sx={[
          (theme) => ({
            p: 2,
            mb: 3,
            borderRadius: 1,
            border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.2)}`,
          }),
        ]}
      />

      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
          <Fab size="small" color="inherit" variant="softExtended" onClick={handleAttach}>
            <Iconify icon="solar:gallery-wide-bold" width={24} sx={{ color: 'success.main' }} />
            Image/Video
          </Fab>
          <Fab size="small" color="inherit" variant="softExtended">
            <Iconify icon="solar:videocamera-record-bold" width={24} sx={{ color: 'error.main' }} />
            Streaming
          </Fab>
        </Box>

        <Button variant="contained">Post</Button>
      </Box>

      <input ref={fileRef} type="file" style={{ display: 'none' }} />
    </Card>
  );

  const renderSocials = () => (
    <Card>
      <CardHeader title="Social" />

      <Box sx={{ p: 3, gap: 2, display: 'flex', flexDirection: 'column', typography: 'body2' }}>
        {_socials.map((social) => (
          <Box
            key={social.label}
            sx={{
              gap: 2,
              display: 'flex',
              lineHeight: '20px',
              wordBreak: 'break-all',
              alignItems: 'flex-start',
            }}
          >
            {social.value === 'twitter' && <Iconify icon="socials:twitter" />}
            {social.value === 'facebook' && <Iconify icon="socials:facebook" />}
            {social.value === 'instagram' && <Iconify icon="socials:instagram" />}
            {social.value === 'linkedin' && <Iconify icon="socials:linkedin" />}

            <Link color="inherit">
              {social.value === 'facebook' && info.socialLinks.facebook}
              {social.value === 'instagram' && info.socialLinks.instagram}
              {social.value === 'linkedin' && info.socialLinks.linkedin}
              {social.value === 'twitter' && info.socialLinks.twitter}
            </Link>
          </Box>
        ))}
      </Box>
    </Card>
  );

  return (
    <Grid container spacing={3} sx={sx} {...other}>
      <Grid size={{ xs: 12, md: 4 }} sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
        {renderFollows()}
        {renderAbout()}
        {renderSocials()}
      </Grid>

      <Grid size={{ xs: 12, md: 8 }} sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
        {renderPostInput()}

        {posts.map((post) => (
          <ProfilePostItem key={post.id} post={post} />
        ))}
      </Grid>
    </Grid>
  );
}
