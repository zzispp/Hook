import { _mock } from 'src/_mock';

import { useAuthContext } from './use-auth-context';

// To get the user from the <AuthContext/>, you can use

// Change:
// import { useMockedUser } from 'src/auth/hooks';
// const { user } = useMockedUser();

// To:
// import { useAuthContext } from 'src/auth/hooks';
// const { user } = useAuthContext();

// ----------------------------------------------------------------------

export function useMockedUser() {
  const { user: authUser } = useAuthContext();

  const user = {
    id: authUser?.id ?? '8864c717-587d-472a-929a-8e5f298024da-0',
    displayName: authUser?.displayName ?? authUser?.username ?? 'Jaydon Frankie',
    email: authUser?.email ?? 'demo@minimals.cc',
    photoURL: _mock.image.avatar(24),
    phoneNumber: _mock.phoneNumber(1),
    country: _mock.countryNames(1),
    address: '90210 Broadway Blvd',
    state: 'California',
    city: 'San Francisco',
    zipCode: '94116',
    about: 'Praesent turpis. Phasellus viverra nulla ut metus varius laoreet. Phasellus tempus.',
    role: authUser?.role ?? 'admin',
    isPublic: true,
  };

  return { user };
}
