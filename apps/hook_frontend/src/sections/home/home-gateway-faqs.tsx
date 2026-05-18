import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';

import { useState } from 'react';
import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import AccordionDetails from '@mui/material/AccordionDetails';
import AccordionSummary from '@mui/material/AccordionSummary';
import Accordion, { accordionClasses } from '@mui/material/Accordion';

import { useTranslate } from 'src/locales/use-locales';

import { varFade, MotionViewport } from 'src/components/animate';

import { SectionTitle } from './components/section-title';
import { FloatLine, FloatTriangleDownIcon } from './components/svg-elements';

// ----------------------------------------------------------------------

const FAQ_KEYS = ['sdk', 'selfHosted', 'quota', 'monitoring'] as const;

type FAQKey = (typeof FAQ_KEYS)[number];

type TFunction = ReturnType<typeof useTranslate>['t'];

export function HomeGatewayFAQs({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');
  const [expanded, setExpanded] = useState<FAQKey | false>(FAQ_KEYS[0]);

  const handleChange = (panel: FAQKey) => (_event: React.SyntheticEvent, isExpanded: boolean) => {
    setExpanded(isExpanded ? panel : false);
  };

  return (
    <Box component="section" sx={sx} {...other}>
      <MotionViewport sx={viewportSx}>
        <DecorativeLines />

        <Container>
          <SectionTitle
            caption={t('home.faqs.caption')}
            title={t('home.faqs.title')}
            txtGradient={t('home.faqs.gradient')}
            sx={{ textAlign: 'center' }}
          />

          <Box sx={contentSx}>
            {FAQ_KEYS.map((item, index) => (
              <FAQItem
                key={item}
                item={item}
                t={t}
                index={index}
                expanded={expanded === item}
                onChange={handleChange(item)}
              />
            ))}
          </Box>
        </Container>
      </MotionViewport>
    </Box>
  );
}

// ----------------------------------------------------------------------

function DecorativeLines() {
  return (
    <>
      <Stack spacing={8} alignItems="center" sx={lineStackSx}>
        <FloatTriangleDownIcon sx={{ position: 'static', opacity: 0.12 }} />
        <FloatTriangleDownIcon sx={{ width: 30, height: 15, opacity: 0.24, position: 'static' }} />
      </Stack>

      <FloatLine vertical sx={{ top: 0, left: 80 }} />
    </>
  );
}

type FAQItemProps = {
  t: TFunction;
  index: number;
  expanded: boolean;
  onChange: (event: React.SyntheticEvent, isExpanded: boolean) => void;
  item: FAQKey;
};

function FAQItem({ item, t, index, expanded, onChange }: FAQItemProps) {
  return (
    <Accordion
      disableGutters
      component={m.div}
      variants={varFade('inUp', { distance: 24 })}
      expanded={expanded}
      onChange={onChange}
      sx={accordionSx}
    >
      <AccordionSummary id={`home-faqs-panel${index}-header`} aria-controls={`home-faqs-panel${index}-content`}>
        <Typography component="span" variant="h6">
          {t(`home.faqs.items.${item}.question`)}
        </Typography>
      </AccordionSummary>

      <AccordionDetails>
        <Typography sx={{ color: 'text.secondary' }}>{t(`home.faqs.items.${item}.answer`)}</Typography>
      </AccordionDetails>
    </Accordion>
  );
}

// ----------------------------------------------------------------------

const viewportSx = {
  py: { xs: 10, md: 14 },
  position: 'relative',
};

const lineStackSx = {
  top: 64,
  left: 80,
  position: 'absolute',
  transform: 'translateX(-50%)',
};

const contentSx = {
  gap: 1,
  mt: 8,
  mx: 'auto',
  maxWidth: 760,
  display: 'flex',
  flexDirection: 'column',
};

const accordionSx: SxProps<Theme> = [
  (theme) => ({
    py: 1,
    px: 2.5,
    border: 'none',
    borderRadius: 2,
    transition: theme.transitions.create(['background-color'], {
      duration: theme.transitions.duration.shorter,
    }),
    '&:hover': {
      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
    },
    [`&.${accordionClasses.expanded}`]: {
      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
    },
  }),
];
