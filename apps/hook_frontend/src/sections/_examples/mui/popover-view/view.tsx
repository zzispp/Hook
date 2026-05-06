'use client';

import type { ButtonProps } from '@mui/material/Button';
import type { IconButtonProps } from '@mui/material/IconButton';
import type { ArrowPlacement } from 'src/components/custom-popover';

import { varAlpha } from 'minimal-shared/utils';
import { useState, Fragment, useCallback } from 'react';
import { usePopover, usePopoverHover } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/components/iconify';
import { CustomPopover } from 'src/components/custom-popover';

import { ComponentBox, contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const ARROW_PLACEMENTS: Record<'top' | 'bottom' | 'left' | 'right', ArrowPlacement[]> = {
  top: ['top-left', 'top-center', 'top-right'],
  bottom: ['bottom-left', 'bottom-center', 'bottom-right'],
  left: ['left-top', 'left-center', 'left-bottom'],
  right: ['right-top', 'right-center', 'right-bottom'],
};

// ----------------------------------------------------------------------

export function PopoverView() {
  const clickPopover = usePopover();
  const customizedPopover = usePopover();
  const hoverPopover = usePopoverHover<HTMLButtonElement>();

  const [useIconButton, setUseIconButton] = useState(false);
  const [activePlacement, setActivePlacement] = useState<ArrowPlacement | null>(null);

  const handleToggleIconButton = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setUseIconButton(event.target.checked);
  }, []);

  const handleCustomizedOpen = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>, placement: ArrowPlacement) => {
      customizedPopover.onOpen(event);
      setActivePlacement(placement);
    },
    [customizedPopover]
  );

  const handleCustomizedClose = useCallback(() => {
    customizedPopover.onClose();
    setActivePlacement(null);
  }, [customizedPopover]);

  const renderPopoverContent = () => (
    <Box sx={{ p: 2, maxWidth: 280 }}>
      <Typography variant="subtitle1" sx={{ mb: 1 }}>
        Etiam feugiat lorem non metus
      </Typography>
      <Typography variant="body2" sx={{ color: 'text.secondary' }}>
        Fusce vulputate eleifend sapien. Curabitur at lacus ac velit ornare lobortis.
      </Typography>
    </Box>
  );

  const renderClickPopover = () => (
    <>
      <Button variant="contained" onClick={clickPopover.onOpen}>
        Click popover
      </Button>

      <CustomPopover
        open={clickPopover.open}
        onClose={clickPopover.onClose}
        anchorEl={clickPopover.anchorEl}
        slotProps={{ arrow: { placement: 'top-center' } }}
      >
        {renderPopoverContent()}
      </CustomPopover>
    </>
  );

  const renderHoverPopover = () => (
    <>
      <Button
        ref={hoverPopover.elementRef}
        variant="outlined"
        onMouseEnter={hoverPopover.onOpen}
        onMouseLeave={hoverPopover.onClose}
      >
        Hover popover
      </Button>

      {hoverPopover.open && (
        <CustomPopover
          open={hoverPopover.open}
          anchorEl={hoverPopover.anchorEl}
          slotProps={{
            arrow: { placement: 'bottom-center' },
            paper: {
              onMouseEnter: hoverPopover.onOpen,
              onMouseLeave: hoverPopover.onClose,
              sx: { ...(hoverPopover.open && { pointerEvents: 'auto' }) },
            },
          }}
          sx={{ pointerEvents: 'none' }}
        >
          {renderPopoverContent()}
        </CustomPopover>
      )}
    </>
  );

  const renderCustomizedPopover = (placement: ArrowPlacement) => {
    const isOpen = customizedPopover.open && activePlacement === placement;

    const buttonProps: ButtonProps & IconButtonProps = {
      color: isOpen ? 'primary' : 'inherit',
      onClick: (event) => handleCustomizedOpen(event, placement),
      sx: {
        ...(isOpen && { bgcolor: varAlpha('currentColor', 0.08) }),
      },
    };

    return (
      <Fragment key={placement}>
        {useIconButton ? (
          <IconButton {...buttonProps} sx={{ bgcolor: 'action.hover', ...buttonProps.sx }}>
            <Iconify icon="eva:more-vertical-fill" />
          </IconButton>
        ) : (
          <Button {...buttonProps} variant="outlined">
            {placement}
          </Button>
        )}

        <CustomPopover
          open={isOpen}
          onClose={handleCustomizedClose}
          anchorEl={customizedPopover.anchorEl}
          slotProps={{ arrow: { placement } }}
        >
          {renderPopoverContent()}
        </CustomPopover>
      </Fragment>
    );
  };

  const DEMO_COMPONENTS = [
    {
      name: 'Click & hover',
      component: (
        <ComponentBox>
          {renderClickPopover()}
          {renderHoverPopover()}
        </ComponentBox>
      ),
    },
    {
      name: 'Custom arrow',
      action: (
        <FormControlLabel
          label="Icon button"
          labelPlacement="start"
          control={
            <Switch
              checked={useIconButton}
              onChange={handleToggleIconButton}
              slotProps={{ input: { id: 'icon-button-switch' } }}
            />
          }
        />
      ),
      component: (
        <Box sx={contentStyles.column()}>
          <ComponentBox title="top-*">
            {ARROW_PLACEMENTS.top.map((placement) => renderCustomizedPopover(placement))}
          </ComponentBox>

          <ComponentBox title="bottom-*">
            {ARROW_PLACEMENTS.bottom.map((placement) => renderCustomizedPopover(placement))}
          </ComponentBox>

          <ComponentBox title="left-*">
            {ARROW_PLACEMENTS.left.map((placement) => renderCustomizedPopover(placement))}
          </ComponentBox>

          <ComponentBox title="right-*">
            {ARROW_PLACEMENTS.right.map((placement) => renderCustomizedPopover(placement))}
          </ComponentBox>
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Popover',
        moreLinks: ['https://mui.com/material-ui/react-popover/'],
      }}
    />
  );
}
