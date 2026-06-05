import type { CSSProperties, ComponentProps } from 'react';

import { CONFIG } from 'src/global-config';

type LogoVariant = 'full' | 'icon';
type LogoProps = ComponentProps<'span'> & {
  variant?: LogoVariant;
};

const LOGO_ASSETS: Record<LogoVariant, string> = {
  full: '/logo/logo.svg',
  icon: '/logo/logo-icon.svg',
};

const LOGO_SIZES: Record<LogoVariant, Pick<CSSProperties, 'height' | 'width'>> = {
  full: { width: 100, height: 36 },
  icon: { width: 36, height: 36 },
};

export const Logo = ({ variant = 'full', className, style, ...other }: LogoProps) => (
  <span
    role="img"
    aria-label="Hook"
    className={['ln-brand-logo', className].filter(Boolean).join(' ')}
    style={logoStyle(variant, style)}
    {...other}
  />
);

export const LogoIcon = (props: Omit<LogoProps, 'variant'>) => <Logo variant="icon" {...props} />;

function logoStyle(variant: LogoVariant, style?: CSSProperties): CSSProperties {
  const assetUrl = `${CONFIG.assetsDir}${LOGO_ASSETS[variant]}`;

  return {
    ...LOGO_SIZES[variant],
    display: 'inline-block',
    flexShrink: 0,
    color: 'var(--landing-logo-color, currentColor)',
    backgroundColor: 'currentColor',
    mask: `url("${assetUrl}") center / contain no-repeat`,
    WebkitMask: `url("${assetUrl}") center / contain no-repeat`,
    ...style,
  };
}
