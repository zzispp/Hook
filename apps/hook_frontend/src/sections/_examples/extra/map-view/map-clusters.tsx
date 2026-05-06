import type { MapRef, LayerProps, MapMouseEvent } from 'react-map-gl/maplibre';
import type { LngLatLike, GeoJSONSource, MapGeoJSONFeature } from 'maplibre-gl';
import type { MapProps } from 'src/components/map';

import { useRef, useCallback } from 'react';
import { Layer, Source } from 'react-map-gl/maplibre';

import { Map } from 'src/components/map';

// ----------------------------------------------------------------------

export function MapClusters({ sx, ...other }: MapProps) {
  const mapRef = useRef<MapRef>(null);

  const getClusterCoordinates = (feature: MapGeoJSONFeature): LngLatLike | undefined => {
    if (feature.geometry.type === 'Point' && Array.isArray(feature.geometry.coordinates)) {
      return feature.geometry.coordinates as LngLatLike;
    }
    return undefined;
  };

  const handleClickCluster = useCallback(async (event: MapMouseEvent) => {
    const mapEl = mapRef.current;
    const feature = event.features?.[0];
    if (!mapEl || !feature) return;

    const clusterId = feature.properties.cluster_id;
    if (!clusterId) return;

    const geojsonSource = mapEl.getSource('earthquakes') as GeoJSONSource;
    if (!geojsonSource.getClusterExpansionZoom) return;

    const zoom = await geojsonSource.getClusterExpansionZoom(clusterId);
    const center = getClusterCoordinates(feature);
    if (!center) return;

    mapEl.easeTo({
      center,
      zoom: zoom ?? 3,
      duration: 500,
    });
  }, []);

  return (
    <Map
      ref={mapRef}
      onClick={handleClickCluster}
      interactiveLayerIds={[clusterLayer.id ?? '']}
      initialViewState={{ latitude: 40.67, longitude: -103.59, zoom: 3 }}
      sx={sx}
      {...other}
    >
      <Source
        id="earthquakes"
        type="geojson"
        data="https://maplibre.org/maplibre-gl-js/docs/assets/earthquakes.geojson"
        cluster
        clusterMaxZoom={14}
        clusterRadius={50}
      >
        <Layer {...clusterLayer} />
        <Layer {...clusterCountLayer} />
        <Layer {...unclusteredPointLayer} />
      </Source>
    </Map>
  );
}

// ----------------------------------------------------------------------

const clusterLayer: LayerProps = {
  id: 'clusters',
  type: 'circle',
  source: 'earthquakes',
  filter: ['has', 'point_count'],
  paint: {
    'circle-color': ['step', ['get', 'point_count'], '#51bbd6', 100, '#f1f075', 750, '#f28cb1'],
    'circle-radius': ['step', ['get', 'point_count'], 20, 100, 30, 750, 40],
  },
};

const clusterCountLayer: LayerProps = {
  id: 'cluster-count',
  type: 'symbol',
  source: 'earthquakes',
  filter: ['has', 'point_count'],
  layout: {
    'text-field': '{point_count_abbreviated}',
    'text-size': 12,
  },
};

const unclusteredPointLayer: LayerProps = {
  id: 'unclustered-point',
  type: 'circle',
  source: 'earthquakes',
  filter: ['!', ['has', 'point_count']],
  paint: {
    'circle-color': '#11b4da',
    'circle-radius': 4,
    'circle-stroke-width': 1,
    'circle-stroke-color': '#fff',
  },
};
