'use client';

import type { ScheduledTask } from 'src/types/scheduler';
import type {
  Translate,
  TaskFormState,
  RunStatusFilter,
  ScheduledTaskTab,
} from './scheduled-tasks-utils';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useScheduledTasks, updateScheduledTask, useScheduledTaskRuns } from 'src/actions/scheduler';

import { toast } from 'src/components/snackbar';
import {
  useTable,
} from 'src/components/table';

import { ScheduledTaskEditor } from './scheduled-task-edit-dialog';
import {
  RefreshButton,
  AdminBreadcrumbs,
  ManagementDialog,
} from './shared';
import { ScheduledTaskTable, ScheduledTaskRunTable } from './scheduled-tasks-table';
import {
  AdminFiltersToolbar,
  DEFAULT_ADMIN_FILTERS,
} from './admin-filters-toolbar';
import {
  taskLabel,
  filterTasks,
  taskPayload,
  formFromTask,
  toRunFilters,
  pagedTaskRows,
} from './scheduled-tasks-utils';

export function ScheduledTasksView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<ScheduledTaskTab>('tasks');
  const [taskFilters, setTaskFilters] = useState(DEFAULT_ADMIN_FILTERS);
  const [selectedTaskCode, setSelectedTaskCode] = useState('');
  const [runStatus, setRunStatus] = useState<RunStatusFilter>('');
  const [editTask, setEditTask] = useState<ScheduledTask | null>(null);
  const [taskForm, setTaskForm] = useState<TaskFormState | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const taskTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'code' });
  const runTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'started_at' });
  const tasks = useScheduledTasks();
  const runs = useScheduledTaskRuns(
    runTable.page,
    runTable.rowsPerPage,
    toRunFilters(selectedTaskCode, runStatus)
  );
  const locale = currentLang.numberFormat.code;
  const filteredTasks = useMemo(
    () => filterTasks(tasks.items, taskFilters, t as Translate),
    [taskFilters, tasks.items, t]
  );

  useEffect(() => {
    if (!editTask) {
      setTaskForm(null);
      return;
    }
    setTaskForm(formFromTask(editTask));
  }, [editTask]);

  const handleTaskFiltersChange = useCallback((nextFilters: typeof DEFAULT_ADMIN_FILTERS) => {
    taskTable.onResetPage();
    setTaskFilters(nextFilters);
  }, [taskTable]);

  const handleTaskStatusToggle = useCallback(async (task: ScheduledTask, enabled: boolean) => {
    try {
      await updateScheduledTask(task.code, { enabled });
      toast.success(
        t(enabled ? 'scheduledTasks.messages.taskEnabled' : 'scheduledTasks.messages.taskDisabled')
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [t]);

  const handleSubmitTask = useCallback(async () => {
    if (!editTask || !taskForm) {
      return;
    }
    setSubmitting(true);
    try {
      await updateScheduledTask(editTask.code, taskPayload(editTask, taskForm));
      toast.success(t('scheduledTasks.messages.taskUpdated'));
      setEditTask(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [editTask, taskForm, t]);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.scheduledTaskManagement}
        action={<RefreshButton loading={tab === 'tasks' ? tasks.isLoading : runs.isLoading} onClick={() => void (tab === 'tasks' ? tasks.refresh() : runs.refresh())} />}
      />

      <Card>
        <Tabs value={tab} onChange={(_event, value: ScheduledTaskTab) => setTab(value)} sx={{ px: 2.5 }}>
          <Tab value="tasks" label={t('scheduledTasks.tabs.tasks')} />
          <Tab value="runs" label={t('scheduledTasks.tabs.runs')} />
        </Tabs>

        {tab === 'tasks' ? (
          <>
            <AdminFiltersToolbar
              filters={taskFilters}
              searchPlaceholder={t('scheduledTasks.filters.searchTasks')}
              onChange={handleTaskFiltersChange}
            />
            <ScheduledTaskTable
              loading={tasks.isLoading}
              rows={pagedTaskRows(filteredTasks, taskTable.page, taskTable.rowsPerPage)}
              total={filteredTasks.length}
              locale={locale}
              table={taskTable}
              onEdit={setEditTask}
              onToggle={handleTaskStatusToggle}
              t={t}
            />
          </>
        ) : (
          <>
            <RunFiltersBar
              status={runStatus}
              taskCode={selectedTaskCode}
              tasks={tasks.items}
              t={t as Translate}
              onStatusChange={(value) => {
                runTable.onResetPage();
                setRunStatus(value);
              }}
              onTaskCodeChange={(value) => {
                runTable.onResetPage();
                setSelectedTaskCode(value);
              }}
            />
            <ScheduledTaskRunTable
              loading={runs.isLoading}
              rows={runs.items}
              total={runs.total}
              table={runTable}
              locale={locale}
              tasks={tasks.items}
              t={t}
            />
          </>
        )}
      </Card>

      <ManagementDialog
        open={Boolean(editTask && taskForm)}
        title={editTask ? taskLabel(t, editTask) : ''}
        description={editTask ? t(editTask.description_key) : undefined}
        submitting={submitting}
        onClose={() => setEditTask(null)}
        onSubmit={() => void handleSubmitTask()}
      >
        {editTask && taskForm ? <ScheduledTaskEditor form={taskForm} task={editTask} t={t as Translate} onChange={setTaskForm} /> : null}
      </ManagementDialog>
    </DashboardContent>
  );
}

function RunFiltersBar({
  status,
  taskCode,
  tasks,
  t,
  onStatusChange,
  onTaskCodeChange,
}: {
  status: RunStatusFilter;
  taskCode: string;
  tasks: ScheduledTask[];
  t: Translate;
  onStatusChange: (value: RunStatusFilter) => void;
  onTaskCodeChange: (value: string) => void;
}) {
  return (
    <Box
      sx={{
        p: 2.5,
        gap: 2,
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', md: '220px 220px' },
      }}
    >
      <TextField
        select
        label={t('common.status')}
        value={status}
        onChange={(event) => onStatusChange(event.target.value as RunStatusFilter)}
      >
        <MenuItem value="">{t('filters.allStatuses')}</MenuItem>
        <MenuItem value="running">{t('scheduledTasks.status.running')}</MenuItem>
        <MenuItem value="succeeded">{t('scheduledTasks.status.succeeded')}</MenuItem>
        <MenuItem value="failed">{t('scheduledTasks.status.failed')}</MenuItem>
        <MenuItem value="skipped_running">{t('scheduledTasks.status.skipped_running')}</MenuItem>
      </TextField>
      <TextField
        select
        label={t('scheduledTasks.fields.task')}
        value={taskCode}
        onChange={(event) => onTaskCodeChange(event.target.value)}
      >
        <MenuItem value="">{t('scheduledTasks.filters.allTasks')}</MenuItem>
        {tasks.map((task) => (
          <MenuItem key={task.code} value={task.code}>
            {taskLabel(t, task)}
          </MenuItem>
        ))}
      </TextField>
    </Box>
  );
}
