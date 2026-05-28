import type { TFunction } from 'i18next';
import type { DashboardPreset } from 'src/types/dashboard';

const PERIOD_LABEL_KEYS: Record<DashboardPreset, string> = {
  today: 'dashboard.stats.period.labels.today',
  '7d': 'dashboard.stats.period.labels.sevenDays',
  '30d': 'dashboard.stats.period.labels.thirtyDays',
  '90d': 'dashboard.stats.period.labels.ninetyDays',
};

export function dashboardPeriodLabel(t: TFunction<'admin'>, preset: DashboardPreset) {
  return t(PERIOD_LABEL_KEYS[preset]);
}
