import type { MapRef } from 'react-map-gl/maplibre';
import type { MapProps } from 'src/components/map';

import { useRef, useState, useCallback } from 'react';

import Radio from '@mui/material/Radio';
import RadioGroup from '@mui/material/RadioGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Map, MapControls } from 'src/components/map';

import { ControlPanelRoot } from './styles';

// ----------------------------------------------------------------------

type Props = MapProps & {
  data: Location[];
};

export function MapViewportAnimation({ data, sx, ...other }: Props) {
  const mapRef = useRef<MapRef>(null);
  const [selectedCity, setSelectedCity] = useState(data[2].city ?? '');

  const handleChangeLocation = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>, location: Location) => {
      setSelectedCity(event.target.value);

      const mapEl = mapRef.current;
      if (!mapEl) return;

      const currentCenter = mapEl.getCenter();
      const sameLocation =
        Math.abs(currentCenter.lng - location.longitude) < 0.0001 &&
        Math.abs(currentCenter.lat - location.latitude) < 0.0001;

      if (!sameLocation) {
        mapEl.flyTo({ center: [location.longitude, location.latitude], duration: 2000 });
      }
    },
    []
  );

  return (
    <Map
      ref={mapRef}
      initialViewState={{ latitude: 37.7751, longitude: -122.4193, zoom: 11, bearing: 0, pitch: 0 }}
      sx={sx}
      {...other}
    >
      <MapControls />
      <ControlPanel data={data} selectedCity={selectedCity} onSelectCity={handleChangeLocation} />
    </Map>
  );
}

// ----------------------------------------------------------------------

type Location = {
  city: string;
  state: string;
  latitude: number;
  longitude: number;
};

type ControlPanelProps = {
  data: Location[];
  selectedCity: string;
  onSelectCity: (event: React.ChangeEvent<HTMLInputElement>, city: Location) => void;
};

function ControlPanel({ data, selectedCity, onSelectCity }: ControlPanelProps) {
  return (
    <ControlPanelRoot>
      {data.map((location) => (
        <RadioGroup
          key={location.city}
          value={selectedCity}
          onChange={(event) => onSelectCity(event, location)}
        >
          <FormControlLabel
            value={location.city}
            label={location.city}
            control={<Radio />}
            sx={{ color: 'common.white' }}
          />
        </RadioGroup>
      ))}
    </ControlPanelRoot>
  );
}
