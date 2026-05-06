import type { MapProps } from 'src/components/map';

import Box from '@mui/material/Box';

import { Image } from 'src/components/image';
import { FlagIcon } from 'src/components/flag-icon';
import { Map, MapPopup, MapMarker, MapControls, useMapMarkerPopup } from 'src/components/map';

// ----------------------------------------------------------------------

type CountryProps = {
  name: string;
  code: string;
  capital: string;
  latlng: number[];
  photoUrl: string;
  timezones: string[];
};

type Props = MapProps & {
  data: CountryProps[];
};

export function MapMarkersPopups({ data, sx, ...other }: Props) {
  const { selectedItem, onOpenPopup, onClosePopup } = useMapMarkerPopup<CountryProps>();

  return (
    <Map initialViewState={{ zoom: 2 }} sx={sx} {...other}>
      <MapControls />

      {data.map((location, index) => (
        <MapMarker
          key={`marker-${index}`}
          latitude={location.latlng[0]}
          longitude={location.latlng[1]}
          onClick={(event) => onOpenPopup(event, location)}
        />
      ))}

      {selectedItem && (
        <MapPopup
          latitude={selectedItem.latlng[0]}
          longitude={selectedItem.latlng[1]}
          onClose={onClosePopup}
        >
          <Box sx={{ display: 'flex', flexDirection: 'column' }}>
            <Box
              sx={{
                mb: 1,
                gap: 0.75,
                display: 'flex',
                alignItems: 'center',
                typography: 'subtitle2',
              }}
            >
              <FlagIcon code={selectedItem.code} />
              {selectedItem.name}
            </Box>

            <Box component="span" sx={{ typography: 'caption' }}>
              Timezones: {selectedItem.timezones}
            </Box>

            <Box component="span" sx={{ typography: 'caption' }}>
              Lat: {selectedItem.latlng[0]}
            </Box>

            <Box component="span" sx={{ typography: 'caption' }}>
              Long: {selectedItem.latlng[1]}
            </Box>

            <Image
              alt={selectedItem.name}
              src={selectedItem.photoUrl}
              ratio="4/3"
              sx={{ mt: 1, borderRadius: 1 }}
            />
          </Box>
        </MapPopup>
      )}
    </Map>
  );
}
