'use client';

import type { Theme, SxProps } from '@mui/material/styles';
import type { ICalendarEvent, ICalendarFilters } from 'src/types/calendar';

import { startTransition } from 'react';
import Calendar from '@fullcalendar/react';
import listPlugin from '@fullcalendar/list';
import dayGridPlugin from '@fullcalendar/daygrid';
import timeGridPlugin from '@fullcalendar/timegrid';
import interactionPlugin from '@fullcalendar/interaction';
import { useBoolean, useSetState } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';

import { fIsAfter, fIsBetween } from 'src/utils/format-time';

import { DashboardContent } from 'src/layouts/dashboard';
import { CALENDAR_COLOR_OPTIONS } from 'src/_mock/_calendar';
import { updateEvent, useGetEvents } from 'src/actions/calendar';

import { Iconify } from 'src/components/iconify';

import { CalendarRoot } from '../styles';
import { useEvent } from '../hooks/use-event';
import { CalendarForm } from '../calendar-form';
import { useCalendar } from '../hooks/use-calendar';
import { CalendarToolbar } from '../calendar-toolbar';
import { CalendarFilters } from '../calendar-filters';
import { CalendarFiltersResult } from '../calendar-filters-result';

// ----------------------------------------------------------------------

export function CalendarView() {
  const theme = useTheme();

  const openFilters = useBoolean();

  const { events, eventsLoading } = useGetEvents();

  const filters = useSetState<ICalendarFilters>({ colors: [], startDate: null, endDate: null });
  const { state: currentFilters } = filters;

  const dateError = fIsAfter(currentFilters.startDate, currentFilters.endDate);

  const {
    calendarRef,
    /********/
    view,
    title,
    /********/
    onDropEvent,
    onChangeView,
    onSelectRange,
    onClickEvent,
    onResizeEvent,
    onDateNavigation,
    /********/
    openForm,
    onOpenForm,
    onCloseForm,
    /********/
    selectedRange,
    selectedEventId,
    /********/
    onClickEventInFilters,
  } = useCalendar();

  const currentEvent = useEvent(events, selectedEventId, selectedRange, openForm);

  const canReset =
    currentFilters.colors.length > 0 || (!!currentFilters.startDate && !!currentFilters.endDate);

  const dataFiltered = applyFilter({
    inputData: events,
    filters: currentFilters,
    dateError,
  });

  const flexStyles: SxProps<Theme> = {
    flex: '1 1 auto',
    display: 'flex',
    flexDirection: 'column',
  };

  const renderCreateFormDialog = () => (
    <Dialog
      fullWidth
      maxWidth="xs"
      open={openForm}
      onClose={onCloseForm}
      transitionDuration={{
        enter: theme.transitions.duration.shortest,
        exit: theme.transitions.duration.shortest - 80,
      }}
      slotProps={{
        paper: {
          sx: {
            display: 'flex',
            overflow: 'hidden',
            flexDirection: 'column',
            '& form': { ...flexStyles, minHeight: 0 },
          },
        },
      }}
    >
      <DialogTitle sx={{ minHeight: 76 }}>
        {openForm && <> {currentEvent?.id ? 'Edit' : 'Add'} event</>}
      </DialogTitle>

      <CalendarForm
        currentEvent={currentEvent}
        colorOptions={CALENDAR_COLOR_OPTIONS}
        onClose={onCloseForm}
      />
    </Dialog>
  );

  const renderFiltersDrawer = () => (
    <CalendarFilters
      events={events}
      filters={filters}
      canReset={canReset}
      dateError={dateError}
      open={openFilters.value}
      onClose={openFilters.onFalse}
      onClickEvent={onClickEventInFilters}
      colorOptions={CALENDAR_COLOR_OPTIONS}
    />
  );

  const renderResults = () => (
    <CalendarFiltersResult
      filters={filters}
      totalResults={dataFiltered.length}
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );

  return (
    <>
      <DashboardContent maxWidth="xl" sx={{ ...flexStyles }}>
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            mb: { xs: 3, md: 5 },
          }}
        >
          <Typography variant="h4">Calendar</Typography>
          <Button
            variant="contained"
            startIcon={<Iconify icon="mingcute:add-line" />}
            onClick={onOpenForm}
          >
            Add event
          </Button>
        </Box>

        {canReset && renderResults()}

        <Card sx={{ ...flexStyles, minHeight: '50vh' }}>
          <CalendarRoot sx={{ ...flexStyles }}>
            <CalendarToolbar
              view={view}
              title={title}
              canReset={canReset}
              loading={eventsLoading}
              onChangeView={onChangeView}
              onDateNavigation={onDateNavigation}
              onOpenFilters={openFilters.onTrue}
              viewOptions={[
                { value: 'dayGridMonth', label: 'Month', icon: 'mingcute:calendar-month-line' },
                { value: 'timeGridWeek', label: 'Week', icon: 'mingcute:calendar-week-line' },
                { value: 'timeGridDay', label: 'Day', icon: 'mingcute:calendar-day-line' },
                { value: 'listWeek', label: 'Agenda', icon: 'custom:calendar-agenda-outline' },
              ]}
            />

            <Calendar
              weekends
              editable
              droppable
              selectable
              allDayMaintainDuration
              eventResizableFromStart
              firstDay={1}
              aspectRatio={3}
              dayMaxEvents={3}
              eventMaxStack={2}
              rerenderDelay={10}
              headerToolbar={false}
              eventDisplay="block"
              ref={calendarRef}
              initialView={view}
              events={dataFiltered}
              select={onSelectRange}
              eventClick={onClickEvent}
              businessHours={{
                daysOfWeek: [1, 2, 3, 4, 5], // Mon-Fri
              }}
              eventDrop={(arg) => {
                startTransition(() => {
                  onDropEvent(arg, updateEvent);
                });
              }}
              eventResize={(arg) => {
                startTransition(() => {
                  onResizeEvent(arg, updateEvent);
                });
              }}
              plugins={[dayGridPlugin, timeGridPlugin, listPlugin, interactionPlugin]}
            />
          </CalendarRoot>
        </Card>
      </DashboardContent>

      {renderCreateFormDialog()}
      {renderFiltersDrawer()}
    </>
  );
}

// ----------------------------------------------------------------------

type ApplyFilterProps = {
  dateError: boolean;
  filters: ICalendarFilters;
  inputData: ICalendarEvent[];
};

function applyFilter({ inputData, filters, dateError }: ApplyFilterProps) {
  const { colors, startDate, endDate } = filters;

  const stabilizedThis = inputData.map((el, index) => [el, index] as const);

  inputData = stabilizedThis.map((el) => el[0]);

  if (colors.length) {
    inputData = inputData.filter((event) => colors.includes(event.color as string));
  }

  if (!dateError) {
    if (startDate && endDate) {
      inputData = inputData.filter((event) => fIsBetween(event.start, startDate, endDate));
    }
  }

  return inputData;
}
