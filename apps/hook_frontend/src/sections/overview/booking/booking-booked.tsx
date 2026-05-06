import type { CardProps } from '@mui/material/Card';
import type { LinearProgressProps } from '@mui/material/LinearProgress';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import LinearProgress, { linearProgressClasses } from '@mui/material/LinearProgress';

import { fShortenNumber } from 'src/utils/format-number';

// ----------------------------------------------------------------------

type Props = CardProps & {
  title?: string;
  subheader?: string;
  data: {
    value: number;
    status: string;
    quantity: number;
  }[];
};

export function BookingBooked({ title, subheader, data, sx, ...other }: Props) {
  return (
    <Card sx={sx} {...other}>
      <CardHeader title={title} subheader={subheader} />

      <Box component="ul" sx={{ p: 3, gap: 3, display: 'flex', flexDirection: 'column' }}>
        {data.map((progress) => {
          const color: LinearProgressProps['color'] =
            (progress.status === 'Pending' && 'warning') ||
            (progress.status === 'Canceled' && 'error') ||
            'success';

          return (
            <li key={progress.status}>
              <Box sx={{ mb: 1, display: 'flex', alignItems: 'center' }}>
                <Box component="span" sx={{ typography: 'overline', flexGrow: 1 }}>
                  {progress.status}
                </Box>
                <Box component="span" sx={{ typography: 'subtitle1' }}>
                  {fShortenNumber(progress.quantity)}
                </Box>
              </Box>

              <LinearProgress
                color={color}
                variant="determinate"
                value={progress.value}
                sx={[
                  (theme) => ({
                    height: 8,
                    bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
                    [`& .${linearProgressClasses.bar}`]: {
                      background: `linear-gradient(135deg, ${theme.vars.palette[color].light}, ${theme.vars.palette[color].main})`,
                    },
                  }),
                ]}
              />
            </li>
          );
        })}
      </Box>
    </Card>
  );
}
