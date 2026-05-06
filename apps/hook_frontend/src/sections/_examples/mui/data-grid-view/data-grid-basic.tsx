import type { GridColDef } from '@mui/x-data-grid';
import type { IDateValue } from 'src/types/common';

import { DataGrid } from '@mui/x-data-grid';

import { Iconify } from 'src/components/iconify';
import { CustomGridActionsCellItem } from 'src/components/custom-data-grid';

// ----------------------------------------------------------------------

const columns: GridColDef[] = [
  { field: 'id', headerName: 'ID', width: 120 },
  {
    field: 'firstName',
    headerName: 'First name',
    width: 160,
    editable: true,
  },
  {
    field: 'lastName',
    headerName: 'Last name',
    width: 160,
    editable: true,
  },
  {
    field: 'age',
    headerName: 'Age',
    type: 'number',
    width: 120,
    editable: true,
    align: 'center',
    headerAlign: 'center',
  },
  {
    field: 'fullName',
    headerName: 'Full name',
    description: 'This column has a value getter and is not sortable.',
    flex: 1,
    renderCell: (params) => `${params.row.firstName} ${params.row.lastName}`,
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
        label="View"
        icon={<Iconify icon="solar:eye-bold" />}
        onClick={() => console.info('VIEW', params.row.id)}
      />,
      <CustomGridActionsCellItem
        label="Edit"
        icon={<Iconify icon="solar:pen-bold" />}
        onClick={() => console.info('EDIT', params.row.id)}
      />,
      <CustomGridActionsCellItem
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

export function DataGridBasic({ data: rows }: Props) {
  return (
    <DataGrid
      checkboxSelection
      disableRowSelectionOnClick
      rows={rows}
      columns={columns}
      pageSizeOptions={[5, 10, 20]}
      initialState={{ pagination: { paginationModel: { pageSize: 10 } } }}
    />
  );
}
