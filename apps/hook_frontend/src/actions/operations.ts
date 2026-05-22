'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  Announcement,
  TicketStatus,
  SupportTicket,
  OperationsPage,
  TicketPriority,
  NotificationItem,
  AnnouncementInput,
  SupportTicketDetail,
  SupportTicketMutationResponse,
} from 'src/types/operations';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

const NOTIFICATION_REFRESH_INTERVAL_MS = 30_000;

const notificationSWRKeyOptions = {
  ...swrOptions,
  revalidateOnFocus: true,
  revalidateOnReconnect: true,
  refreshWhenHidden: false,
  refreshInterval: () => (documentIsVisible() ? NOTIFICATION_REFRESH_INTERVAL_MS : 0),
};

export type AnnouncementFilters = {
  search?: string;
  announcement_type?: string;
  enabled?: boolean;
};

export type TicketFilters = {
  search?: string;
  status?: TicketStatus | '';
  priority?: TicketPriority | '';
};

export function useAnnouncements(
  page: number,
  pageSize: number,
  filters: AnnouncementFilters = {},
  admin = false
) {
  const endpoint = admin ? endpoints.adminAnnouncements.list : endpoints.announcements.list;
  return usePagedResource<Announcement>(endpoint, page, pageSize, filters);
}

export function useAnnouncement(id: string, admin = false) {
  const endpoint = admin ? endpoints.adminAnnouncements.byId(id) : endpoints.announcements.byId(id);
  return useSingleResource<Announcement>(id ? endpoint : null);
}

export async function createAnnouncement(payload: AnnouncementInput) {
  const value = await requestData<Announcement>(axios.post(endpoints.adminAnnouncements.list, payload));
  await refreshAnnouncements();
  return value;
}

export async function updateAnnouncement(id: string, payload: Partial<AnnouncementInput>) {
  const value = await requestData<Announcement>(axios.patch(endpoints.adminAnnouncements.byId(id), payload));
  await refreshAnnouncements();
  await mutate(endpoints.adminAnnouncements.byId(id));
  return value;
}

export async function deleteAnnouncement(id: string) {
  await requestSuccess(axios.delete(endpoints.adminAnnouncements.byId(id)));
  await refreshAnnouncements();
}

export function useTickets(page: number, pageSize: number, filters: TicketFilters = {}, admin = false) {
  const endpoint = admin ? endpoints.adminTickets.list : endpoints.tickets.list;
  return usePagedResource<SupportTicket>(endpoint, page, pageSize, cleanTicketFilters(filters));
}

export function useTicketDetail(id: string, admin = false) {
  const endpoint = admin ? endpoints.adminTickets.byId(id) : endpoints.tickets.byId(id);
  return useSingleResource<SupportTicketDetail>(id ? endpoint : null);
}

export async function createTicket(payload: {
  subject: string;
  body_markdown: string;
  contact_email?: string;
  captcha_token?: string;
}) {
  const value = await requestData<SupportTicketMutationResponse>(axios.post(endpoints.tickets.list, payload));
  await refreshTickets();
  return value;
}

export async function replyTicket(id: string, body_markdown: string, admin = false) {
  const endpoint = admin ? endpoints.adminTickets.messages(id) : endpoints.tickets.messages(id);
  const value = await requestData<SupportTicketMutationResponse>(axios.patch(endpoint, { body_markdown }));
  await refreshTickets();
  await refreshTicketDetail(id, admin);
  return value;
}

export async function updateTicket(id: string, payload: { status?: TicketStatus; priority?: TicketPriority }) {
  const value = await requestData<SupportTicketMutationResponse>(axios.patch(endpoints.adminTickets.byId(id), payload));
  await refreshTickets();
  await refreshTicketDetail(id, true);
  return value;
}

export function useNotifications(status?: 'read' | 'unread') {
  return usePagedResource<NotificationItem>(
    endpoints.notifications.list,
    0,
    30,
    { status },
    notificationSWRKeyOptions
  );
}

export async function markAllNotificationsRead() {
  await requestSuccess(axios.patch(endpoints.notifications.readAll));
  await refreshNotifications();
}

export async function markNotificationRead(item: NotificationItem) {
  await requestSuccess(axios.patch(endpoints.notifications.read(item.source_type, item.source_id)));
  await refreshNotifications();
}

export async function deleteNotification(item: NotificationItem) {
  await requestSuccess(axios.delete(endpoints.notifications.delete(item.source_type, item.source_id)));
  await refreshNotifications();
}

function usePagedResource<T>(
  endpoint: string,
  page: number,
  pageSize: number,
  filters = {},
  options = swrOptions
) {
  const disabled = page < 0 || pageSize <= 0;
  const key = disabled ? null : [endpoint, { params: { ...pageQuery(page, pageSize), ...filters } }];
  const {
    data,
    isLoading,
    error,
    isValidating,
    mutate: revalidate,
  } = useSWR<ApiEnvelope<OperationsPage<T>>>(key, fetcher, options);

  return useMemo(() => {
    const pageData = data?.success ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading: disabled ? false : isLoading,
      error: error ?? apiError(data),
      isValidating: disabled ? false : isValidating,
      refresh: revalidate,
    };
  }, [data, disabled, error, isLoading, isValidating, revalidate]);
}

function useSingleResource<T>(endpoint: string | null) {
  const { data, isLoading, error, isValidating, mutate: refresh } = useSWR<ApiEnvelope<T>>(endpoint, fetcher, swrOptions);
  return useMemo(
    () => ({
      data: data?.success ? requireApiData(data) : undefined,
      isLoading: endpoint ? isLoading : false,
      error: error ?? apiError(data),
      isValidating: endpoint ? isValidating : false,
      refresh,
    }),
    [data, endpoint, error, isLoading, isValidating, refresh]
  );
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function requestSuccess<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  if (!response.data.success) {
    throw new Error(response.data.message || 'Request failed');
  }
}

async function refreshAnnouncements() {
  await mutate((key) => isEndpointKey(key, endpoints.announcements.list));
  await mutate((key) => isEndpointKey(key, endpoints.adminAnnouncements.list));
}

async function refreshTickets() {
  await mutate((key) => isEndpointKey(key, endpoints.tickets.list));
  await mutate((key) => isEndpointKey(key, endpoints.adminTickets.list));
  await refreshNotifications();
}

async function refreshTicketDetail(id: string, admin: boolean) {
  await mutate(admin ? endpoints.adminTickets.byId(id) : endpoints.tickets.byId(id));
}

async function refreshNotifications() {
  await mutate((key) => isEndpointKey(key, endpoints.notifications.list));
}

function cleanTicketFilters(filters: TicketFilters) {
  return {
    ...filters,
    status: filters.status || undefined,
    priority: filters.priority || undefined,
  };
}

function apiError<T>(payload?: ApiEnvelope<T>) {
  return payload && !payload.success ? new Error(payload.message || 'Request failed') : undefined;
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function documentIsVisible() {
  return typeof document !== 'undefined' && document.visibilityState === 'visible';
}
