'use client';

import { useState, useCallback } from 'react';

import { paths } from 'src/routes/paths';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useCaptchaConfig } from 'src/actions/captcha';
import { useTranslate } from 'src/locales/use-locales';
import {
  useTickets,
  replyTicket,
  createTicket,
  updateTicket,
  useTicketDetail,
} from 'src/actions/operations';

import { toast } from 'src/components/snackbar';
import { useTable } from 'src/components/table';

import { useAuthContext } from 'src/auth/hooks';

type TicketEmailDelivery = Awaited<ReturnType<typeof replyTicket>>['email_delivery'];

const EMAIL_DELIVERY_ERROR_KEYS: Record<string, string> = {
  email_configuration_disabled: 'operations.ticket.emailErrors.emailConfigurationDisabled',
  smtp_configuration_incomplete: 'operations.ticket.emailErrors.smtpConfigurationIncomplete',
};

export function useTicketWorkspaceState(admin: boolean) {
  const { t } = useTranslate('admin');
  const { user } = useAuthContext();
  const selection = useTicketSelection(admin);
  const table = useTable({ defaultRowsPerPage: 10 });
  const [search, setSearch] = useState('');
  const [creating, setCreating] = useState(false);
  const captcha = useCaptchaConfig();
  const tickets = useTickets(table.page, table.rowsPerPage, { search }, admin);
  const detail = useTicketDetail(selection.selectedId, admin);
  const mutations = useTicketMutations({ admin, t, setCreating, ...selection });
  const refreshTickets = useCallback(async () => {
    await tickets.refresh();
    if (selection.selectedId) {
      await detail.refresh();
    }
  }, [detail, selection.selectedId, tickets]);
  const handleSearch = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      table.onResetPage();
      setSearch(event.target.value);
    },
    [table]
  );

  return {
    t,
    table,
    search,
    tickets,
    detail,
    creating,
    userEmail: user?.email ?? '',
    ticketCaptchaConfig: ticketCaptchaConfig(captcha),
    handleSearch,
    refreshTickets,
    setCreating,
    ...selection,
    ...mutations,
  };
}

function ticketCaptchaConfig(captcha: ReturnType<typeof useCaptchaConfig>) {
  return {
    enabled: captcha.data?.support_ticket_captcha_enabled,
    loading: captcha.isLoading,
    errorMessage: captcha.error?.message,
  };
}

function useTicketSelection(admin: boolean) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const selectedId = searchParams.get('id') || '';
  const basePath = admin ? paths.dashboard.admin.tickets : paths.dashboard.tickets;

  const selectTicket = useCallback(
    (id: string) => router.push(`${basePath}?id=${id}`),
    [basePath, router]
  );

  return { selectedId, selectTicket };
}

function useTicketMutations({
  t,
  admin,
  selectedId,
  selectTicket,
  setCreating,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  admin: boolean;
  selectedId: string;
  selectTicket: (id: string) => void;
  setCreating: (value: boolean) => void;
}) {
  const [submitting, setSubmitting] = useState(false);
  const onDelivery = useTicketDeliveryNotifier(t);
  const submitCreate = useTicketCreateAction({ t, setSubmitting, setCreating, selectTicket });
  const submitReply = useTicketReplyAction({ t, admin, selectedId, setSubmitting, onDelivery });
  const submitUpdate = useTicketUpdateAction({ t, selectedId, setSubmitting, onDelivery });

  return { submitting, submitCreate, submitReply, submitUpdate };
}

function useTicketDeliveryNotifier(t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(
    (result: Awaited<ReturnType<typeof replyTicket>>) => {
      notifyEmailDelivery(result.email_delivery, t);
    },
    [t]
  );
}

function useTicketCreateAction({
  t,
  setSubmitting,
  setCreating,
  selectTicket,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  setSubmitting: (value: boolean) => void;
  setCreating: (value: boolean) => void;
  selectTicket: (id: string) => void;
}) {
  return useCallback(
    async (form: Parameters<typeof createTicket>[0]) => {
      await runTicketMutation(
        async () => {
          const result = await createTicket(form);
          notifyEmailDelivery(result.email_delivery, t);
          setCreating(false);
          selectTicket(result.ticket.id);
        },
        setSubmitting,
        t
      );
    },
    [selectTicket, setCreating, setSubmitting, t]
  );
}

function useTicketReplyAction({
  t,
  admin,
  selectedId,
  setSubmitting,
  onDelivery,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  admin: boolean;
  selectedId: string;
  setSubmitting: (value: boolean) => void;
  onDelivery: (result: Awaited<ReturnType<typeof replyTicket>>) => void;
}) {
  return useCallback(
    async (message: string) => {
      if (!selectedId) return;
      await runTicketMutation(
        async () => onDelivery(await replyTicket(selectedId, message, admin)),
        setSubmitting,
        t
      );
    },
    [admin, onDelivery, selectedId, setSubmitting, t]
  );
}

function useTicketUpdateAction({
  t,
  selectedId,
  setSubmitting,
  onDelivery,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  selectedId: string;
  setSubmitting: (value: boolean) => void;
  onDelivery: (result: Awaited<ReturnType<typeof replyTicket>>) => void;
}) {
  return useCallback(
    async (patch: Parameters<typeof updateTicket>[1]) => {
      if (!selectedId) return;
      await runTicketMutation(
        async () => onDelivery(await updateTicket(selectedId, patch)),
        setSubmitting,
        t
      );
    },
    [onDelivery, selectedId, setSubmitting, t]
  );
}

async function runTicketMutation(
  action: () => Promise<void>,
  setSubmitting: (value: boolean) => void,
  t: ReturnType<typeof useTranslate>['t']
) {
  setSubmitting(true);
  try {
    await action();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  } finally {
    setSubmitting(false);
  }
}

function notifyEmailDelivery(
  delivery: TicketEmailDelivery,
  t: ReturnType<typeof useTranslate>['t']
) {
  if (delivery.status === 'failed') {
    toast.error(localizedEmailDeliveryError(delivery, t));
    return;
  }
  toast.success(t('operations.messages.saved'));
}

function localizedEmailDeliveryError(
  delivery: TicketEmailDelivery,
  t: ReturnType<typeof useTranslate>['t']
) {
  const translationKey = delivery.error_code ? EMAIL_DELIVERY_ERROR_KEYS[delivery.error_code] : '';
  if (translationKey) {
    return t(translationKey);
  }
  return delivery.error_message || t('operations.ticket.emailFailed');
}
