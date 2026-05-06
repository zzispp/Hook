import type { CardProps } from '@mui/material/Card';
import type { IPaymentCard } from 'src/types/common';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import CardHeader from '@mui/material/CardHeader';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';

import { Iconify } from 'src/components/iconify';

import { PaymentCardItem } from '../payment/payment-card-item';
import { PaymentCardCreateForm } from '../payment/payment-card-create-form';

// ----------------------------------------------------------------------

type Props = CardProps & {
  cards: IPaymentCard[];
};

export function AccountBillingPayment({ cards, sx, ...other }: Props) {
  const openForm = useBoolean();

  const renderCardCreateFormDialog = () => (
    <Dialog fullWidth maxWidth="xs" open={openForm.value} onClose={openForm.onFalse}>
      <DialogTitle>Add card</DialogTitle>

      <PaymentCardCreateForm sx={{ px: 3 }} />

      <DialogActions>
        <Button color="inherit" variant="outlined" onClick={openForm.onFalse}>
          Cancel
        </Button>
        <Button color="inherit" variant="contained" onClick={openForm.onFalse}>
          Add
        </Button>
      </DialogActions>
    </Dialog>
  );

  return (
    <>
      <Card sx={[{ my: 3 }, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
        <CardHeader
          title="Payment method"
          action={
            <Button
              size="small"
              color="primary"
              startIcon={<Iconify icon="mingcute:add-line" />}
              onClick={openForm.onTrue}
            >
              Add card
            </Button>
          }
        />

        <Box
          sx={{
            p: 3,
            rowGap: 2.5,
            columnGap: 2,
            display: 'grid',
            gridTemplateColumns: { xs: 'repeat(1, 1fr)', md: 'repeat(2, 1fr)' },
          }}
        >
          {cards.map((card) => (
            <PaymentCardItem key={card.id} card={card} />
          ))}
        </Box>
      </Card>

      {renderCardCreateFormDialog()}
    </>
  );
}
