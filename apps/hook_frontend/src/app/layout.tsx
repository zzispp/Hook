import 'src/global.css';
import '@rainbow-me/rainbowkit/styles.css';

import type { Metadata, Viewport } from 'next';

import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';
import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { CONFIG } from 'src/global-config';
import { LocalizationProvider } from 'src/locales';
import { detectLanguage } from 'src/locales/server';
import { I18nProvider } from 'src/locales/i18n-provider';
import { themeConfig, ThemeProvider, primary as primaryColor } from 'src/theme';

import { Snackbar } from 'src/components/snackbar';
import { ProgressBar } from 'src/components/progress-bar';
import { MotionLazy } from 'src/components/animate/motion-lazy';
import { SettingsDrawer, defaultSettings, SettingsProvider } from 'src/components/settings';

import { AuthProvider } from 'src/auth/context/jwt';
import { WalletConnectorProvider } from 'src/auth/context/jwt/wallet-connector-provider';

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: primaryColor.main,
};

export const metadata: Metadata = {
  icons: [
    {
      rel: 'icon',
      url: `${CONFIG.assetsDir}/favicon.svg`,
      type: 'image/svg+xml',
    },
  ],
};

const REACT_BITS_HOME_LIGHT_BACKGROUND = '#F8FAFC';
const REACT_BITS_HOME_DARK_BACKGROUND = '#120F17';

const REACT_BITS_HOME_BACKGROUND_SCRIPT = `
(function () {
  if (window.location.pathname === '/') {
    document.documentElement.setAttribute('data-react-bits-home', 'true');
  }
})();
`;

// ----------------------------------------------------------------------

type RootLayoutProps = {
  children: React.ReactNode;
};

async function getAppConfig() {
  if (CONFIG.isStaticExport) {
    return {
      lang: 'en',
      i18nLang: undefined,
      dir: defaultSettings.direction,
    };
  } else {
    const lang = await detectLanguage();

    return {
      lang,
      i18nLang: lang,
      dir: defaultSettings.direction,
    };
  }
}

export default async function RootLayout({ children }: RootLayoutProps) {
  const appConfig = await getAppConfig();

  return (
    <html lang={appConfig.lang} dir={appConfig.dir} suppressHydrationWarning>
      <head>
        <style>{`
          html[data-react-bits-home='true'],
          html[data-react-bits-home='true'] body {
            background: ${REACT_BITS_HOME_LIGHT_BACKGROUND} !important;
          }

          html[data-react-bits-home='true'][data-color-scheme='dark'],
          html[data-react-bits-home='true'][data-color-scheme='dark'] body {
            background: ${REACT_BITS_HOME_DARK_BACKGROUND} !important;
          }

          html[data-react-bits-home='true'] {
            color-scheme: light;
          }

          html[data-react-bits-home='true'][data-color-scheme='dark'] {
            color-scheme: dark;
          }
        `}</style>
        <script dangerouslySetInnerHTML={{ __html: REACT_BITS_HOME_BACKGROUND_SCRIPT }} />
      </head>
      <body>
        <InitColorSchemeScript
          modeStorageKey={themeConfig.modeStorageKey}
          attribute={themeConfig.cssVariables.colorSchemeSelector}
          defaultMode={themeConfig.defaultMode}
        />

        <I18nProvider lang={appConfig.i18nLang}>
          <WalletConnectorProvider>
            <AuthProvider>
              <SettingsProvider defaultSettings={defaultSettings}>
                <LocalizationProvider>
                  <AppRouterCacheProvider options={{ key: 'css' }}>
                    <ThemeProvider
                      modeStorageKey={themeConfig.modeStorageKey}
                      defaultMode={themeConfig.defaultMode}
                    >
                      <MotionLazy>
                        <Snackbar />
                        <ProgressBar />
                        <SettingsDrawer defaultSettings={defaultSettings} />
                        {children}
                      </MotionLazy>
                    </ThemeProvider>
                  </AppRouterCacheProvider>
                </LocalizationProvider>
              </SettingsProvider>
            </AuthProvider>
          </WalletConnectorProvider>
        </I18nProvider>
      </body>
    </html>
  );
}
