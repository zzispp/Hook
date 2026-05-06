import { uuidv4 } from 'src/utils/uuidv4';

import { CONFIG } from 'src/global-config';

import { _mock } from './_mock';
import { _tags } from './assets';

// ----------------------------------------------------------------------

const codeBlock = `
<pre class="nml__editor__content__code__block"><code class="language-javascript">for (var i=1; i &lt;= 20; i++) {\n  if (i % 15 == 0)\n    return "FizzBuzz"\n  else if (i % 3 == 0)\n    return "Fizz"\n  else if (i % 5 == 0)\n    return "Buzz"\n  else\n    return i\n  }</code></pre>
`;

// Made with Tiptap editor

const CONTENT = `
<h1 class="nml__editor__content__heading" style="text-align: start">Heading H1</h1>
<h2 class="nml__editor__content__heading" style="text-align: start">Heading H2</h2>
<h3 class="nml__editor__content__heading" style="text-align: start">Heading H3</h3>
<h4 class="nml__editor__content__heading" style="text-align: start">Heading H4</h4>
<h5 class="nml__editor__content__heading" style="text-align: start">Heading H5</h5>
<h6 class="nml__editor__content__heading" style="text-align: start">Heading H6</h6>
<hr class="nml__editor__content__hr">
<h4 class="nml__editor__content__heading" style="text-align: start">Paragraph</h4>
<p style="text-align: start">What is MTAweb Directory?</p>
<p style="text-align: start">So you have heard about this site or you have been to it, but you cannot figure out what it is or what it can do. MTA web directory is the simplest way in which one can bid on a link, or a few links if they wish to do so. The link directory on MTA displays all of the links it currently has, and does so in alphabetical order, which makes it much easier for someone to find what they are looking for if it is something specific and they do not want to go through all the other sites and links as well. It allows you to start your bid at the bottom and slowly work your way to the top of the list.</p>
<p style="text-align: start">With a very low costing starting bid of just $1, you are guaranteed to have a spot in MTA’s successful directory list. When you would like to increase your bid to one of the top positions, you have to know that this would be a wise decision to make as it will not only get your link to be at a higher point in the directory but it will also give you a chance to have your site advertised with the rest of the top ten on the home page of the website. This means that when visitors come to <a target="_blank" rel="noopener noreferrer nofollow" class="nml__editor__content__link" href="http://MTAweb.com">MTAweb.com</a>, your site will be one of the first things they see. In other words, you stand a great chance at getting a comeback to your site sooner than you thought.</p>
<p style="text-align: start"><strong>This is strong text.</strong></p>
<p style="text-align: start"><em>This is italic text</em></p>
<p style="text-align: start">This is underline text</p>
<h4 class="nml__editor__content__heading" style="text-align: start">Unordered list</h4>
<ul class="nml__editor__content__bullet__list">
   <li class="nml__editor__content__listItem">
      <p>Implements <a target="_blank" rel="noopener noreferrer nofollow" class="nml__editor__content__link" href="https://docs-minimals.vercel.app/introduction">This is an external link</a></p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>Implements <a target="_blank" rel="noopener noreferrer nofollow" class="nml__editor__content__link" href="https://codebeautify.org/dashboard/blog">This is an inside link</a></p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>Renders actual, "native" React DOM elements</p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>Allows you to escape or skip HTML (try toggling the checkboxes above)</p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>If you escape or skip the HTML, no dangerouslySetInnerHTML is used! Yay!</p>
   </li>
</ul>
<h4 class="nml__editor__content__heading" style="text-align: start">Ordered list</h4>
<ol class="nml__editor__content__ordered__list">
   <li class="nml__editor__content__listItem">
      <p>Analysis</p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>Design</p>
   </li>
   <li class="nml__editor__content__listItem">
      <p>Implementation</p>
   </li>
</ol>
<h4 class="nml__editor__content__heading" style="text-align: start">Blockquote</h4>
<blockquote class="nml__editor__content__blockquote">
   <p>Life is short, Smile while you still have teeth!&nbsp;</p>
</blockquote>
<h4 class="nml__editor__content__heading" style="text-align: start"><br>Block code</h4>
${codeBlock}
<p>Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.</p>
<h5 class="nml__editor__content__heading" style="text-align: start">Why do we use it?</h5>
<p style="text-align: start">It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like).</p>
<img class="nml__editor__content__image" src="${CONFIG.basePath}/assets/images/cover/cover-5.webp">
<p>It is important that you buy links because the links are what get you the results that you want. The popularity of the links that are listed in the MTA directory is in fact one of the most important factors in the performance of the search engine. Links are important and this is why you have to purchase a link in order to bid on something and the best part is that a link will only cost you $1, which is nothing compared to what you would pay if you decided to do it through any other company or website.</p>
<img class="nml__editor__content__image" src="${CONFIG.basePath}/assets/images/cover/cover-14.webp">
`;

const generateComments = () => {
  const userList = Array.from({ length: 12 }, (_, index) => ({
    id: _mock.id(index),
    name: _mock.fullName(index),
    avatarUrl: _mock.image.avatar(index),
  }));

  return [
    {
      id: uuidv4(),
      name: userList[0].name,
      avatarUrl: userList[0].avatarUrl,
      message: _mock.sentence(1),
      postedAt: _mock.time(1),
      users: [userList[0], userList[1], userList[2]],
      replyComment: [
        {
          id: uuidv4(),
          userId: userList[1].id,
          message: _mock.sentence(2),
          postedAt: _mock.time(2),
        },
        {
          id: uuidv4(),
          userId: userList[0].id,
          message: _mock.sentence(3),
          tagUser: userList[1].name,
          postedAt: _mock.time(3),
        },
        {
          id: uuidv4(),
          userId: userList[2].id,
          message: _mock.sentence(4),
          postedAt: _mock.time(4),
        },
      ],
    },
    {
      id: uuidv4(),
      name: userList[4].name,
      avatarUrl: userList[4].avatarUrl,
      message: _mock.sentence(5),
      postedAt: _mock.time(5),
      users: [userList[5], userList[6], userList[7]],
      replyComment: [
        {
          id: uuidv4(),
          userId: userList[5].id,
          message: _mock.sentence(6),
          postedAt: _mock.time(6),
        },
        {
          id: uuidv4(),
          userId: userList[6].id,
          message: _mock.sentence(7),
          postedAt: _mock.time(7),
        },
        {
          id: uuidv4(),
          userId: userList[7].id,
          message: _mock.sentence(8),
          postedAt: _mock.time(8),
        },
      ],
    },
    {
      id: uuidv4(),
      name: userList[8].name,
      avatarUrl: userList[8].avatarUrl,
      message: _mock.sentence(9),
      postedAt: _mock.time(9),
      users: [],
      replyComment: [],
    },
    {
      id: uuidv4(),
      name: userList[9].name,
      avatarUrl: userList[9].avatarUrl,
      message: _mock.sentence(10),
      postedAt: _mock.time(10),
      users: [],
      replyComment: [],
    },
  ];
};

export const _posts = () =>
  Array.from({ length: 19 }, (_, index) => {
    const comments = generateComments();

    const publish = index % 3 ? 'published' : 'draft';

    const metaKeywords = _tags.slice(8, 11);

    return {
      id: _mock.id(index),
      publish,
      comments,
      metaKeywords,
      content: CONTENT,
      tags: _tags.slice(0, 5),
      metaTitle: 'Minimal UI Kit',
      createdAt: _mock.time(index),
      title: _mock.postTitle(index),
      coverUrl: _mock.image.cover(index),
      totalViews: _mock.number.nativeL(index),
      totalShares: _mock.number.nativeL(index + 2),
      totalComments: _mock.number.nativeL(index + 1),
      totalFavorites: _mock.number.nativeL(index + 3),
      metaDescription: 'The starting point for your next project with Minimal UI Kit',
      description: `Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. `,
      author: {
        name: _mock.fullName(index),
        avatarUrl: _mock.image.avatar(index),
      },
      favoritePerson: Array.from({ length: 20 }, (__, personIndex) => ({
        name: _mock.fullName(personIndex),
        avatarUrl: _mock.image.avatar(personIndex),
      })),
    };
  });
