import type { MapProps, MapStyleKey } from 'src/components/map';

import { lowerCase, upperFirst } from 'es-toolkit';
import { useMemo, useState, useCallback } from 'react';

import Radio from '@mui/material/Radio';
import RadioGroup from '@mui/material/RadioGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Map, MAP_STYLES, MapControls } from 'src/components/map';

import { ControlPanelRoot } from './styles';

// ----------------------------------------------------------------------

export function MapChangeStyle({ sx, ...other }: MapProps) {
  const [selectedStyle, setSelectedStyle] = useState<MapStyleKey>('light');

  const handleChangeStyle = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedStyle(event.target.value as MapStyleKey);
  }, []);

  const styleOptions = useMemo(() => Object.keys(MAP_STYLES) as MapStyleKey[], []);

  return (
    <Map
      mapStyle={MAP_STYLES[selectedStyle]}
      initialViewState={{ latitude: 37.785164, longitude: -100, zoom: 3.5, bearing: 0, pitch: 0 }}
      sx={sx}
      {...other}
    >
      <MapControls />
      <ControlPanel
        value={selectedStyle}
        onChange={handleChangeStyle}
        styleOptions={styleOptions}
      />
    </Map>
  );
}

// ----------------------------------------------------------------------

type ControlPanelProps = {
  value: MapStyleKey;
  onChange: (event: React.ChangeEvent<HTMLInputElement>) => void;
  styleOptions: MapStyleKey[];
};

function ControlPanel({ styleOptions, value, onChange }: ControlPanelProps) {
  return (
    <ControlPanelRoot>
      <RadioGroup value={value} onChange={onChange}>
        {styleOptions.map((item) => (
          <FormControlLabel
            key={item}
            value={item}
            label={upperFirst(lowerCase(item))}
            control={<Radio size="small" />}
            sx={{ color: 'common.white' }}
          />
        ))}
      </RadioGroup>
    </ControlPanelRoot>
  );
}
