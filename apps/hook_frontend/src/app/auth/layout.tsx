import { AuthI18nGate } from 'src/locales/auth-i18n-gate';

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  return <AuthI18nGate>{children}</AuthI18nGate>;
}
