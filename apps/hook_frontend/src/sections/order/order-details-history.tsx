import type { IOrderHistory } from 'src/types/order';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Paper from '@mui/material/Paper';
import Timeline from '@mui/lab/Timeline';
import Button from '@mui/material/Button';
import TimelineDot from '@mui/lab/TimelineDot';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import TimelineContent from '@mui/lab/TimelineContent';
import TimelineSeparator from '@mui/lab/TimelineSeparator';
import TimelineConnector from '@mui/lab/TimelineConnector';
import TimelineItem, { timelineItemClasses } from '@mui/lab/TimelineItem';

import { fDateTime } from 'src/utils/format-time';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

type Props = {
  history?: IOrderHistory;
};

export function OrderDetailsHistory({ history }: Props) {
  const items = [
    { label: 'Order placed', value: fDateTime(history?.orderTime) },
    { label: 'Payment time', value: fDateTime(history?.orderTime) },
    { label: 'Delivery time for the carrier', value: fDateTime(history?.orderTime) },
    { label: 'Completion time', value: fDateTime(history?.orderTime) },
  ];

  const renderSummary = () => (
    <Paper
      variant="outlined"
      sx={{
        p: 2.5,
        gap: 2,
        minWidth: 260,
        flexShrink: 0,
        borderRadius: 2,
        display: 'flex',
        typography: 'body2',
        borderStyle: 'dashed',
        flexDirection: 'column',
      }}
    >
      {items.map((item) => (
        <Box key={item.label} sx={{ gap: 0.5, display: 'flex', flexDirection: 'column' }}>
          <Box component="span" sx={{ color: 'text.secondary' }}>
            {item.label}
          </Box>
          {item.value}
        </Box>
      ))}
    </Paper>
  );

  const renderTimeline = () => (
    <Timeline
      sx={{
        p: 0,
        [`& .${timelineItemClasses.root}:before`]: { p: 0, flex: 0 },
      }}
    >
      {history?.timeline.map((item, index) => {
        const firstTime = index === 0;
        const lastTime = index === history.timeline.length - 1;

        return (
          <TimelineItem key={item.title}>
            <TimelineSeparator>
              <TimelineDot color={firstTime ? 'primary' : 'grey'} />
              {lastTime ? null : <TimelineConnector />}
            </TimelineSeparator>

            <TimelineContent>
              <Typography variant="subtitle2">{item.title}</Typography>
              <Box component="span" sx={{ color: 'text.disabled', typography: 'caption', mt: 0.5 }}>
                {fDateTime(item.time)}
              </Box>
            </TimelineContent>
          </TimelineItem>
        );
      })}
    </Timeline>
  );

  return (
    <Card>
      <CardHeader title="History" />
      <Box
        sx={{
          p: 3,
          gap: 3,
          display: 'flex',
          alignItems: { md: 'flex-start' },
          flexDirection: { xs: 'column-reverse', md: 'row' },
        }}
      >
        <Box sx={{ flexGrow: 1 }}>
          {renderTimeline()}
          <Button
            size="small"
            color="inherit"
            endIcon={<Iconify icon="eva:arrow-ios-forward-fill" width={18} />}
          >
            Show more
          </Button>
        </Box>
        {renderSummary()}
      </Box>
    </Card>
  );
}
