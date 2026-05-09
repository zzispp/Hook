'use client';

import type { MenuManagementData } from './menu-management-view';
import type { useMenuManagementActions } from './menu-management-actions';
import type { MenuDeleteState, MenuManagementForms } from './menu-management-state';

import { MenuItemFormDialog } from './menu-item-form-dialog';
import { MenuApiBindingDialog } from './menu-api-binding-dialog';
import { MenuSectionFormDialog } from './menu-section-form-dialog';
import { DeleteItemDialog, DeleteSectionDialog } from './menu-delete-dialogs';

type Props = {
  data: MenuManagementData;
  formState: MenuManagementForms;
  deleteState: MenuDeleteState;
  actionState: ReturnType<typeof useMenuManagementActions>;
};

export function MenuManagementDialogs({ data, formState, deleteState, actionState }: Props) {
  return (
    <>
      <MenuSectionFormDialog
        editing={formState.section.editing}
        form={formState.section.form}
        open={formState.section.open}
        submitting={actionState.submitting}
        onClose={formState.section.close}
        onFormChange={formState.section.setForm}
        onSubmit={actionState.submitSection}
      />
      <MenuItemFormDialog
        allItems={data.allItems.items}
        allSections={data.allSections.items}
        editing={formState.item.editing}
        form={formState.item.form}
        open={formState.item.open}
        submitting={actionState.submitting}
        onClose={formState.item.close}
        onFormChange={formState.item.setForm}
        onSubmit={actionState.submitItem}
      />
      <MenuApiBindingDialog
        apis={data.apis.items}
        loading={formState.apiBinding.loading}
        menu={formState.apiBinding.target}
        selectedApiIds={formState.apiBinding.selectedApiIds}
        submitting={actionState.submitting}
        onClose={formState.apiBinding.close}
        onSelectedApiIdsChange={formState.apiBinding.setSelectedApiIds}
        onSubmit={actionState.saveApiBindings}
      />
      <DeleteSectionDialog
        target={deleteState.sectionTarget}
        onClose={() => deleteState.setSectionTarget(null)}
        onConfirm={actionState.confirmDeleteSection}
      />
      <DeleteItemDialog
        target={deleteState.itemTarget}
        onClose={() => deleteState.setItemTarget(null)}
        onConfirm={actionState.confirmDeleteItem}
      />
    </>
  );
}
