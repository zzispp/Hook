import type { IInvoice } from 'src/types/invoice';

import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';
import { useRouter } from 'src/routes/hooks';

import { today, fIsAfter } from 'src/utils/format-time';

import { _addressBooks } from 'src/_mock';

import { Form, schemaUtils } from 'src/components/hook-form';

import { InvoiceCreateEditAddress } from './invoice-create-edit-address';
import { InvoiceCreateEditStatusDate } from './invoice-create-edit-status-date';
import { defaultItem, InvoiceCreateEditDetails } from './invoice-create-edit-details';

// ----------------------------------------------------------------------

export type InvoiceCreateSchemaType = z.infer<typeof InvoiceCreateSchema>;

export const InvoiceCreateSchema = z
  .object({
    invoiceTo: schemaUtils.nullableInput(z.custom<IInvoice['invoiceTo']>(), {
      error: 'Invoice to is required!',
    }),
    createDate: schemaUtils.date({ error: { required: 'Create date is required!' } }),
    dueDate: schemaUtils.date({ error: { required: 'Due date is required!' } }),
    items: z.array(
      z.object({
        title: z.string().min(1, { error: 'Title is required!' }),
        service: z.string().min(1, { error: 'Service is required!' }),
        quantity: z.number().int().positive().min(1, { error: 'Quantity must be more than 0' }),
        // Not required
        price: z.number(),
        total: z.number(),
        description: z.string(),
      })
    ),
    // Not required
    taxes: z.number(),
    status: z.string(),
    discount: z.number(),
    shipping: z.number(),
    subtotal: z.number(),
    totalAmount: z.number(),
    invoiceNumber: z.string(),
    invoiceFrom: z.custom<IInvoice['invoiceFrom']>().nullable(),
  })
  .refine((val) => !fIsAfter(val.createDate, val.dueDate), {
    error: 'Due date cannot be earlier than create date!',
    path: ['dueDate'],
  });

// ----------------------------------------------------------------------

type Props = {
  currentInvoice?: IInvoice;
};

export function InvoiceCreateEditForm({ currentInvoice }: Props) {
  const router = useRouter();

  const loadingSave = useBoolean();
  const loadingSend = useBoolean();

  const defaultValues: InvoiceCreateSchemaType = {
    invoiceNumber: 'INV-1990',
    createDate: today(),
    dueDate: null,
    taxes: 0,
    shipping: 0,
    status: 'draft',
    discount: 0,
    invoiceFrom: _addressBooks[0],
    invoiceTo: null,
    subtotal: 0,
    totalAmount: 0,
    items: [defaultItem],
  };

  const methods = useForm({
    mode: 'all',
    resolver: zodResolver(InvoiceCreateSchema),
    defaultValues,
    values: currentInvoice,
  });

  const {
    reset,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const handleSaveAsDraft = handleSubmit(async (data) => {
    loadingSave.onTrue();

    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      loadingSave.onFalse();
      router.push(paths.dashboard.invoice.root);
      console.info('DATA', JSON.stringify(data, null, 2));
    } catch (error) {
      console.error(error);
      loadingSave.onFalse();
    }
  });

  const handleCreateAndSend = handleSubmit(async (data) => {
    loadingSend.onTrue();

    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      loadingSend.onFalse();
      router.push(paths.dashboard.invoice.root);
      console.info('DATA', JSON.stringify(data, null, 2));
    } catch (error) {
      console.error(error);
      loadingSend.onFalse();
    }
  });

  return (
    <Form methods={methods}>
      <Card>
        <InvoiceCreateEditAddress />
        <InvoiceCreateEditStatusDate />
        <InvoiceCreateEditDetails />
      </Card>

      <Box
        sx={{
          mt: 3,
          gap: 2,
          display: 'flex',
          justifyContent: 'flex-end',
        }}
      >
        <Button
          color="inherit"
          size="large"
          variant="outlined"
          loading={loadingSave.value && isSubmitting}
          onClick={handleSaveAsDraft}
        >
          Save as draft
        </Button>

        <Button
          size="large"
          variant="contained"
          loading={loadingSend.value && isSubmitting}
          onClick={handleCreateAndSend}
        >
          {currentInvoice ? 'Update' : 'Create'} & send
        </Button>
      </Box>
    </Form>
  );
}
