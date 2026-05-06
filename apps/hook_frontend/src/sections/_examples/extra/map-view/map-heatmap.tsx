import type { LayerProps } from 'react-map-gl/maplibre';
import type { Point, FeatureCollection } from 'geojson';
import type { MapProps } from 'src/components/map';

import { Layer, Source } from 'react-map-gl/maplibre';
import { useMemo, useState, useEffect, useCallback } from 'react';

import Slider from '@mui/material/Slider';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { fDate } from 'src/utils/format-time';

import { Map } from 'src/components/map';

import { ControlPanelRoot } from './styles';

// ----------------------------------------------------------------------

type EarthquakeFeatureCollection = FeatureCollection<Point, { mag: number; time: number }>;

export function MapHeatmap({ sx, ...other }: MapProps) {
  const [allDays, setAllDays] = useState(true);
  const [selectedTime, setSelectedTime] = useState(0);
  const [timeRange, setTimeRange] = useState<[number, number]>([0, 0]);
  const [earthquakes, setEarthquakes] = useState<EarthquakeFeatureCollection | null>(null);

  useEffect(() => {
    const fetchEarthquakes = async () => {
      try {
        const res = await fetch(
          'https://maplibre.org/maplibre-gl-js/docs/assets/earthquakes.geojson'
        );
        const json = await res.json();
        const features = json.features;
        if (!features?.length) return;

        const endTime = features[0].properties.time;
        const startTime = features[features.length - 1].properties.time;

        setEarthquakes(json);
        setSelectedTime(endTime);
        setTimeRange([startTime, endTime]);
      } catch (error) {
        console.error('Could not load data', error);
      }
    };

    fetchEarthquakes();
  }, []);

  const filteredData = useMemo<EarthquakeFeatureCollection | null>(() => {
    if (!earthquakes) return null;

    return allDays ? earthquakes : filterFeaturesByDay(earthquakes, selectedTime);
  }, [earthquakes, allDays, selectedTime]);

  return (
    <Map initialViewState={{ latitude: 40, longitude: -100, zoom: 3 }} sx={sx} {...other}>
      {filteredData && (
        <Source type="geojson" data={filteredData}>
          <Layer {...heatmapLayer()} />
        </Source>
      )}

      <ControlPanel
        allDays={allDays}
        startTime={timeRange[0]}
        endTime={timeRange[1]}
        selectedTime={selectedTime}
        onChangeTime={setSelectedTime}
        onChangeAllDays={setAllDays}
      />
    </Map>
  );
}

// ----------------------------------------------------------------------

function filterFeaturesByDay(
  featureCollection: EarthquakeFeatureCollection,
  time: number
): EarthquakeFeatureCollection {
  const date = new Date(time);
  const year = date.getFullYear();
  const month = date.getMonth();
  const day = date.getDate();

  const features = featureCollection.features.filter((feature) => {
    const featureDate = new Date(feature.properties?.time);

    return (
      featureDate.getFullYear() === year &&
      featureDate.getMonth() === month &&
      featureDate.getDate() === day
    );
  });

  return {
    type: 'FeatureCollection',
    features,
  };
}

// ----------------------------------------------------------------------

const heatmapLayer = (maxZoom: number = 9): LayerProps => ({
  id: 'heatmap',
  maxzoom: maxZoom,
  type: 'heatmap',
  paint: {
    'heatmap-weight': ['interpolate', ['linear'], ['get', 'mag'], 0, 0, 6, 1],
    'heatmap-intensity': ['interpolate', ['linear'], ['zoom'], 0, 1, maxZoom, 3],
    'heatmap-color': [
      'interpolate',
      ['linear'],
      ['heatmap-density'],
      0,
      'rgba(33,102,172,0)',
      0.2,
      'rgb(103,169,207)',
      0.4,
      'rgb(209,229,240)',
      0.6,
      'rgb(253,219,199)',
      0.8,
      'rgb(239,138,98)',
      0.9,
      'rgb(255,201,101)',
    ],
    'heatmap-radius': ['interpolate', ['linear'], ['zoom'], 0, 2, maxZoom, 20],
    'heatmap-opacity': ['interpolate', ['linear'], ['zoom'], 7, 1, 9, 0],
  },
});

// ----------------------------------------------------------------------

type ControlPanelProps = {
  allDays: boolean;
  startTime: number;
  endTime: number;
  selectedTime: number;
  onChangeTime: (value: number) => void;
  onChangeAllDays: (value: boolean) => void;
};

function ControlPanel({
  endTime,
  allDays,
  startTime,
  selectedTime,
  onChangeTime,
  onChangeAllDays,
}: ControlPanelProps) {
  const DAY_MS = 24 * 60 * 60 * 1000;

  const totalDays = Math.round((endTime - startTime) / DAY_MS);
  const selectedDay = Math.round((selectedTime - startTime) / DAY_MS);

  const handleChangeDay = useCallback(
    (value: number) => {
      onChangeTime(startTime + value * DAY_MS);
    },
    [DAY_MS, onChangeTime, startTime]
  );

  return (
    <ControlPanelRoot>
      <FormControlLabel
        label="All days"
        labelPlacement="start"
        control={
          <Switch
            size="small"
            checked={allDays}
            onChange={(event) => onChangeAllDays(event.target.checked)}
            slotProps={{ input: { id: 'all-days-switch' } }}
          />
        }
        sx={{
          mb: 2,
          mx: 0,
          width: 1,
          color: 'common.white',
          justifyContent: 'space-between',
        }}
      />

      <Typography variant="body2" sx={{ mb: 1, color: allDays ? 'text.disabled' : 'common.white' }}>
        Each day: {fDate(selectedTime)}
      </Typography>

      <Slider
        min={1}
        step={1}
        max={totalDays}
        disabled={allDays}
        value={selectedDay}
        onChange={(event, newValue) => {
          if (typeof newValue === 'number') handleChangeDay(newValue);
        }}
        sx={{ width: 180 }}
      />
    </ControlPanelRoot>
  );
}
