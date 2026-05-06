import type { Theme } from '@mui/material/styles';
import type { LayerProps } from 'react-map-gl/maplibre';
import type { MapProps } from 'src/components/map';

import { useRef, useState, useEffect } from 'react';
import { Layer, Source } from 'react-map-gl/maplibre';

import { useTheme } from '@mui/material/styles';

import { Map, MapControls } from 'src/components/map';

// ----------------------------------------------------------------------

type PointData = {
  type: 'Point';
  coordinates: [number, number];
};

export function MapGeoJSONAnimation({ sx, ...other }: MapProps) {
  const theme = useTheme();

  const animationRef = useRef<number>(null);
  const [pointData, setPointData] = useState<PointData | null>(null);

  useEffect(() => {
    const animate = () => {
      const point = getPointOnCircle({
        center: [-100, 0],
        angle: Date.now() / 1000,
        radius: 20,
      });
      setPointData(point);
      animationRef.current = window.requestAnimationFrame(animate);
    };
    animationRef.current = window.requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) window.cancelAnimationFrame(animationRef.current);
    };
  }, []);

  return (
    <Map initialViewState={{ latitude: 0, longitude: -100, zoom: 3 }} sx={sx} {...other}>
      <MapControls />

      {pointData && (
        <Source type="geojson" data={pointData}>
          <Layer {...getPointLayer(theme)} />
        </Source>
      )}
    </Map>
  );
}

// ----------------------------------------------------------------------

const getPointLayer = (theme: Theme): LayerProps => ({
  id: 'point',
  type: 'circle',
  paint: {
    'circle-radius': 10,
    'circle-color': theme.palette.error.main,
  },
});

function getPointOnCircle({
  center,
  angle,
  radius,
}: {
  center: [number, number];
  angle: number;
  radius: number;
}): PointData {
  return {
    type: 'Point',
    coordinates: [center[0] + Math.cos(angle) * radius, center[1] + Math.sin(angle) * radius],
  };
}
