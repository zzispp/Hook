import { uuidv4 } from 'src/utils/uuidv4';
import { fAdd, fSub } from 'src/utils/set-date';

import { _mock } from './_mock';
import { _tags } from './assets';

// ----------------------------------------------------------------------

const generateAttachments = () =>
  Array.from({ length: 20 }, (_, index) => _mock.image.cover(index));

const generateAssignees = () =>
  Array.from({ length: 20 }, (_, index) => ({
    id: _mock.id(index),
    name: _mock.fullName(index),
    avatarUrl: _mock.image.avatar(index),
  }));

const generateComments = () =>
  Array.from({ length: 8 }, (_, index) => ({
    id: uuidv4(),
    name: _mock.fullName(index),
    avatarUrl: _mock.image.avatar(index),
    createdAt: fSub({ minutes: 20 - index }),
    messageType: [1, 2].includes(index) ? 'image' : 'text',
    message: [1, 2].includes(index) ? _mock.image.cover(index + 5) : _mock.sentence(index),
  }));

const COLUMN_NAMES = {
  name1: 'To do',
  name2: 'In progress',
  name3: 'Ready to test',
  name4: 'Done',
};

const COLUMN_IDS = {
  id1: `${1}-column-${_mock.id(1)}`,
  id2: `${2}-column-${_mock.id(2)}`,
  id3: `${3}-column-${_mock.id(3)}`,
  id4: `${4}-column-${_mock.id(4)}`,
};

const PRIORITY_LEVEL = {
  low: 'low',
  medium: 'medium',
  high: 'high',
};

// ----------------------------------------------------------------------

const createTask = (index: number, status: string) => {
  const commentList = generateComments();
  const assignedUser = generateAssignees();
  const attachmentList = generateAttachments();

  const reporter = {
    id: _mock.id(16),
    name: _mock.fullName(16),
    avatarUrl: _mock.image.avatar(16),
  };

  return {
    id: `${index}-task-${_mock.id(index)}`,
    reporter,
    name: _mock.taskNames(index),
    labels: _tags.slice(0, index),
    comments: commentList.slice(0, index),
    assignee: assignedUser.slice(0, index),
    description: _mock.description(index),
    due: [fAdd({ days: index + 1 }), fAdd({ days: index + 2 })],
    priority:
      ([1, 3].includes(index) && PRIORITY_LEVEL.high) ||
      ([2, 4].includes(index) && PRIORITY_LEVEL.medium) ||
      PRIORITY_LEVEL.low,
    attachments:
      (index === 1 && attachmentList.slice(11, 15)) ||
      (index === 5 && attachmentList.slice(4, 9)) ||
      [],
    status,
  };
};

const tasks = () => ({
  [COLUMN_IDS.id1]: [
    createTask(1, COLUMN_NAMES.name1),
    createTask(2, COLUMN_NAMES.name1),
    createTask(3, COLUMN_NAMES.name1),
  ],
  [COLUMN_IDS.id2]: [createTask(4, COLUMN_NAMES.name2), createTask(5, COLUMN_NAMES.name2)],
  [COLUMN_IDS.id3]: [],
  [COLUMN_IDS.id4]: [createTask(6, COLUMN_NAMES.name4)],
});

const columns = () => [
  { id: COLUMN_IDS.id1, name: COLUMN_NAMES.name1 },
  { id: COLUMN_IDS.id2, name: COLUMN_NAMES.name2 },
  { id: COLUMN_IDS.id3, name: COLUMN_NAMES.name3 },
  { id: COLUMN_IDS.id4, name: COLUMN_NAMES.name4 },
];

export const _board = () => ({
  tasks: tasks(),
  columns: columns(),
});
