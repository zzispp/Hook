import type { Metadata } from 'next';

import LandingPage from 'src/react-bits/pages/LandingPage';


// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  return {
    title: 'React Bits - Animated UI Components For React',
    description: 'Animated UI components for React.',
  };
}

export default function Page() {
  return <LandingPage />;
}
