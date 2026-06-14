'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderQuickImportActionState } from './provider-quick-import-action-state';

import { ProviderQuickImportBindDialog } from './provider-quick-import-bind-dialog';
import { ProviderQuickImportAppendDialog } from './provider-quick-import-append-dialog';
import { ProviderQuickImportResolutionDialog } from './provider-quick-import-resolution-dialog';
import { ProviderQuickImportModelAssociationsDialog } from './provider-quick-import-model-associations-dialog';

type Props = {
  actions: ReturnType<typeof useProviderQuickImportActionState>;
  models: GlobalModelResponse[];
  onBound: () => void;
};

export function ProviderQuickImportActionDialogs({ actions, models, onBound }: Props) {
  return (
    <>
      <ProviderQuickImportAppendDialog
        open={!!actions.appendProvider}
        provider={actions.appendProvider}
        models={models}
        onClose={actions.closeAppend}
      />
      <ProviderQuickImportBindDialog
        open={!!actions.bindProvider}
        provider={actions.bindProvider}
        models={models}
        onClose={actions.closeBind}
        onBound={onBound}
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
