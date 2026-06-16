import type { AdminT } from './shared';
import type { SystemUser } from 'src/types/rbac';
import type { ApiToken } from 'src/types/api-token';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { RoutingProfile, RoutingMetricWindow, RouteScoreExplanation } from 'src/types/routing';

import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import CardHeader from '@mui/material/CardHeader';

import { routingProfileName } from './routing-i18n';
import { RoutingRankingTable } from './routing-ranking-table';
import { RoutingProfileEditor } from './routing-profile-editor';
import { RoutingFilters } from './routing-observability-controls';
import { RoutingProfileSummary } from './routing-profile-summary';
import { RoutingProfileSettings } from './routing-profile-settings';

type ConfigurationPanelProps = {
  t: AdminT;
  users: SystemUser[];
  apiTokens: ApiToken[];
  models: GlobalModelResponse[];
  selectedGroup: BillingGroup | null;
  selectedModel: GlobalModelResponse | null;
  profiles: RoutingProfile[];
  settingsLoading: boolean;
  userId: string;
  apiTokenId: string;
  groupCode: string;
  modelName: string;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  requestInput: string;
  canSimulate: boolean;
  onUserChange: (value: string) => void;
  onApiTokenChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
  onRequestInputChange: (value: string) => void;
  onSimulate: VoidFunction;
  onDecisionLookup: VoidFunction;
  onSaved: VoidFunction;
};

type RankingPanelProps = {
  t: AdminT;
  selectedProfile: RoutingProfile | null;
  rankingRows: RouteScoreExplanation[];
  rankingsLoading: boolean;
  hasSubmittedQuery: boolean;
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
          <Stack spacing={2.5} sx={{ p: 2.5, pt: 1.5 }}>
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
            {props.hasSubmittedQuery ? (
              <RoutingRankingTable
                rows={props.rankingRows}
                loading={props.rankingsLoading}
                t={props.t}
                onOpen={props.onOpenRanking}
              />
            ) : (
              <Alert severity="info">{props.t('routing.emptyBeforeQuery')}</Alert>
            )}
          </Stack>
        </Card>
      </Grid>
    </Grid>
  );
}
