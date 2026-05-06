import type { LngLat, MarkerDragEvent } from 'react-map-gl/maplibre';
import type { MapProps, MapMarkerProps } from 'src/components/map';

import { useState, useCallback } from 'react';

import Typography from '@mui/material/Typography';

import { Map, MapMarker, MapControls } from 'src/components/map';

import { ControlPanelRoot } from './styles';

// ----------------------------------------------------------------------

const EVENT_NAMES = ['onDragStart', 'onDrag', 'onDragEnd'] as const;

type DragEvents = Record<(typeof EVENT_NAMES)[number], LngLat | undefined>;

export function MapDraggableMarkers({ sx, ...other }: MapProps) {
  const [marker, setMarker] = useState<Pick<MapMarkerProps, 'longitude' | 'latitude'>>({
    latitude: 40,
    longitude: -100,
  });

  const [events, setEvents] = useState<DragEvents>({
    onDragStart: undefined,
    onDrag: undefined,
    onDragEnd: undefined,
  });

  const updateEvent = useCallback((type: keyof DragEvents, lngLat: LngLat) => {
    setEvents((prev) => ({ ...prev, [type]: lngLat }));
  }, []);

  const handleMarkerDragStart = useCallback(
    (event: MarkerDragEvent) => updateEvent('onDragStart', event.lngLat),
    [updateEvent]
  );

  const handleMarkerDrag = useCallback(
    (event: MarkerDragEvent) => {
      updateEvent('onDrag', event.lngLat);
      setMarker({ longitude: event.lngLat.lng, latitude: event.lngLat.lat });
    },
    [updateEvent]
  );

  const handleMarkerDragEnd = useCallback(
    (event: MarkerDragEvent) => updateEvent('onDragEnd', event.lngLat),
    [updateEvent]
  );

  return (
    <Map initialViewState={{ latitude: 40, longitude: -100, zoom: 3.5 }} sx={sx} {...other}>
      <MapControls />

      <MapMarker
        {...marker}
        draggable
        anchor="bottom"
        onDragStart={handleMarkerDragStart}
        onDrag={handleMarkerDrag}
        onDragEnd={handleMarkerDragEnd}
      />

      <ControlPanel events={events} />
    </Map>
  );
}

// ----------------------------------------------------------------------

type ControlPanelProps = {
  events: DragEvents;
};

function ControlPanel({ events }: ControlPanelProps) {
  return (
    <ControlPanelRoot sx={{ gap: 0.5, display: 'flex', flexDirection: 'column' }}>
      {EVENT_NAMES.map((event) => {
        const lngLat = events[event];

        return (
          <div key={event}>
            <Typography variant="subtitle2" sx={{ color: 'common.white' }}>
              {event}:
            </Typography>

            <Typography
              component={lngLat ? 'span' : 'em'}
              variant={lngLat ? 'subtitle2' : 'body2'}
              sx={{ color: lngLat ? 'primary.main' : 'error.main' }}
            >
              {lngLat ? `${lngLat.lng.toFixed(5)}, ${lngLat.lat.toFixed(5)}` : 'null'}
            </Typography>
          </div>
        );
      })}
    </ControlPanelRoot>
  );
}
