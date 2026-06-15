import type { AdminT } from './shared';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { RoutingProfile, RoutingProfileId, RoutingMetricWindow, RouteScoreExplanation } from 'src/types/routing';

import Tab from '@mui/material/Tab';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import CardHeader from '@mui/material/CardHeader';

import { routingProfileName } from './routing-i18n';
import { RoutingRankingTable } from './routing-ranking-table';
import { RoutingProfileEditor } from './routing-profile-editor';
import { RoutingFilters } from './routing-observability-controls';
import { RoutingProfileSummary } from './routing-profile-summary';
import { RoutingProfileSettings } from './routing-profile-settings';

type ConfigurationPanelProps = {
  t: AdminT;
  groups: BillingGroup[];
  models: GlobalModelResponse[];
  selectedGroup: BillingGroup | null;
  selectedModel: GlobalModelResponse | null;
  profiles: RoutingProfile[];
  settingsLoading: boolean;
  groupCode: string;
  modelName: string;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  requestInput: string;
  onGroupChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
  onRequestInputChange: (value: string) => void;
  onDecisionLookup: VoidFunction;
  onSaved: VoidFunction;
};

type RankingPanelProps = {
  t: AdminT;
  profiles: RoutingProfile[];
  profileId: RoutingProfileId;
  selectedProfile: RoutingProfile | null;
  rankingRows: RouteScoreExplanation[];
  rankingsLoading: boolean;
  onProfileChange: (value: RoutingProfileId) => void;
  onOpenRanking: (item: RouteScoreExplanation) => void;
  onSaved: VoidFunction;
};

export function RoutingConfigurationPanel(props: ConfigurationPanelProps) {
  return (
    <Grid container spacing={3}>
      <Grid size={{ xs: 12, md: 6 }}>
        <Card sx={{ height: 1 }}>
          <CardHeader
            title={props.t('routing.runtime.title')}
            subheader={props.t('routing.runtime.helper')}
            titleTypographyProps={{ variant: 'subtitle1' }}
            subheaderTypographyProps={{ variant: 'caption', color: 'text.secondary' }}
          />
          <Stack sx={{ p: 2.5, pt: 1.5 }}>
            <RoutingProfileSettings
              group={props.selectedGroup}
              model={props.selectedModel}
              profiles={props.profiles}
              loading={props.settingsLoading}
              onSaved={props.onSaved}
            />
          </Stack>
        </Card>
      </Grid>

      <Grid size={{ xs: 12, md: 6 }}>
        <Card sx={{ height: 1 }}>
          <CardHeader
            title={props.t('routing.cards.simulationContextTitle')}
            subheader={props.t('routing.cards.simulationContextHelper')}
            titleTypographyProps={{ variant: 'subtitle1' }}
            subheaderTypographyProps={{ variant: 'caption', color: 'text.secondary' }}
          />
          <Stack sx={{ p: 2.5, pt: 1.5 }}>
            <RoutingFilters {...props} />
          </Stack>
        </Card>
      </Grid>
    </Grid>
  );
}

export function RoutingRankingPanel(props: RankingPanelProps) {
  return (
    <Grid container spacing={3}>
      <Grid size={{ xs: 12, lg: 4 }}>
        <Card sx={{ height: 1 }}>
          <CardHeader
            title={props.t('routing.profile.editorTitle')}
            subheader={props.t('routing.cards.profileTuningHelper')}
            titleTypographyProps={{ variant: 'subtitle1' }}
            subheaderTypographyProps={{ variant: 'caption', color: 'text.secondary' }}
          />
          <ProfileTabs {...props} />
          <Stack spacing={2.5} sx={{ p: 2.5, pt: 0 }}>
            <RoutingProfileSummary profile={props.selectedProfile} t={props.t} />
            <RoutingProfileEditor profile={props.selectedProfile} onSaved={props.onSaved} />
          </Stack>
        </Card>
      </Grid>

      <Grid size={{ xs: 12, lg: 8 }}>
        <Card sx={{ height: 1 }}>
          <CardHeader
            title={props.t('routing.cards.rankingsTitle', {
              profile: props.selectedProfile ? routingProfileName(props.selectedProfile, props.t) : '',
            })}
            subheader={props.t('routing.cards.rankingsHelper')}
            titleTypographyProps={{ variant: 'subtitle1' }}
            subheaderTypographyProps={{ variant: 'caption', color: 'text.secondary' }}
          />
          <Stack sx={{ p: 2.5 }}>
            <RoutingRankingTable
              rows={props.rankingRows}
              loading={props.rankingsLoading}
              t={props.t}
              onOpen={props.onOpenRanking}
            />
          </Stack>
        </Card>
      </Grid>
    </Grid>
  );
}

function ProfileTabs(props: RankingPanelProps) {
  return (
    <Stack sx={{ px: 2.5, pt: 1 }}>
      <Tabs
        value={props.profileId}
        variant="scrollable"
        onChange={(_, value) => props.onProfileChange(value as RoutingProfileId)}
        sx={{
          borderBottom: 1,
          borderColor: 'divider',
          mb: 2,
          '& .MuiTab-root': { py: 1, minHeight: 38 },
        }}
      >
        {props.profiles.map((profile) => (
          <Tab key={profile.id} value={profile.id} label={routingProfileName(profile, props.t)} />
        ))}
      </Tabs>
    </Stack>
  );
}
