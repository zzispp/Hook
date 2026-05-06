'use client';

import { varAlpha } from 'minimal-shared/utils';
import { useCountdownDate } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';
import { outlinedInputClasses } from '@mui/material/OutlinedInput';

import { _socials } from 'src/_mock';
import { ComingSoonIllustration } from 'src/assets/illustrations';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

export function ComingSoonView() {
  const countdown = useCountdownDate(new Date('2026-08-20 20:30'));

  return (
    <Container>
      <Typography variant="h3" sx={{ mb: 2 }}>
        Coming soon!
      </Typography>

      <Typography sx={{ color: 'text.secondary' }}>
        We are currently working hard on this page!
      </Typography>

      <ComingSoonIllustration sx={{ my: { xs: 5, sm: 10 } }} />

      <Stack
        divider={<Box sx={{ mx: { xs: 1, sm: 2.5 } }}>:</Box>}
        sx={{ typography: 'h2', justifyContent: 'center', flexDirection: 'row' }}
      >
        <TimeBlock label="days" value={countdown.days} />
        <TimeBlock label="hours" value={countdown.hours} />
        <TimeBlock label="minutes" value={countdown.minutes} />
        <TimeBlock label="seconds" value={countdown.seconds} />
      </Stack>

      <TextField
        fullWidth
        placeholder="Enter your email"
        slotProps={{
          input: {
            endAdornment: (
              <InputAdornment position="end">
                <Button variant="contained" size="large">
                  Notify me
                </Button>
              </InputAdornment>
            ),
            sx: [
              (theme) => ({
                pr: 0.5,
                [`&.${outlinedInputClasses.focused}`]: {
                  boxShadow: theme.vars.customShadows.z20,
                  transition: theme.transitions.create(['box-shadow'], {
                    duration: theme.transitions.duration.shorter,
                  }),
                  [`& .${outlinedInputClasses.notchedOutline}`]: {
                    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.32)}`,
                  },
                },
              }),
            ],
          },
        }}
        sx={{ my: 5 }}
      />
      <Box sx={{ display: 'flex', justifyContent: 'center' }}>
        {_socials.map((social) => (
          <IconButton key={social.label}>
            {social.value === 'twitter' && <Iconify icon="socials:twitter" />}
            {social.value === 'facebook' && <Iconify icon="socials:facebook" />}
            {social.value === 'instagram' && <Iconify icon="socials:instagram" />}
            {social.value === 'linkedin' && <Iconify icon="socials:linkedin" />}
          </IconButton>
        ))}
      </Box>
    </Container>
  );
}

// ----------------------------------------------------------------------

type TimeBlockProps = {
  label: string;
  value: string;
};

function TimeBlock({ label, value }: TimeBlockProps) {
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column' }}>
      <span> {value} </span>
      <Box component="span" sx={{ color: 'text.secondary', typography: 'body1' }}>
        {label}
      </Box>
    </Box>
  );
}
