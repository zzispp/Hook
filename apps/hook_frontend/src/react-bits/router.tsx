'use client';

import type { ReactNode, ComponentProps } from 'react';

import NextLink from 'next/link';
import { usePathname } from 'next/navigation';

type NextLinkProps = ComponentProps<typeof NextLink>;

type LinkProps = Omit<NextLinkProps, 'href'> & {
  readonly to?: NextLinkProps['href'];
  readonly href?: NextLinkProps['href'];
  readonly children: ReactNode;
};

export function Link({ to, href, children, ...props }: LinkProps) {
  return (
    <NextLink href={href ?? to ?? '#'} {...props}>
      {children}
    </NextLink>
  );
}

export function useLocation(): { pathname: string } {
  return { pathname: usePathname() ?? '/' };
}
