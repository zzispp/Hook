'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderQuickImportActionState } from './provider-quick-import-action-state';

import { ProviderQuickImportAppendDialog } from './provider-quick-import-append-dialog';
import { ProviderQuickImportResolutionDialog } from './provider-quick-import-resolution-dialog';
import { ProviderQuickImportModelAssociationsDialog } from './provider-quick-import-model-associations-dialog';

type Props = {
  actions: ReturnType<typeof useProviderQuickImportActionState>;
  models: GlobalModelResponse[];
};

export function ProviderQuickImportActionDialogs({ actions, models }: Props) {
  return (
    <>
      <ProviderQuickImportAppendDialog
        open={!!actions.appendProvider}
        provider={actions.appendProvider}
        models={models}
        onClose={actions.closeAppend}
      />
      <ProviderQuickImportResolutionDialog
        open={!!actions.resolutionTarget}
        provider={actions.resolutionTarget?.provider ?? null}
        apiKey={actions.resolutionTarget?.apiKey ?? null}
        models={models}
        onClose={actions.closeResolution}
      />
      <ProviderQuickImportModelAssociationsDialog
        open={!!actions.modelAssociationsTarget}
        provider={actions.modelAssociationsTarget?.provider ?? null}
        apiKey={actions.modelAssociationsTarget?.apiKey ?? null}
        models={models}
        onClose={actions.closeModelAssociations}
      />
    </>
  );
}
