'use client';

import { useState, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Paper from '@mui/material/Paper';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';
import AccordionSummary from '@mui/material/AccordionSummary';
import AccordionDetails from '@mui/material/AccordionDetails';
import Accordion, { accordionClasses } from '@mui/material/Accordion';

import { _mock } from 'src/_mock';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const ACCORDIONS = Array.from({ length: 4 }, (_, index) => ({
  id: _mock.id(index),
  value: `panel${index + 1}`,
  title: `Accordion ${index + 1}`,
  subheader: _mock.postTitle(index),
  content: _mock.description(index),
  disabled: index === 2,
}));

// ----------------------------------------------------------------------

export function AccordionView() {
  const [disableGutters, setDisableGutters] = useState(false);
  const [controlled, setControlled] = useState<string | false>('panel1');

  const handleChangeControlled = useCallback(
    (panel: string) => (event: React.SyntheticEvent, isExpanded: boolean) => {
      setControlled(isExpanded ? panel : false);
    },
    []
  );

  const handleToggleDisableGutters = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setDisableGutters(event.target.checked);
  }, []);

  const renderTitle = (title: string, disabled: boolean, subheader?: boolean) => (
    <Typography
      component="span"
      variant="subtitle1"
      sx={{
        ...(subheader && { width: '33%', flexShrink: 0 }),
      }}
    >
      {title} {!!disabled && '(disabled)'}
    </Typography>
  );

  const renderSubheader = (subheader: string) => (
    <Typography component="span" variant="body2" sx={{ color: 'text.secondary' }}>
      {subheader}
    </Typography>
  );

  const renderDetails = (content: string) => (
    <AccordionDetails sx={{ typography: 'body2', color: 'text.secondary' }}>
      {content}
    </AccordionDetails>
  );

  const getA11yProps = (prefix: string, id: string) => ({
    id: `${prefix}-panel${id}-header`,
    'aria-controls': `${prefix}-panel${id}-content`,
  });

  const DEMO_COMPONENTS = [
    {
      name: 'Simple',
      component: (
        <ComponentBox>
          <div>
            {ACCORDIONS.map((item, index) => (
              <Accordion key={item.id} disabled={item.disabled} defaultExpanded={index === 0}>
                <AccordionSummary {...getA11yProps('simple', item.id)}>
                  {renderTitle(item.title, item.disabled)}
                </AccordionSummary>
                {renderDetails(item.content)}
              </Accordion>
            ))}
          </div>
        </ComponentBox>
      ),
    },

    {
      name: 'Controlled',
      action: (
        <FormControlLabel
          label="Disable gutters"
          control={
            <Switch
              checked={disableGutters}
              onChange={handleToggleDisableGutters}
              slotProps={{ input: { id: `disable-gutters-switch` } }}
            />
          }
        />
      ),

      component: (
        <ComponentBox sx={{ p: 5, gap: 0 }}>
          {ACCORDIONS.map((item) => (
            <Accordion
              key={item.id}
              disabled={item.disabled}
              expanded={controlled === item.value}
              onChange={handleChangeControlled(item.value)}
              disableGutters={disableGutters}
            >
              <AccordionSummary {...getA11yProps('controlled', item.id)}>
                {renderTitle(item.title, item.disabled, true)}
                {renderSubheader(item.subheader)}
              </AccordionSummary>
              {renderDetails(item.content)}
            </Accordion>
          ))}
        </ComponentBox>
      ),
    },
    {
      name: 'Custom styles',
      component: (
        <>
          <ComponentBox title="Wrapper" sx={{ mb: 5, gap: 0 }}>
            <Paper variant="outlined">
              {ACCORDIONS.map((item, index) => (
                <Accordion
                  key={item.id}
                  disableGutters
                  disabled={item.disabled}
                  defaultExpanded={index === 2}
                  sx={{ px: 2 }}
                >
                  <AccordionSummary {...getA11yProps('wrapper', item.id)}>
                    {renderTitle(item.title, item.disabled)}
                  </AccordionSummary>
                  {renderDetails(item.content)}
                </Accordion>
              ))}
            </Paper>
          </ComponentBox>

          <ComponentBox title="Standalone" sx={{ rowGap: 1 }}>
            {ACCORDIONS.map((item, index) => (
              <Accordion
                key={item.id}
                disableGutters
                disabled={item.disabled}
                defaultExpanded={index === 0}
                sx={[
                  (theme) => ({
                    py: 1,
                    px: 2,
                    border: 'none',
                    borderRadius: 2,
                    bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
                    '&:hover': {
                      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.12),
                    },
                    [`&.${accordionClasses.disabled}`]: {
                      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
                    },
                    [`&.${accordionClasses.expanded}`]: {
                      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
                    },
                  }),
                ]}
              >
                <AccordionSummary
                  {...getA11yProps('standalone', item.id)}
                  expandIcon={<Iconify icon="eva:arrow-ios-downward-fill" />}
                >
                  {renderTitle(item.title, item.disabled)}
                </AccordionSummary>
                {renderDetails(item.content)}
              </Accordion>
            ))}
          </ComponentBox>
        </>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Accordion',
        moreLinks: ['https://mui.com/material-ui/react-accordion/'],
      }}
    />
  );
}
