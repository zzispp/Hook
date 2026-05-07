'use client';

import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/global-config';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  _analyticTasks,
  _analyticPosts,
  _analyticTraffic,
  _analyticOrderTimeline,
} from 'src/_mock';

import { AnalyticsNews } from '../analytics-news';
import { AnalyticsTasks } from '../analytics-tasks';
import { AnalyticsCurrentVisits } from '../analytics-current-visits';
import { AnalyticsOrderTimeline } from '../analytics-order-timeline';
import { AnalyticsWebsiteVisits } from '../analytics-website-visits';
import { AnalyticsWidgetSummary } from '../analytics-widget-summary';
import { AnalyticsTrafficBySite } from '../analytics-traffic-by-site';
import { AnalyticsCurrentSubject } from '../analytics-current-subject';
import { AnalyticsConversionRates } from '../analytics-conversion-rates';

// ----------------------------------------------------------------------

export function OverviewAnalyticsView() {
  const { t } = useTranslate('admin');
  const months = t('dashboard.months', { returnObjects: true }) as string[];
  const newsTitles = t('dashboard.newsItems.titles', { returnObjects: true }) as string[];
  const newsDescriptions = t('dashboard.newsItems.descriptions', { returnObjects: true }) as string[];
  const timelineItems = t('dashboard.timeline.items', { returnObjects: true }) as string[];
  const taskItems = t('dashboard.taskItems', { returnObjects: true }) as string[];
  const localizedPosts = _analyticPosts.map((post, index) => ({
    ...post,
    title: newsTitles[index] ?? post.title,
    description: newsDescriptions[index] ?? post.description,
  }));
  const localizedTimeline = _analyticOrderTimeline.map((item, index) => ({
    ...item,
    title: timelineItems[index] ?? item.title,
  }));
  const localizedTasks = _analyticTasks.map((task, index) => ({
    ...task,
    name: taskItems[index] ?? task.name,
  }));

  return (
    <DashboardContent maxWidth="xl">
      <Typography variant="h4" sx={{ mb: { xs: 3, md: 5 } }}>
        {t('dashboard.welcome')}
      </Typography>

      <Grid container spacing={3}>
        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <AnalyticsWidgetSummary
            title={t('dashboard.weeklySales')}
            percent={2.6}
            total={714000}
            icon={
              <img
                alt={t('dashboard.weeklySales')}
                src={`${CONFIG.assetsDir}/assets/icons/glass/ic-glass-bag.svg`}
              />
            }
            chart={{
              categories: months.slice(0, 8),
              series: [22, 8, 35, 50, 82, 84, 77, 12],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <AnalyticsWidgetSummary
            title={t('dashboard.newUsers')}
            percent={-0.1}
            total={1352831}
            color="secondary"
            icon={
              <img
                alt={t('dashboard.newUsers')}
                src={`${CONFIG.assetsDir}/assets/icons/glass/ic-glass-users.svg`}
              />
            }
            chart={{
              categories: months.slice(0, 8),
              series: [56, 47, 40, 62, 73, 30, 23, 54],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <AnalyticsWidgetSummary
            title={t('dashboard.purchaseOrders')}
            percent={2.8}
            total={1723315}
            color="warning"
            icon={
              <img
                alt={t('dashboard.purchaseOrders')}
                src={`${CONFIG.assetsDir}/assets/icons/glass/ic-glass-buy.svg`}
              />
            }
            chart={{
              categories: months.slice(0, 8),
              series: [40, 70, 50, 28, 70, 75, 7, 64],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, sm: 6, md: 3 }}>
          <AnalyticsWidgetSummary
            title={t('dashboard.messages')}
            percent={3.6}
            total={234}
            color="error"
            icon={
              <img
                alt={t('dashboard.messages')}
                src={`${CONFIG.assetsDir}/assets/icons/glass/ic-glass-message.svg`}
              />
            }
            chart={{
              categories: months.slice(0, 8),
              series: [56, 30, 23, 54, 47, 40, 62, 73],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 4 }}>
          <AnalyticsCurrentVisits
            title={t('dashboard.currentVisits')}
            chart={{
              series: [
                { label: t('dashboard.regions.america'), value: 3500 },
                { label: t('dashboard.regions.asia'), value: 2500 },
                { label: t('dashboard.regions.europe'), value: 1500 },
                { label: t('dashboard.regions.africa'), value: 500 },
              ],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 8 }}>
          <AnalyticsWebsiteVisits
            title={t('dashboard.websiteVisits')}
            subheader={t('dashboard.thanLastYear')}
            tooltipSuffix={t('dashboard.visits')}
            chart={{
              categories: months,
              series: [
                { name: t('dashboard.series.teamA'), data: [43, 33, 22, 37, 67, 68, 37, 24, 55] },
                { name: t('dashboard.series.teamB'), data: [51, 70, 47, 67, 40, 37, 24, 70, 24] },
              ],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 8 }}>
          <AnalyticsConversionRates
            title={t('dashboard.conversionRates')}
            subheader={t('dashboard.thanLastYear')}
            chart={{
              categories: [
                t('dashboard.countries.italy'),
                t('dashboard.countries.japan'),
                t('dashboard.countries.china'),
                t('dashboard.countries.canada'),
                t('dashboard.countries.france'),
              ],
              series: [
                { name: '2022', data: [44, 55, 41, 64, 22] },
                { name: '2023', data: [53, 32, 33, 52, 13] },
              ],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 4 }}>
          <AnalyticsCurrentSubject
            title={t('dashboard.currentSubject')}
            chart={{
              categories: [
                t('dashboard.subjects.english'),
                t('dashboard.subjects.history'),
                t('dashboard.subjects.physics'),
                t('dashboard.subjects.geography'),
                t('dashboard.subjects.chinese'),
                t('dashboard.subjects.math'),
              ],
              series: [
                { name: t('dashboard.series.series1'), data: [80, 50, 30, 40, 100, 20] },
                { name: t('dashboard.series.series2'), data: [20, 30, 40, 80, 20, 80] },
                { name: t('dashboard.series.series3'), data: [44, 76, 78, 13, 43, 10] },
              ],
            }}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 8 }}>
          <AnalyticsNews
            title={t('dashboard.news')}
            list={localizedPosts}
            viewAllText={t('dashboard.viewAll')}
          />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 4 }}>
          <AnalyticsOrderTimeline title={t('dashboard.orderTimeline')} list={localizedTimeline} />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 4 }}>
          <AnalyticsTrafficBySite title={t('dashboard.trafficBySite')} list={_analyticTraffic} />
        </Grid>

        <Grid size={{ xs: 12, md: 6, lg: 8 }}>
          <AnalyticsTasks
            title={t('dashboard.tasks')}
            list={localizedTasks}
            actionLabels={{
              delete: t('common.delete'),
              edit: t('common.edit'),
              markComplete: t('dashboard.markComplete'),
              share: t('dashboard.share'),
            }}
          />
        </Grid>
      </Grid>
    </DashboardContent>
  );
}
