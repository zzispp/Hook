import type { BoxProps } from '@mui/material/Box';
import type { IPostHero } from 'src/types/blog';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Avatar from '@mui/material/Avatar';
import Container from '@mui/material/Container';
import SpeedDial from '@mui/material/SpeedDial';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';
import useMediaQuery from '@mui/material/useMediaQuery';
import SpeedDialAction from '@mui/material/SpeedDialAction';

import { fDate } from 'src/utils/format-time';

import { _socials } from 'src/_mock';

import { Iconify } from 'src/components/iconify';
import { useFilePreview } from 'src/components/file-thumbnail';

// ----------------------------------------------------------------------

export function PostDetailsHero({
  sx,
  title,
  author,
  coverUrl,
  createdAt,
  ...other
}: BoxProps & IPostHero) {
  const smUp = useMediaQuery((theme) => theme.breakpoints.up('sm'));
  const { previewUrl } = useFilePreview(coverUrl);

  return (
    <Box
      sx={[
        (theme) => ({
          ...theme.mixins.bgGradient({
            images: [
              `linear-gradient(0deg, ${varAlpha(theme.vars.palette.grey['900Channel'], 0.64)}, ${varAlpha(theme.vars.palette.grey['900Channel'], 0.64)})`,
              `url(${previewUrl})`,
            ],
          }),
          height: 480,
          overflow: 'hidden',
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Container sx={{ height: 1, position: 'relative' }}>
        <Typography
          variant="h3"
          component="h1"
          sx={{
            zIndex: 9,
            maxWidth: 480,
            position: 'absolute',
            pt: { xs: 2, md: 8 },
            color: 'common.white',
          }}
        >
          {title}
        </Typography>

        <Box
          sx={{
            left: 0,
            width: 1,
            bottom: 0,
            position: 'absolute',
          }}
        >
          {author && createdAt && (
            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                px: { xs: 2, md: 3 },
                pb: { xs: 3, md: 8 },
              }}
            >
              <Avatar
                alt={author.name}
                src={author.avatarUrl}
                sx={{ width: 64, height: 64, mr: 2 }}
              />

              <ListItemText
                sx={{ color: 'common.white' }}
                primary={author.name}
                secondary={fDate(createdAt)}
                slotProps={{
                  primary: { sx: { typography: 'subtitle1' } },
                  secondary: { sx: { mt: 0.5, opacity: 0.64, color: 'inherit' } },
                }}
              />
            </Box>
          )}

          <SpeedDial
            direction={smUp ? 'left' : 'up'}
            ariaLabel="Share post"
            icon={<Iconify icon="solar:share-bold" />}
            FabProps={{ size: 'medium' }}
            sx={{ position: 'absolute', bottom: { xs: 32, md: 64 }, right: { xs: 16, md: 24 } }}
          >
            {_socials.map((social) => (
              <SpeedDialAction
                key={social.label}
                icon={
                  <>
                    {social.value === 'twitter' && <Iconify icon="socials:twitter" />}
                    {social.value === 'facebook' && <Iconify icon="socials:facebook" />}
                    {social.value === 'instagram' && <Iconify icon="socials:instagram" />}
                    {social.value === 'linkedin' && <Iconify icon="socials:linkedin" />}
                  </>
                }
                slotProps={{
                  fab: { color: 'default' },
                  tooltip: {
                    placement: 'top',
                    title: social.label,
                  },
                }}
              />
            ))}
          </SpeedDial>
        </Box>
      </Container>
    </Box>
  );
}
