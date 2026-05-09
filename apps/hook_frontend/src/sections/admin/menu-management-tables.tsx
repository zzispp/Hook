'use client';

import type { MenuTab, MenuManagementData } from './menu-management-view';
import type { MenuDeleteState, MenuManagementForms } from './menu-management-state';

import { MenuItemsTable } from './menu-items-table';
import { MenuSectionsTable } from './menu-sections-table';

type Props = {
  tab: MenuTab;
  data: MenuManagementData;
  formState: MenuManagementForms;
  deleteState: MenuDeleteState;
};

export function MenuManagementTables({ tab, data, formState, deleteState }: Props) {
  if (tab === 'sections') {
    return (
      <MenuSectionsTable
        loading={data.sections.isLoading}
        rows={data.sections.items}
        table={data.sectionTable}
        total={data.sections.total}
        onDelete={deleteState.setSectionTarget}
        onEdit={formState.section.openEdit}
      />
    );
  }

  return (
    <MenuItemsTable
      loading={data.items.isLoading}
      rows={data.items.items}
      sectionNameById={data.sectionNameById}
      table={data.itemTable}
      total={data.items.total}
      onBindApis={formState.apiBinding.open}
      onDelete={deleteState.setItemTarget}
      onEdit={formState.item.openEdit}
    />
  );
}
