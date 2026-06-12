'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { AuthSplitSectionProps } from './section';
import type { AuthSplitContentProps } from './content';
import type { MainSectionProps, HeaderSectionProps, LayoutSectionProps } from '../core';

import { useMemo } from 'react';
import { merge } from 'es-toolkit';

import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/locales/use-locales';
import { useSiteInfo } from 'src/actions/system-settings';
import { normalizePublicSiteInfo } from 'src/actions/site-info-utils';

import { Logo } from 'src/components/logo';

import { AuthSplitContent } from './content';
import { SettingsButton } from '../components/settings-button';
import { MainSection, LayoutSection, HeaderSection } from '../core';
import { AuthSplitSection, AuthSplitSectionStatus } from './section';

// ----------------------------------------------------------------------

type LayoutBaseProps = Pick<LayoutSectionProps, 'sx' | 'children' | 'cssVars'>;

export type AuthSplitLayoutProps = LayoutBaseProps & {
  layoutQuery?: Breakpoint;
  slotProps?: {
    header?: HeaderSectionProps;
    main?: MainSectionProps;
    section?: AuthSplitSectionProps;
    content?: AuthSplitContentProps;
  };
};

export function AuthSplitLayout({
  sx,
  cssVars,
  children,
  slotProps,
  layoutQuery = 'md',
}: AuthSplitLayoutProps) {
  const { t } = useTranslate('auth');
  const site = useSiteInfo();
  const siteInfo = useMemo(() => normalizePublicSiteInfo(site.data), [site.data]);
  const siteError = useMemo(() => {
    if (site.error) {
      return site.error;
    }

    if (site.data && !siteInfo) {
      return new Error(t('siteInfoStatus.invalidSiteName'));
    }

    return undefined;
  }, [site.data, site.error, siteInfo, t]);

  const renderHeader = () => {
    const headerSlotProps: HeaderSectionProps['slotProps'] = {
      container: { maxWidth: false },
    };

    const headerSlots: HeaderSectionProps['slots'] = {
      topArea: (
        <Alert severity="info" sx={{ display: 'none', borderRadius: 0 }}>
          This is an info Alert.
        </Alert>
      ),
      leftArea: (
        <>
          {/** @slot Logo */}
          <Logo label={siteInfo?.site_name} source={siteInfo?.site_logo_base64 ?? ''} />
        </>
      ),
      rightArea: <SettingsButton />,
    };

    return (
      <HeaderSection
        disableElevation
        layoutQuery={layoutQuery}
        {...slotProps?.header}
        slots={{ ...headerSlots, ...slotProps?.header?.slots }}
        slotProps={merge(headerSlotProps, slotProps?.header?.slotProps ?? {})}
        sx={[
          { position: { [layoutQuery]: 'fixed' } },
          ...(Array.isArray(slotProps?.header?.sx) ? slotProps.header.sx : [slotProps?.header?.sx]),
        ]}
      />
    );
  };

  const renderFooter = () => null;

  const renderSection = () =>
    siteInfo ? (
      <AuthSplitSection layoutQuery={layoutQuery} siteName={siteInfo.site_name} {...slotProps?.section} />
    ) : null;

  const renderContent = () => {
    if (siteError || !siteInfo) {
      return <AuthSplitSectionStatus error={siteError} onRetry={() => void site.refresh()} />;
    }

    return children;
  };

  const renderMain = () => (
    <MainSection
      {...slotProps?.main}
      sx={[
        (theme) => ({
          minHeight: 'calc(100vh - var(--layout-header-mobile-height))',
          [theme.breakpoints.up(layoutQuery)]: {
            minHeight: '100vh',
            flexDirection: 'row',
          },
        }),
        ...(Array.isArray(slotProps?.main?.sx) ? slotProps.main.sx : [slotProps?.main?.sx]),
      ]}
    >
      {renderSection()}
      <AuthSplitContent layoutQuery={layoutQuery} {...slotProps?.content}>
        {renderContent()}
      </AuthSplitContent>
    </MainSection>
  );

  return (
    <LayoutSection
      /** **************************************
       * @Header
       *************************************** */
      headerSection={renderHeader()}
      /** **************************************
       * @Footer
       *************************************** */
      footerSection={renderFooter()}
      /** **************************************
       * @Styles
       *************************************** */
      cssVars={{ '--layout-auth-content-width': '420px', ...cssVars }}
      sx={sx}
    >
      {renderMain()}
    </LayoutSection>
  );
}
