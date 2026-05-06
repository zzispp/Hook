'use client';

import type { MapProps } from 'src/components/map';

import { MAP_STYLES } from 'src/components/map';

import { MapHeatmap } from './map-heatmap';
import { cities, countries } from './data';
import { MapClusters } from './map-clusters';
import { ComponentLayout } from '../../layout';
import { MapSideBySide } from './map-side-by-side';
import { MapInteraction } from './map-interaction';
import { MapChangeStyle } from './map-change-style';
import { MapMarkersPopups } from './map-markers-popups';
import { MapDraggableMarkers } from './map-draggable-markers';
import { MapGeoJSONAnimation } from './map-geojson-animation';
import { MapViewportAnimation } from './map-viewport-animation';
import { MapHighlightByFilter } from './map-highlight-by-filter';

// ----------------------------------------------------------------------

const baseSettings: MapProps = {
  minZoom: 1,
  sx: {
    height: 480,
    borderRadius: 1,
  },
};

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Change theme',
    component: <MapChangeStyle {...baseSettings} />,
  },
  {
    name: 'Markers & popups',
    component: <MapMarkersPopups {...baseSettings} data={countries} mapStyle={MAP_STYLES.light} />,
  },
  {
    name: 'Draggable markers',
    component: <MapDraggableMarkers {...baseSettings} mapStyle={MAP_STYLES.dark} />,
  },
  {
    name: 'Geojson animation',
    component: <MapGeoJSONAnimation {...baseSettings} mapStyle={MAP_STYLES.neutral} />,
  },
  {
    name: 'Clusters',
    component: <MapClusters {...baseSettings} />,
  },
  {
    name: 'Interaction',
    component: <MapInteraction {...baseSettings} />,
  },
  {
    name: 'Viewport animation',
    component: (
      <MapViewportAnimation
        {...baseSettings}
        data={cities.filter((city) => city.state === 'Texas')}
      />
    ),
  },
  {
    name: 'Highlight by filter',
    component: <MapHighlightByFilter {...baseSettings} />,
  },
  {
    name: 'Heatmap',
    component: <MapHeatmap {...baseSettings} />,
  },
  {
    name: 'Side by side',
    component: <MapSideBySide {...baseSettings} />,
  },
];

// ----------------------------------------------------------------------

export function MapView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Map',
        moreLinks: [
          'http://visgl.github.io/react-map-gl',
          'http://visgl.github.io/react-map-gl/examples',
        ],
      }}
    />
  );
}
