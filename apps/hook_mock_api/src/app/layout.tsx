import type { Metadata } from 'next';

import CssBaseline from '@mui/material/CssBaseline';

import colors from 'src/colors.json';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: 'API - Minimal UI',
  description: 'Demo API for Minimal UI',
  icons: [
    {
      rel: 'icon',
      url: `/favicon.ico`,
    },
  ],
};

type RootLayoutProps = {
  children: React.ReactNode;
};

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en">
      <body
        style={{
          color: colors.common.white,
          backgroundColor: colors.grey[900],
        }}
      >
        <CssBaseline />
        {children}
      </body>
    </html>
  );
}
