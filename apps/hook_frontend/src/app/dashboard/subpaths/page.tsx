'use client';

import { useEffect } from 'react';

import { useRouter } from 'src/routes/hooks';

// ----------------------------------------------------------------------

export default function Page() {
  const router = useRouter();

  useEffect(() => {
    router.push('/dashboard/subpaths/sub-1/sub-2');
  }, [router]);

  return null;
}
