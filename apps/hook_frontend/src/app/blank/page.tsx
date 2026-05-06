import type { Metadata } from 'next';

import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/global-config';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Blank - ${CONFIG.appName}` };

export default function Page() {
  return (
    <Container>
      <Typography variant="h4">Blank</Typography>
    </Container>
  );
}
