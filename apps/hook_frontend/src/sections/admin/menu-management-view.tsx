'use client';

import { useMemo, useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useApis, useMenuItems, useMenuSections } from 'src/actions/rbac';

import { useTable } from 'src/components/table';

import { MenuManagementTables } from './menu-management-tables';
import { MenuManagementDialogs } from './menu-management-dialogs';
import { useMenuManagementActions } from './menu-management-actions';
import { AddButton, RefreshButton, AdminBreadcrumbs } from './shared';
import { useMenuDeleteState, useMenuManagementForms } from './menu-management-state';
import { toEnabledFilters, AdminFiltersToolbar, DEFAULT_ADMIN_FILTERS } from './admin-filters-toolbar';

export type MenuTab = 'items' | 'sections';
export type MenuManagementData = ReturnType<typeof useMenuManagementData>;

export function MenuManagementView() {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<MenuTab>('items');
  const [itemFilters, setItemFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const [sectionFilters, setSectionFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const data = useMenuManagementData({ itemFilters, sectionFilters });
  const formState = useMenuManagementForms(data.allSections.items);
  const deleteState = useMenuDeleteState();
  const actionState = useMenuManagementActions({ formState, deleteState });
  const filters = tab === 'items' ? itemFilters : sectionFilters;
  const activeResource = tab === 'items' ? data.items : data.sections;
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
      if (tab === 'items') {
        data.itemTable.onResetPage();
        setItemFilters(nextFilters);
        return;
      }
      data.sectionTable.onResetPage();
      setSectionFilters(nextFilters);
    },
    [data.itemTable, data.sectionTable, tab]
  );

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        heading={DASHBOARD_MENU_TITLES.menuManagement}
        action={
          <Stack direction="row" spacing={1}>
            <RefreshButton loading={activeResource.isLoading} onClick={() => void activeResource.refresh()} />
            <AddButton
              onClick={tab === 'items' ? formState.item.openCreate : formState.section.openCreate}
            >
              {t(tab === 'items' ? 'actions.addMenuItem' : 'actions.addSection')}
            </AddButton>
          </Stack>
        }
      />

      <Card>
        <Tabs value={tab} onChange={(_event, value: MenuTab) => setTab(value)} sx={{ px: 2.5 }}>
          <Tab value="items" label={t('common.menus')} />
          <Tab value="sections" label={t('common.section')} />
        </Tabs>
        <AdminFiltersToolbar
          filters={filters}
          searchPlaceholder={t(tab === 'items' ? 'filters.searchMenus' : 'filters.searchSections')}
          onChange={handleFiltersChange}
        />
        <MenuManagementTables
          tab={tab}
          data={data}
          formState={formState}
          deleteState={deleteState}
        />
      </Card>

      <MenuManagementDialogs
        data={data}
        formState={formState}
        deleteState={deleteState}
        actionState={actionState}
      />
    </DashboardContent>
  );
}

export function useMenuManagementData({
  itemFilters,
  sectionFilters,
}: {
  itemFilters: typeof DEFAULT_ADMIN_FILTERS;
  sectionFilters: typeof DEFAULT_ADMIN_FILTERS;
}) {
  const sectionTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const itemTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const sections = useMenuSections(
    sectionTable.page,
    sectionTable.rowsPerPage,
    toEnabledFilters(sectionFilters)
  );
  const allSections = useMenuSections(0, 100);
  const items = useMenuItems(itemTable.page, itemTable.rowsPerPage, toEnabledFilters(itemFilters));
  const allItems = useMenuItems(0, 100);
  const apis = useApis(0, 100);

  const sectionNameById = useMemo(
    () => new Map(allSections.items.map((section) => [section.id, section.subheader])),
    [allSections.items]
  );

  return { allItems, allSections, apis, itemTable, items, sectionNameById, sectionTable, sections };
}
