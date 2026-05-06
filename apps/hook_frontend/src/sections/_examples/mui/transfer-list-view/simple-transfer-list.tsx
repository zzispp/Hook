import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import { useTheme } from '@mui/material/styles';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/components/iconify';

import { not, intersection } from './enhanced-transfer-list';

// ----------------------------------------------------------------------

export function SimpleTransferList() {
  const theme = useTheme();

  const [checked, setChecked] = useState<number[]>([]);

  const [leftList, setLeftList] = useState<number[]>([0, 1, 2, 3]);
  const [rightList, setRightList] = useState<number[]>([4, 5, 6, 7]);

  const leftChecked = intersection(checked, leftList);
  const rightChecked = intersection(checked, rightList);

  const isRtl = theme.direction === 'rtl';

  const handleToggle = useCallback(
    (value: number) => () => {
      const currentIndex = checked.indexOf(value);
      const newChecked = [...checked];

      if (currentIndex === -1) {
        newChecked.push(value);
      } else {
        newChecked.splice(currentIndex, 1);
      }
      setChecked(newChecked);
    },
    [checked]
  );

  const handleAllRight = useCallback(() => {
    setRightList((prevRightList) => {
      setLeftList([]);
      return prevRightList.concat(leftList);
    });
  }, [leftList]);

  const handleCheckedRight = useCallback(() => {
    setRightList((prevRightList) => {
      setLeftList(not(leftList, leftChecked));
      setChecked(not(checked, leftChecked));
      return prevRightList.concat(leftChecked);
    });
  }, [checked, leftChecked, leftList]);

  const handleCheckedLeft = useCallback(() => {
    setLeftList((prevLeftList) => {
      setRightList(not(rightList, rightChecked));
      setChecked(not(checked, rightChecked));
      return prevLeftList.concat(rightChecked);
    });
  }, [checked, rightChecked, rightList]);

  const handleAllLeft = useCallback(() => {
    setLeftList((prevLeftList) => {
      setRightList([]);
      return prevLeftList.concat(rightList);
    });
  }, [rightList]);

  const noData = (
    <Box
      sx={{
        height: 1,
        display: 'flex',
        alignItems: 'center',
        color: 'text.secondary',
        justifyContent: 'center',
        fontWeight: 'fontWeightMedium',
      }}
    >
      No items
    </Box>
  );

  const renderList = (items: number[]) => (
    <Card sx={{ width: 220 }}>
      <List dense component="div" role="list" sx={{ overflow: 'auto', height: 200 }}>
        {items.length
          ? items.map((value: number) => (
              <ListItemButton
                key={value}
                role="listitem"
                onClick={handleToggle(value)}
                sx={{ px: 1, gap: 0.5 }}
              >
                <Checkbox
                  checked={checked.indexOf(value) !== -1}
                  tabIndex={-1}
                  disableRipple
                  slotProps={{
                    input: {
                      id: `simple-${value}-checkbox`,
                      'aria-label': `Simple ${value} checkbox`,
                    },
                  }}
                />

                <ListItemText primary={`List item ${value + 1}`} />
              </ListItemButton>
            ))
          : noData}
      </List>
    </Card>
  );

  const renderControls = () => (
    <Box sx={{ gap: 1, display: 'flex', flexDirection: 'column' }}>
      <Button
        color="inherit"
        variant="outlined"
        size="small"
        onClick={isRtl ? handleAllLeft : handleAllRight}
        disabled={isRtl ? rightList.length === 0 : leftList.length === 0}
        aria-label="move all right"
      >
        <Iconify width={18} icon="eva:arrowhead-right-fill" />
      </Button>

      <Button
        color="inherit"
        variant="outlined"
        size="small"
        onClick={isRtl ? handleCheckedLeft : handleCheckedRight}
        disabled={isRtl ? rightChecked.length === 0 : leftChecked.length === 0}
        aria-label="move selected right"
      >
        <Iconify width={18} icon="eva:arrow-ios-forward-fill" />
      </Button>

      <Button
        color="inherit"
        variant="outlined"
        size="small"
        onClick={isRtl ? handleCheckedRight : handleCheckedLeft}
        disabled={isRtl ? leftChecked.length === 0 : rightChecked.length === 0}
        aria-label="move selected left"
      >
        <Iconify width={18} icon="eva:arrow-ios-back-fill" />
      </Button>

      <Button
        color="inherit"
        variant="outlined"
        size="small"
        onClick={isRtl ? handleAllRight : handleAllLeft}
        disabled={isRtl ? leftList.length === 0 : rightList.length === 0}
        aria-label="move all left"
      >
        <Iconify width={18} icon="eva:arrowhead-left-fill" />
      </Button>
    </Box>
  );

  return (
    <>
      {renderList(leftList)}
      {renderControls()}
      {renderList(rightList)}
    </>
  );
}
