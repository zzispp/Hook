import dayjs from 'dayjs';

import { fSub, fAdd } from 'src/utils/set-date';

import { _mock } from 'src/_mock';

import COLORS from '../colors.json';

// ----------------------------------------------------------------------

const primaryMain = COLORS.primary.main;
const secondaryMain = COLORS.secondary.main;
const infoMain = COLORS.info.main;
const infoDarker = COLORS.info.darker;
const successMain = COLORS.success.main;
const warningMain = COLORS.warning.main;
const errorMain = COLORS.error.main;
const errorDarker = COLORS.error.darker;

const base = (index: number) => ({
  id: _mock.id(index),
  title: _mock.eventNames(index),
  description: _mock.description(index),
});

export const _events = () => [
  {
    ...base(1),
    allDay: false,
    color: primaryMain,
    start: fSub({ days: 12, hours: 3, minutes: 30 }),
    end: fSub({ days: 12, hours: 0, minutes: 0 }),
  },
  {
    ...base(2),
    allDay: false,
    color: infoMain,
    start: fSub({ days: 6, hours: 3, minutes: 30 }),
    end: fSub({ days: 6, hours: 0, minutes: 0 }),
  },
  {
    ...base(3),
    allDay: true,
    color: errorMain,
    start: dayjs(fSub({ days: 3 }))
      .startOf('day')
      .format(),
    end: dayjs(fSub({ days: 3 }))
      .endOf('day')
      .format(),
  },
  {
    ...base(4),
    allDay: false,
    color: secondaryMain,
    start: fSub({ days: 0, hours: 2 }),
    end: fSub({ days: 0, hours: 0 }),
  },
  {
    ...base(5),
    allDay: false,
    color: infoDarker,
    start: fAdd({ days: 2, hours: 1, minutes: 15 }),
    end: fAdd({ days: 2, hours: 2, minutes: 30 }),
  },
  {
    ...base(6),
    allDay: false,
    color: warningMain,
    start: fAdd({ days: 2, hours: 3, minutes: 15 }),
    end: fAdd({ days: 2, hours: 4, minutes: 30 }),
  },
  {
    ...base(7),
    allDay: false,
    color: successMain,
    start: fAdd({ days: 2, hours: 5, minutes: 15 }),
    end: fAdd({ days: 2, hours: 6, minutes: 30 }),
  },
  {
    ...base(8),
    allDay: false,
    color: infoMain,
    start: fAdd({ days: 2, hours: 7, minutes: 15 }),
    end: fAdd({ days: 2, hours: 8, minutes: 30 }),
  },
  {
    ...base(9),
    allDay: false,
    color: errorDarker,
    start: fAdd({ days: 6, hours: 0, minutes: 0 }),
    end: fAdd({ days: 6, hours: 0, minutes: 30 }),
  },
];
