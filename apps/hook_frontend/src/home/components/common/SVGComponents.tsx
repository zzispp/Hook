import type { CSSProperties, HTMLAttributes } from 'react';

import { CONFIG } from 'src/global-config';

import { logoDisplaySource, isMaskableLogoSource } from 'src/components/logo/logo-utils';

type LogoVariant = 'full' | 'icon';
type LogoProps = HTMLAttributes<HTMLElement> & {
  variant?: LogoVariant;
  label?: string;
  source?: string;
};

const DEFAULT_LOGO_SOURCES: Record<LogoVariant, string> = {
  full: '/logo/logo.svg',
  icon: '/logo/logo-icon.svg',
};

const LOGO_SIZES: Record<LogoVariant, Pick<CSSProperties, 'height' | 'width'>> = {
  full: { width: 100, height: 36 },
  icon: { width: 36, height: 36 },
};

export const Logo = ({
  variant = 'full',
  label = 'Logo',
  source,
  className,
  style,
  ...other
}: LogoProps) => {
  const displaySource = logoSource(variant, source);

  if (!displaySource) {
    return (
      <span
        aria-hidden
        className={['ln-brand-logo', className].filter(Boolean).join(' ')}
        style={emptyLogoStyle(variant, style)}
        {...other}
      />
    );
  }

  if (isMaskableLogoSource(displaySource)) {
    return (
      <span
        role="img"
        aria-label={label}
        className={['ln-brand-logo', className].filter(Boolean).join(' ')}
        style={maskLogoStyle(variant, displaySource, style)}
        {...other}
      />
    );
  }

  return (
    <img
      alt={label}
      className={['ln-brand-logo', className].filter(Boolean).join(' ')}
      src={assetAwareSource(displaySource)}
      style={imageLogoStyle(variant, style)}
      {...other}
    />
  );
};

export const LogoIcon = (props: Omit<LogoProps, 'variant'>) => <Logo variant="icon" {...props} />;

function logoSource(variant: LogoVariant, source?: string) {
  const configuredSource = source === undefined ? DEFAULT_LOGO_SOURCES[variant] : source;

  return logoDisplaySource(configuredSource, { isSingle: variant === 'icon' });
}

function emptyLogoStyle(variant: LogoVariant, style?: CSSProperties): CSSProperties {
  return {
    ...LOGO_SIZES[variant],
    display: 'inline-block',
    flexShrink: 0,
    ...style,
  };
}

function maskLogoStyle(variant: LogoVariant, source: string, style?: CSSProperties): CSSProperties {
  const assetUrl = assetAwareSource(source);

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

function imageLogoStyle(variant: LogoVariant, style?: CSSProperties): CSSProperties {
  return {
    ...LOGO_SIZES[variant],
    display: 'inline-block',
    flexShrink: 0,
    objectFit: 'contain',
    objectPosition: 'center',
    ...style,
  };
}

function assetAwareSource(source: string) {
  if (source.startsWith('/') && CONFIG.assetsDir) {
    return `${CONFIG.assetsDir}${source}`;
  }

  return source;
}
