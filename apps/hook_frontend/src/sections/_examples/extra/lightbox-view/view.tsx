'use client';

import type { Slide, SlideImage, SlideVideo } from 'yet-another-react-lightbox';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Switch from '@mui/material/Switch';
import FormLabel from '@mui/material/FormLabel';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';

import { _mock } from 'src/_mock';

import { Lightbox, useLightbox } from 'src/components/lightbox';

import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const IMAGES = Array.from({ length: 4 }, (_, index) => ({
  src: _mock.image.cover(index + 1),
  title: 'Flamingo',
  description: 'Vicko Mozara \n Veliki zali, Dubravica, Croatia',
}));

const SLIDES: Slide[] = [
  ...IMAGES,
  {
    type: 'video',
    width: 1280,
    height: 720,
    poster:
      'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/images/BigBuckBunny.jpg',
    sources: [
      {
        src: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4',
        type: 'video/mp4',
      },
    ],
  },
];

const defaultState = {
  disableZoom: false,
  disableVideo: false,
  disableTotal: false,
  disableCaptions: false,
  disableSlideshow: false,
  disableThumbnails: false,
  disableFullscreen: false,
} as const;

type State = typeof defaultState;
type Keys = keyof State;

// ----------------------------------------------------------------------

export function LightboxView() {
  const lightbox = useLightbox(SLIDES);
  const [state, setState] = useState<State>(defaultState);

  const handleChange = useCallback((event: React.ChangeEvent<HTMLInputElement>, name: Keys) => {
    setState((prev) => ({
      ...prev,
      [name]: event.target.checked,
    }));
  }, []);

  const renderControls = (name: Keys) => (
    <FormControlLabel
      key={name}
      label={name}
      control={
        <Switch
          size="small"
          checked={state[name]}
          onChange={(event) => handleChange(event, name)}
          slotProps={{ input: { id: `${name}-switch` } }}
        />
      }
    />
  );

  return (
    <>
      <ComponentLayout
        heroProps={{
          heading: 'Lightbox',
          moreLinks: ['https://www.npmjs.com/package/yet-another-react-lightbox'],
        }}
      >
        <Card sx={{ p: 1, gap: 2, display: 'flex', alignItems: 'flex-start' }}>
          <Box
            sx={{
              gap: 1,
              display: 'grid',
              gridTemplateColumns: {
                xs: 'repeat(2, 1fr)',
                sm: 'repeat(3, 1fr)',
                md: 'repeat(4, 1fr)',
              },
            }}
          >
            {SLIDES.map((slide) => {
              const thumbnail =
                slide.type === 'video' ? (slide as SlideVideo).poster : (slide as SlideImage).src;

              return (
                <Box
                  component="img"
                  key={thumbnail}
                  alt={thumbnail}
                  src={thumbnail}
                  onClick={() => lightbox.onOpen(`${thumbnail}`)}
                  sx={{
                    width: 240,
                    borderRadius: 1,
                    cursor: 'pointer',
                    aspectRatio: '1/1',
                    objectFit: 'cover',
                  }}
                />
              );
            })}
          </Box>

          <Box
            sx={[
              {
                p: 2.5,
                width: 220,
                flexShrink: 0,
                borderRadius: 1.5,
                bgcolor: 'background.neutral',
              },
            ]}
          >
            <FormControl component="fieldset" variant="standard">
              <FormLabel component="legend" sx={{ mb: 2, typography: 'body2' }}>
                Controls
              </FormLabel>

              <Box sx={{ gap: 2, display: 'flex', flexDirection: 'column' }}>
                {(Object.keys(state) as Keys[]).map(renderControls)}
              </Box>
            </FormControl>
          </Box>
        </Card>
      </ComponentLayout>

      <Lightbox
        open={lightbox.open}
        close={lightbox.onClose}
        slides={SLIDES}
        index={lightbox.selected}
        {...state}
      />
    </>
  );
}
