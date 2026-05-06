import type { ControlPanelProps } from '../control-panel';

import { useState, useCallback } from 'react';
import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';

import { Toolbar } from './toolbar';
import { ContainerView } from './container';
import { ControlPanel } from '../control-panel';

// ----------------------------------------------------------------------

export function AnimateInview({ options }: Pick<ControlPanelProps, 'options'>) {
  const isTextObject = useBoolean();
  const isMultipleItems = useBoolean();

  const [count, setCount] = useState(0);
  const [selectedVariant, setSelectedVariant] = useState('slideInUp');

  const handleRefresh = useCallback(() => {
    setCount((prev) => prev + 1);
  }, []);

  const handleChangeVariant = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setCount((prev) => prev + 1);
    setSelectedVariant((event.target as HTMLInputElement).value);
  }, []);

  return (
    <Card sx={{ height: 640, display: 'flex' }}>
      <Box
        sx={{
          p: 2.5,
          gap: 2.5,
          display: 'flex',
          flex: '1 1 auto',
          flexDirection: 'column',
        }}
      >
        <Toolbar
          isText={isTextObject.value}
          isMultiple={isMultipleItems.value}
          onChangeText={isTextObject.onToggle}
          onChangeMultiple={isMultipleItems.onToggle}
          onRefresh={handleRefresh}
        />
        <ContainerView
          key={count}
          isText={isTextObject.value}
          isMultiple={isMultipleItems.value}
          selectedVariant={selectedVariant}
        />
      </Box>

      <ControlPanel
        options={options}
        selectedVariant={selectedVariant}
        onChangeVariant={handleChangeVariant}
      />
    </Card>
  );
}
