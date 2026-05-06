'use client';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Editor } from 'src/components/editor';
import { Markdown } from 'src/components/markdown';

import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const codeBlock = `
<pre><code class="language-javascript">for (var i=1; i &#x3C;= 20; i++) {
  if (i % 15 == 0)
    return "FizzBuzz"
  else if (i % 3 == 0)
    return "Fizz"
  else if (i % 5 == 0)
    return "Buzz"
  else
    return i
  }</code></pre>
`;

const defaultValue = `
<h4>This is Heading 4</h4>
<p>
  <strong>Lorem Ipsum</strong> is simply <em>dummy</em> text of the <u>printing</u> and <s>typesetting</s> industry. Lorem Ipsum has been the <a target="_blank" rel="noopener noreferrer nofollow" class="minimal__editor__content__link" href="https://www.google.com/">industry's</a> standard dummy text ever since the 1500s, when an <strong>
    <span style="text-transform: uppercase;">unknown</span>
  </strong> printer took a <span style="text-transform: capitalize;">galley</span> of type and scrambled it to make a type specimen book.
</p>
<code>This is code</code>
${codeBlock}
`;

// ----------------------------------------------------------------------

export function EditorView() {
  const [checked, setChecked] = useState(true);
  const [content, setContent] = useState(defaultValue);

  const handleChange = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setChecked(event.target.checked);
  }, []);

  return (
    <ComponentLayout
      heroProps={{
        heading: 'Editor',
        moreLinks: ['https://tiptap.dev/docs/editor/introduction'],
      }}
      containerProps={{ maxWidth: false }}
    >
      <FormControlLabel
        control={<Switch name="fullItem" checked={checked} onChange={handleChange} />}
        label="Full item"
        sx={{ mb: 3 }}
      />

      <Box
        sx={{
          rowGap: 5,
          columnGap: 3,
          display: 'grid',
          alignItems: 'flex-start',
          gridTemplateColumns: { xs: 'repeat(1, 1fr)', lg: 'repeat(2, 1fr)' },
        }}
      >
        <Editor
          fullItem={checked}
          value={content}
          onChange={(value) => setContent(value)}
          sx={{ maxHeight: 720 }}
        />

        <Paper variant="outlined" sx={{ borderRadius: 2, bgcolor: 'background.neutral' }}>
          <Box
            sx={[
              (theme) => ({
                px: 3,
                py: 3.75,
                typography: 'h6',
                bgcolor: 'background.paper',
                borderTopLeftRadius: 'inherit',
                borderTopRightRadius: 'inherit',
                borderBottom: `1px solid ${theme.palette.divider}`,
              }),
            ]}
          >
            Preview
          </Box>
          <Markdown children={content} sx={{ px: 3 }} />
        </Paper>
      </Box>
    </ComponentLayout>
  );
}
