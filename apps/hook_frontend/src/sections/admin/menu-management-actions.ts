'use client';

import type { MenuDeleteState, MenuManagementForms } from './menu-management-state';

import { useState, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { deleteMenuItem, updateMenuApis, deleteMenuSection } from 'src/actions/rbac';

import { toast } from 'src/components/snackbar';

import { saveMenuItem, saveMenuSection } from './menu-management-state';

type ActionOptions = {
  formState: MenuManagementForms;
  deleteState: MenuDeleteState;
};

export function useMenuManagementActions({ formState, deleteState }: ActionOptions) {
  const [submitting, setSubmitting] = useState(false);
  const section = useSectionActions({ formState, setSubmitting });
  const item = useItemActions({ deleteState, formState, setSubmitting });
  const apiBinding = useApiBindingActions({ formState, setSubmitting });
  const deleteSection = useDeleteSectionAction({ deleteState });

  return { ...section, ...item, ...apiBinding, ...deleteSection, submitting };
}

function useSectionActions({
  formState,
  setSubmitting,
}: Pick<ActionOptions, 'formState'> & SubmitState) {
  const { t } = useTranslate('admin');

  const submitSection = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveMenuSection(formState.section.editing, formState.section.form);
      toast.success(t(formState.section.editing ? 'messages.menuSectionUpdated' : 'messages.menuSectionCreated'));
      formState.section.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [formState.section, setSubmitting, t]);

  return { submitSection };
}

function useItemActions({
  deleteState,
  formState,
  setSubmitting,
}: ActionOptions & SubmitState) {
  const { t } = useTranslate('admin');

  const submitItem = useCallback(async () => {
    setSubmitting(true);
    try {
      await saveMenuItem(formState.item.editing, formState.item.form);
      toast.success(t(formState.item.editing ? 'messages.menuItemUpdated' : 'messages.menuItemCreated'));
      formState.item.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [formState.item, setSubmitting, t]);

  const confirmDeleteItem = useCallback(async () => {
    if (!deleteState.itemTarget) return;
    try {
      await deleteMenuItem(deleteState.itemTarget.id);
      toast.success(t('messages.menuItemDeleted'));
      deleteState.setItemTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteState, t]);

  return { confirmDeleteItem, submitItem };
}

function useApiBindingActions({
  formState,
  setSubmitting,
}: Pick<ActionOptions, 'formState'> & SubmitState) {
  const { t } = useTranslate('admin');

  const saveApiBindings = useCallback(async () => {
    if (!formState.apiBinding.target) return;
    setSubmitting(true);
    try {
      await updateMenuApis(formState.apiBinding.target.id, formState.apiBinding.selectedApiIds);
      toast.success(t('messages.menuApiPermissionsUpdated'));
      formState.apiBinding.close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [formState.apiBinding, setSubmitting, t]);

  return { saveApiBindings };
}

function useDeleteSectionAction({ deleteState }: Pick<ActionOptions, 'deleteState'>) {
  const { t } = useTranslate('admin');

  const confirmDeleteSection = useCallback(async () => {
    if (!deleteState.sectionTarget) return;
    try {
      await deleteMenuSection(deleteState.sectionTarget.id);
      toast.success(t('messages.menuSectionDeleted'));
      deleteState.setSectionTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteState, t]);

  return { confirmDeleteSection };
}

type SubmitState = {
  setSubmitting: (value: boolean) => void;
};
