'use client';

import type {
  GridColDef,
  GridRowSelectionModel,
  GridColumnVisibilityModel,
} from '@mui/x-data-grid';
import type { IProductItem, IProductTableFilters } from 'src/types/product';

import { useBoolean, useSetState } from 'minimal-shared/hooks';
import { useMemo, useState, useEffect, useCallback } from 'react';

import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import { useTheme } from '@mui/material/styles';
import { DataGrid, gridClasses } from '@mui/x-data-grid';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { PRODUCT_STOCK_OPTIONS } from 'src/_mock';
import { useGetProducts } from 'src/actions/product';
import { DashboardContent } from 'src/layouts/dashboard';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { EmptyContent } from 'src/components/empty-content';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';
import { useToolbarSettings, CustomGridActionsCellItem } from 'src/components/custom-data-grid';

import { ProductTableToolbar } from '../product-table-toolbar';
import {
  RenderCellStock,
  RenderCellPrice,
  RenderCellProduct,
  RenderCellPublish,
  RenderCellCreatedAt,
} from '../product-table-row';

// ----------------------------------------------------------------------

const PUBLISH_OPTIONS = [
  { value: 'published', label: 'Published' },
  { value: 'draft', label: 'Draft' },
];

const HIDE_COLUMNS = { category: false };
const HIDE_COLUMNS_TOGGLABLE = ['category', 'actions'];

// ----------------------------------------------------------------------

export function ProductListView() {
  const confirmDialog = useBoolean();
  const toolbarOptions = useToolbarSettings();
  const { products, productsLoading } = useGetProducts();

  const [tableData, setTableData] = useState<IProductItem[]>(products);
  const [selectedRows, setSelectedRows] = useState<GridRowSelectionModel>({
    type: 'include',
    ids: new Set(),
  });

  const filters = useSetState<IProductTableFilters>({
    publish: [],
    stock: [],
  });

  const [columnVisibilityModel, setColumnVisibilityModel] =
    useState<GridColumnVisibilityModel>(HIDE_COLUMNS);

  useEffect(() => {
    setTableData(products);
  }, [products]);

  const canReset = filters.state.publish.length > 0 || filters.state.stock.length > 0;

  const dataFiltered = applyFilter({
    inputData: tableData,
    filters: filters.state,
  });

  const handleDeleteRow = useCallback((id: string) => {
    setTableData((prev) => prev.filter((row) => row.id !== id));
    toast.success('Delete success!');
  }, []);

  const handleDeleteRows = useCallback(() => {
    setTableData((prev) => prev.filter((row) => !selectedRows.ids.has(row.id)));
    toast.success('Delete success!');
  }, [selectedRows.ids]);

  const columns = useGetColumns({ onDeleteRow: handleDeleteRow });

  const renderConfirmDialog = () => (
    <ConfirmDialog
      open={confirmDialog.value}
      onClose={confirmDialog.onFalse}
      title="Delete"
      content={
        <>
          Are you sure want to delete <strong> {selectedRows.ids.size} </strong> items?
        </>
      }
      action={
        <Button
          variant="contained"
          color="error"
          onClick={() => {
            handleDeleteRows();
            confirmDialog.onFalse();
          }}
        >
          Delete
        </Button>
      }
    />
  );

  return (
    <>
      <DashboardContent sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        <CustomBreadcrumbs
          heading="List"
          links={[
            { name: 'Dashboard', href: paths.dashboard.root },
            { name: 'Product', href: paths.dashboard.product.root },
            { name: 'List' },
          ]}
          action={
            <Button
              component={RouterLink}
              href={paths.dashboard.product.new}
              variant="contained"
              startIcon={<Iconify icon="mingcute:add-line" />}
            >
              Add product
            </Button>
          }
          sx={{ mb: { xs: 3, md: 5 } }}
        />

        <Card
          sx={{
            minHeight: 640,
            flexGrow: { md: 1 },
            display: { md: 'flex' },
            height: { xs: 800, md: '1px' },
            flexDirection: { md: 'column' },
          }}
        >
          <DataGrid
            {...toolbarOptions.settings}
            checkboxSelection
            disableRowSelectionOnClick
            rows={dataFiltered}
            columns={columns}
            loading={productsLoading}
            getRowHeight={() => 'auto'}
            pageSizeOptions={[5, 10, 20, { value: -1, label: 'All' }]}
            initialState={{ pagination: { paginationModel: { pageSize: 10 } } }}
            columnVisibilityModel={columnVisibilityModel}
            onColumnVisibilityModelChange={(newModel) => setColumnVisibilityModel(newModel)}
            onRowSelectionModelChange={(newSelectionModel) => setSelectedRows(newSelectionModel)}
            slots={{
              noRowsOverlay: () => <EmptyContent />,
              noResultsOverlay: () => <EmptyContent title="No results found" />,
              toolbar: () => (
                <ProductTableToolbar
                  filters={filters}
                  canReset={canReset}
                  filteredResults={dataFiltered.length}
                  selectedRowCount={selectedRows.ids.size}
                  onOpenConfirmDeleteRows={confirmDialog.onTrue}
                  options={{ stocks: PRODUCT_STOCK_OPTIONS, publishs: PUBLISH_OPTIONS }}
                  /********/
                  settings={toolbarOptions.settings}
                  onChangeSettings={toolbarOptions.onChangeSettings}
                />
              ),
            }}
            slotProps={{
              columnsManagement: {
                getTogglableColumns: () =>
                  columns
                    .filter((col) => !HIDE_COLUMNS_TOGGLABLE.includes(col.field))
                    .map((col) => col.field),
              },
            }}
            sx={{
              [`& .${gridClasses.cell}`]: {
                display: 'flex',
                alignItems: 'center',
              },
            }}
          />
        </Card>
      </DashboardContent>

      {renderConfirmDialog()}
    </>
  );
}

// ----------------------------------------------------------------------

type UseGetColumnsProps = {
  onDeleteRow: (id: string) => void;
};

const useGetColumns = ({ onDeleteRow }: UseGetColumnsProps) => {
  const theme = useTheme();

  const columns: GridColDef[] = useMemo(
    () => [
      {
        field: 'category',
        headerName: 'Category',
        filterable: false,
      },
      {
        field: 'name',
        headerName: 'Product',
        flex: 1,
        minWidth: 360,
        hideable: false,
        renderCell: (params) => (
          <RenderCellProduct
            params={params}
            href={paths.dashboard.product.details(params.row.id)}
          />
        ),
      },
      {
        field: 'createdAt',
        headerName: 'Create at',
        width: 160,
        renderCell: (params) => <RenderCellCreatedAt params={params} />,
      },
      {
        field: 'inventoryType',
        headerName: 'Stock',
        width: 160,
        type: 'singleSelect',
        filterable: false,
        valueOptions: PRODUCT_STOCK_OPTIONS,
        renderCell: (params) => <RenderCellStock params={params} />,
      },
      {
        field: 'price',
        headerName: 'Price',
        width: 120,
        editable: true,
        renderCell: (params) => <RenderCellPrice params={params} />,
      },
      {
        field: 'publish',
        headerName: 'Publish',
        width: 120,
        type: 'singleSelect',
        editable: true,
        filterable: false,
        valueOptions: PUBLISH_OPTIONS,
        renderCell: (params) => <RenderCellPublish params={params} />,
      },
      {
        type: 'actions',
        field: 'actions',
        headerName: ' ',
        width: 64,
        align: 'right',
        headerAlign: 'right',
        sortable: false,
        filterable: false,
        disableColumnMenu: true,
        getActions: (params) => [
          <CustomGridActionsCellItem
            showInMenu
            label="View"
            icon={<Iconify icon="solar:eye-bold" />}
            href={paths.dashboard.product.details(params.row.id)}
          />,
          <CustomGridActionsCellItem
            showInMenu
            label="Edit"
            icon={<Iconify icon="solar:pen-bold" />}
            href={paths.dashboard.product.edit(params.row.id)}
          />,
          <CustomGridActionsCellItem
            showInMenu
            label="Delete"
            icon={<Iconify icon="solar:trash-bin-trash-bold" />}
            onClick={() => onDeleteRow(params.row.id)}
            style={{ color: theme.vars.palette.error.main }}
          />,
        ],
      },
    ],
    [onDeleteRow, theme.vars.palette.error.main]
  );

  return columns;
};

// ----------------------------------------------------------------------

type ApplyFilterProps = {
  inputData: IProductItem[];
  filters: IProductTableFilters;
};

function applyFilter({ inputData, filters }: ApplyFilterProps) {
  const { stock, publish } = filters;

  if (stock.length) {
    inputData = inputData.filter((product) => stock.includes(product.inventoryType));
  }

  if (publish.length) {
    inputData = inputData.filter((product) => publish.includes(product.publish));
  }

  return inputData;
}
