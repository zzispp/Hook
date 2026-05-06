'use client';

import { useRef, useState, useEffect } from 'react';

import Box from '@mui/material/Box';

import { ProgressLinear } from './progress-linear';
import { ProgressCircular } from './progress-circular';
import { contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

export function ProgressView() {
  const [buffer, setBuffer] = useState(10);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setProgress((oldProgress) => {
        if (oldProgress === 100) {
          return 0;
        }
        const diff = Math.random() * 10;
        return Math.min(oldProgress + diff, 100);
      });
    }, 500);

    return () => {
      clearInterval(timer);
    };
  }, []);

  const progressRef = useRef(() => {});

  useEffect(() => {
    progressRef.current = () => {
      if (progress > 100) {
        setProgress(0);
        setBuffer(10);
      } else {
        const diff = Math.random() * 10;
        const diff2 = Math.random() * 10;
        setProgress(progress + diff);
        setBuffer(progress + diff + diff2);
      }
    };
  });

  useEffect(() => {
    const timer = setInterval(() => {
      progressRef.current();
    }, 500);

    return () => {
      clearInterval(timer);
    };
  }, []);

  const DEMO_COMPONENTS = [
    {
      name: 'Circular',
      component: (
        <Box sx={contentStyles.column()}>
          <ProgressCircular progress={progress} />
        </Box>
      ),
    },
    {
      name: 'Linear',
      component: (
        <Box sx={contentStyles.grid()}>
          <ProgressLinear progress={progress} buffer={buffer} />
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Progress',
        moreLinks: ['https://mui.com/material-ui/react-progress/'],
      }}
    />
  );
}
