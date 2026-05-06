'use client';

import { _mock } from 'src/_mock';

import { Markdown } from 'src/components/markdown';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const IMG_PATH = _mock.image.cover(18);

const htmlContent = `
<h1>h1</h1>
<h2>h2</h2>
<p> <strong>Paragraph</strong> Lorem ipsum is placeholder text commonly used in the graphic, print, and publishing industries for previewing layouts and visual mockups</p>
<p>
  <a href='https://www.google.com/'>Link (https://www.google.com/)</a>
</p>

<h6>List</h6>
<ul>
  <li>
    <input type="checkbox" disabled="" checked=""> Write the press release
  </li>
  <li>
    <input type="checkbox" disabled=""> Update the website
  </li>
  <li>
    <input type="checkbox" disabled=""> Contact the media
  </li>
</ul>

<hr/>

<h6>A table:</h6>

<table>
  <thead>
    <tr>
      <th style="text-align: left;">Syntax</th>
      <th style="text-align: center;">Description</th>
      <th style="text-align: right;">Notes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="text-align: left;">Header</td>
      <td style="text-align: center;">Title</td>
      <td style="text-align: right;">Here's this</td>
    </tr>
    <tr>
      <td style="text-align: left;">Paragraph</td>
      <td style="text-align: center;">Text</td>
      <td style="text-align: right;">And more</td>
    </tr>
  </tbody>
</table>

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

<code>Code inline</code>

<p><img alt='cover' src=${IMG_PATH}></p>

<blockquote> <p>A block quote with <s>strikethrough</s> and a URL: <a href='https://reactjs.org'>https://reactjs.org</a>.</p> </blockquote>
`;

const mardownContent = `
# h1

## h2

**Paragraph** Lorem ipsum is placeholder text commonly used in the graphic, print, and publishing industries for previewing layouts and visual mockups.

[Link (https://www.google.com/)](https://www.google.com/)

###### List
- [x] Write the press release
- [ ] Update the website
- [ ] Contact the media

---

###### A table:

| Syntax        | Description     | Notes         |
| :---          | :----:          | ---:          |
| Header        | Title           | Here's this   |
| Paragraph     | Text            | And more      |

\`\`\`tsx
for (var i=1; i &#x3C;= 20; i++) {
  if (i % 15 == 0)
    return "FizzBuzz"
  else if (i % 3 == 0)
    return "Fizz"
  else if (i % 5 == 0)
    return "Buzz"
  else
    return i
  }
\`\`\`

\`Code inline\`

![cover](${IMG_PATH})

> A block quote with ~~strikethrough~~ and a URL: [https://reactjs.org](https://reactjs.org).
`;

// ----------------------------------------------------------------------

export function MarkdownView() {
  return (
    <ComponentLayout
      heroProps={{
        heading: 'Markdown',
        moreLinks: ['https://www.npmjs.com/package/react-markdown'],
      }}
      containerProps={{
        maxWidth: 'lg',
        sx: {
          rowGap: 5,
          columnGap: 3,
          display: 'grid',
          gridTemplateColumns: { xs: 'repeat(1, 1fr)', md: 'repeat(2, 1fr)' },
        },
      }}
    >
      <ComponentBox title="Html content" sx={{ py: 0 }}>
        <Markdown children={htmlContent} />
      </ComponentBox>

      <ComponentBox title="Mardown content" sx={{ py: 0 }}>
        <Markdown children={mardownContent} />
      </ComponentBox>
    </ComponentLayout>
  );
}
