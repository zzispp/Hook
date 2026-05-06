import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';

// ----------------------------------------------------------------------

dayjs.extend(duration);

export function setDate(now: Date, options: { days?: number; hours?: number; minutes?: number }) {
  const month = now.getMonth() + 1;
  const year = now.getFullYear();
  const today = now.getDate();

  const { days, hours = 0, minutes = 0 } = options;

  return new Date(`${year}-${month}-${days ?? today} ${hours}:${minutes}`).toJSON();
}

export const subHours = (
  value: number,
  option: 'years' | 'months' | 'days' | 'hours' | 'minutes' | 'seconds' | 'milliseconds'
) => dayjs().subtract(value, option).format();

// years,
// months,
// days,
// hours,
// minutes,
// seconds,
// milliseconds,

export type DurationProps = {
  years?: number;
  months?: number;
  days?: number;
  hours?: number;
  minutes?: number;
  seconds?: number;
  milliseconds?: number;
};

export function fSub({
  years = 0,
  months = 0,
  days = 0,
  hours = 0,
  minutes = 0,
  seconds = 0,
  milliseconds = 0,
}: DurationProps) {
  const result = dayjs()
    .subtract(
      dayjs.duration({
        years,
        months,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
      })
    )
    .format();

  return result;
}

export function fAdd({
  years = 0,
  months = 0,
  days = 0,
  hours = 0,
  minutes = 0,
  seconds = 0,
  milliseconds = 0,
}: DurationProps) {
  const result = dayjs()
    .add(
      dayjs.duration({
        years,
        months,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
      })
    )
    .format();

  return result;
}
