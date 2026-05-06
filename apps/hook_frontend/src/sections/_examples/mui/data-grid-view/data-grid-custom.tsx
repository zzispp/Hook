import type { RatingProps } from '@mui/material/Rating';
import type {
  GridColDef,
  GridFilterOperator,
  GridRowSelectionModel,
  GridColumnVisibilityModel,
  GridFilterInputValueProps,
} from '@mui/x-data-grid';
import type { IDateValue } from 'src/types/common';
import type { CustomToolbarSettingsButtonProps } from 'src/components/custom-data-grid';

import { useRef, useMemo, useState, useImperativeHandle } from 'react';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Avatar from '@mui/material/Avatar';
import Rating from '@mui/material/Rating';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';
import { Toolbar, DataGrid, gridClasses } from '@mui/x-data-grid';

import { fPercent } from 'src/utils/format-number';
import { fDate, fTime } from 'src/utils/format-time';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { EmptyContent } from 'src/components/empty-content';
import {
  ToolbarContainer,
  ToolbarLeftPanel,
  ToolbarRightPanel,
  useToolbarSettings,
  CustomToolbarQuickFilter,
  CustomToolbarExportButton,
  CustomToolbarFilterButton,
  CustomGridActionsCellItem,
  CustomToolbarColumnsButton,
  CustomToolbarSettingsButton,
} from 'src/components/custom-data-grid';

// ----------------------------------------------------------------------

const baseColumns: GridColDef[] = [
  {
    field: 'id',
    headerName: 'Id',
    filterable: false,
  },
  {
    field: 'name',
    headerName: 'Name',
    flex: 1,
    minWidth: 160,
    hideable: false,
    renderCell: (params) => (
      <Box sx={{ gap: 2, width: 1, display: 'flex', alignItems: 'center' }}>
        <Avatar alt={params.row.name} sx={{ width: 32, height: 32 }}>
          {params.row.name.charAt(0).toUpperCase()}
        </Avatar>
        <Typography component="span" variant="body2" noWrap>
          {params.row.name}
        </Typography>
      </Box>
    ),
  },
  {
    field: 'email',
    headerName: 'Email',
    flex: 1,
    minWidth: 160,
    editable: true,
    renderCell: (params) => (
      <Link color="inherit" noWrap>
        {params.row.email}
      </Link>
    ),
  },
  {
    type: 'string',
    field: 'lastLogin',
    headerName: 'Last login',
    align: 'right',
    headerAlign: 'right',
    width: 120,
    renderCell: (params) => (
      <Box
        sx={{
          gap: 0.5,
          display: 'flex',
          lineHeight: 'normal',
          flexDirection: 'column',
        }}
      >
        {fDate(params.row.lastLogin)}
        <Box component="span" sx={{ color: 'text.secondary', typography: 'caption' }}>
          {fTime(params.row.lastLogin)}
        </Box>
      </Box>
    ),
  },
  {
    type: 'number',
    field: 'rating',
    headerName: 'Rating',
    width: 140,
    renderCell: (params) => (
      <Rating size="small" value={params.row.rating} precision={0.5} readOnly />
    ),
  },
  {
    type: 'singleSelect',
    field: 'status',
    headerName: 'Status',
    align: 'center',
    headerAlign: 'center',
    width: 100,
    editable: true,
    valueOptions: ['online', 'always', 'busy'],
    renderCell: (params) => (
      <Label
        variant="soft"
        color={
          (params.row.status === 'busy' && 'error') ||
          (params.row.status === 'always' && 'warning') ||
          'success'
        }
      >
        {params.row.status}
      </Label>
    ),
  },
  {
    type: 'boolean',
    field: 'isAdmin',
    align: 'center',
    headerAlign: 'center',
    width: 80,
    renderCell: (params) =>
      params.row.isAdmin ? (
        <Iconify icon="solar:check-circle-bold" sx={{ color: 'primary.main' }} />
      ) : (
        '-'
      ),
  },
  {
    type: 'number',
    field: 'performance',
    headerName: 'Performance',
    align: 'center',
    headerAlign: 'center',
    width: 160,
    renderCell: (params) => (
      <Box
        sx={{
          px: 1,
          gap: 1,
          width: 1,
          display: 'flex',
          alignItems: 'center',
        }}
      >
        <LinearProgress
          value={params.row.performance}
          variant="determinate"
          color={
            (params.row.performance < 30 && 'error') ||
            (params.row.performance > 30 && params.row.performance < 70 && 'warning') ||
            'primary'
          }
          sx={{ width: 1, height: 6 }}
        />
        <Typography variant="caption" sx={{ width: 80 }}>
          {fPercent(params.row.performance)}
        </Typography>
      </Box>
    ),
  },
  {
    type: 'actions',
    field: 'actions',
    headerName: 'Actions',
    align: 'right',
    headerAlign: 'right',
    width: 80,
    sortable: false,
    filterable: false,
    disableColumnMenu: true,
    getActions: (params) => [
      <CustomGridActionsCellItem
        showInMenu
        label="View"
        icon={<Iconify icon="solar:eye-bold" />}
        onClick={() => console.info('VIEW', params.row.id)}
      />,
      <CustomGridActionsCellItem
        showInMenu
        label="Edit"
        icon={<Iconify icon="solar:pen-bold" />}
        onClick={() => console.info('EDIT', params.row.id)}
      />,
      <CustomGridActionsCellItem
        showInMenu
        label="Delete"
        icon={<Iconify icon="solar:trash-bin-trash-bold" />}
        onClick={() => console.info('DELETE', params.row.id)}
        style={{ color: 'var(--palette-error-main)' }}
      />,
    ],
  },
];

// ----------------------------------------------------------------------

type Props = {
  data: {
    id: string;
    age: number;
    name: string;
    email: string;
    rating: number;
    status: string;
    isAdmin: boolean;
    lastName: string;
    firstName: string;
    performance: number;
    lastLogin: IDateValue;
  }[];
};

const HIDE_COLUMNS = { id: false };
const HIDE_COLUMNS_TOGGLABLE = ['id', 'actions'];

export function DataGridCustom({ data: rows }: Props) {
  const toolbarOptions = useToolbarSettings();

  const [selectedRows, setSelectedRows] = useState<GridRowSelectionModel>({
    type: 'include',
    ids: new Set(),
  });

  const [columnVisibilityModel, setColumnVisibilityModel] =
    useState<GridColumnVisibilityModel>(HIDE_COLUMNS);

  const columns = useMemo(
    () =>
      baseColumns.map((col) =>
        col.field === 'rating' ? { ...col, filterOperators: ratingOnlyOperators } : col
      ),
    []
  );

  console.info('SELECTED ROWS', Array.from(selectedRows.ids));

  return (
    <DataGrid
      {...toolbarOptions.settings}
      checkboxSelection
      disableRowSelectionOnClick
      rows={rows}
      columns={columns}
      pageSizeOptions={[5, 10, 20, 50, { value: -1, label: 'All' }]}
      initialState={{ pagination: { paginationModel: { pageSize: 10 } } }}
      columnVisibilityModel={columnVisibilityModel}
      onColumnVisibilityModelChange={(newModel) => setColumnVisibilityModel(newModel)}
      onRowSelectionModelChange={(newRowSelectionModel) => setSelectedRows(newRowSelectionModel)}
      slots={{
        noRowsOverlay: () => <EmptyContent />,
        noResultsOverlay: () => <EmptyContent title="No results found" />,
        toolbar: () => (
          <CustomToolbar
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
  );
}

// ----------------------------------------------------------------------

type CustomToolbarProps = CustomToolbarSettingsButtonProps;

function CustomToolbar({ settings, onChangeSettings }: CustomToolbarProps) {
  return (
    <Toolbar>
      <ToolbarContainer>
        <ToolbarLeftPanel>
          <CustomToolbarQuickFilter />
        </ToolbarLeftPanel>

        <ToolbarRightPanel>
          <CustomToolbarColumnsButton />
          <CustomToolbarFilterButton />
          <CustomToolbarExportButton />
          <CustomToolbarSettingsButton settings={settings} onChangeSettings={onChangeSettings} />
        </ToolbarRightPanel>
      </ToolbarContainer>
    </Toolbar>
  );
}

// ----------------------------------------------------------------------

function RatingInput({ item, applyValue, focusElementRef }: GridFilterInputValueProps) {
  const ratingRef = useRef<any>(null);

  useImperativeHandle(focusElementRef, () => ({
    focus: () => {
      ratingRef.current.querySelector(`input[value="${Number(item.value) || ''}"]`).focus();
    },
  }));

  const handleFilter: RatingProps['onChange'] = (event, newValue) => {
    applyValue({ ...item, value: newValue });
  };

  return (
    <Rating
      ref={ratingRef}
      precision={0.5}
      value={Number(item.value)}
      onChange={handleFilter}
      name="custom-rating-filter-operator"
      sx={{ ml: 2 }}
    />
  );
}

const ratingOnlyOperators: GridFilterOperator<any, number>[] = [
  {
    label: 'Above',
    value: 'above',
    getValueAsString: (value: number) => `${value} Stars`,
    getApplyFilterFn: (filterItem) => {
      if (!filterItem.field || !filterItem.value || !filterItem.operator) {
        return null;
      }

      return (value) => Number(value) >= Number(filterItem.value);
    },
    InputComponent: RatingInput,
  },
  {
    label: 'Below',
    value: 'below',
    getValueAsString: (value: number) => `${value} Stars`,
    getApplyFilterFn: (filterItem) => {
      if (!filterItem.field || !filterItem.value || !filterItem.operator) {
        return null;
      }

      return (value) => Number(value) <= Number(filterItem.value);
    },
    InputComponent: RatingInput,
  },
];
