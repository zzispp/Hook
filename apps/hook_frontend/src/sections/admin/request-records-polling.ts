import type { RefObject } from 'react';
import type { RequestRecord } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import { isRequestCancelled } from 'src/lib/axios';
import { fetchActiveRequestRecords } from 'src/actions/request-records';

import {
  shouldPollRequestRecord,
  requestRecordMatchesStatusFilter,
  REQUEST_RECORD_ALL_STATUS_FILTER,
} from './request-records-utils';

const ACTIVE_REQUEST_REFRESH_INTERVAL_MS = 3000;
const EMPTY_REQUEST_IDS: string[] = [];
const REQUEST_STATUS_RANK: Record<string, number> = {
  pending: 0,
  streaming: 1,
  success: 2,
  failed: 2,
  cancelled: 2,
};

export function useAutoRefresh(enabled: boolean, refresh: () => void, intervalMs: number) {
  useEffect(() => {
    if (!enabled) return undefined;
    refresh();
    const timer = window.setInterval(refresh, intervalMs);
    return () => window.clearInterval(timer);
  }, [enabled, intervalMs, refresh]);
}

export function usePageVisible() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const update = () => setVisible(!document.hidden);
    update();
    document.addEventListener('visibilitychange', update);
    return () => document.removeEventListener('visibilitychange', update);
  }, []);

  return visible;
}

export function usePollingRequestIds(items: RequestRecord[], enabled: boolean) {
  const activeRequestIds = useMemo(() => activeRecordIds(items), [items]);
  return enabled ? activeRequestIds : EMPTY_REQUEST_IDS;
}

export function useRestoreScrollOnSelection(
  selectedRecord: RequestRecord | null,
  scrollSnapshotRef: RefObject<number | null>
) {
  useEffect(() => {
    const scrollY = scrollSnapshotRef.current;
    if (!selectedRecord || scrollY === null) return undefined;
    const frame = window.requestAnimationFrame(() => {
      window.scrollTo({ top: scrollY, left: window.scrollX, behavior: 'instant' });
      scrollSnapshotRef.current = null;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [scrollSnapshotRef, selectedRecord]);
}

export function useActiveRequestPolling(
  ids: string[],
  statusFilter: string,
  updateItems: (updater: (items: RequestRecord[]) => RequestRecord[]) => void,
  refresh: () => void
) {
  const idsKey = ids.join('\n');

  useEffect(() => {
    if (!ids.length) return undefined;
    let active = true;
    let controller: AbortController | null = null;
    const poll = async () => {
      if (controller) return;
      const nextController = new AbortController();
      controller = nextController;
      try {
        const response = await fetchActiveRequestRecords(ids, nextController.signal);
        if (!active || nextController.signal.aborted) return;
        updateItems((items) => mergedVisibleRecords(items, response.records, statusFilter));
        if (shouldRefreshRecords(ids, response.records, statusFilter)) refresh();
      } catch (error) {
        if (!isRequestCancelled(error)) {
          console.error('Failed to poll active request records:', error);
        }
      } finally {
        if (controller === nextController) {
          controller = null;
        }
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), ACTIVE_REQUEST_REFRESH_INTERVAL_MS);
    return () => {
      active = false;
      window.clearInterval(timer);
      controller?.abort();
    };
  }, [ids, idsKey, refresh, statusFilter, updateItems]);
}

function activeRecordIds(items: RequestRecord[]) {
  return items
    .filter((record) => shouldPollRequestRecord(record))
    .map((record) => record.request_id);
}

function mergedVisibleRecords(
  items: RequestRecord[],
  updates: RequestRecord[],
  statusFilter: string
) {
  return mergeRequestRecords(items, updates).filter((record) =>
    requestRecordMatchesStatusFilter(record, statusFilter)
  );
}

function mergeRequestRecords(items: RequestRecord[], updates: RequestRecord[]) {
  if (!updates.length) return items;
  const updatesById = new Map(updates.map((record) => [record.request_id, record]));
  return items.map((item) => mergeRequestRecord(item, updatesById.get(item.request_id)));
}

function mergeRequestRecord(item: RequestRecord, update?: RequestRecord) {
  if (!update) return item;
  if (statusRank(update.status) < statusRank(item.status)) return item;
  return {
    ...item,
    ...update,
    provider_id: update.provider_id ?? item.provider_id,
    provider_name: update.provider_name ?? item.provider_name,
    provider_key_name: update.provider_key_name ?? item.provider_key_name,
    provider_key_preview: update.provider_key_preview ?? item.provider_key_preview,
    provider_api_format: update.provider_api_format ?? item.provider_api_format,
    first_byte_time_ms: update.first_byte_time_ms ?? item.first_byte_time_ms,
    total_latency_ms: update.total_latency_ms ?? item.total_latency_ms,
    prompt_tokens: update.prompt_tokens ?? item.prompt_tokens,
    completion_tokens: update.completion_tokens ?? item.completion_tokens,
    total_tokens: update.total_tokens ?? item.total_tokens,
    cache_creation_input_tokens:
      update.cache_creation_input_tokens ?? item.cache_creation_input_tokens,
    cache_read_input_tokens: update.cache_read_input_tokens ?? item.cache_read_input_tokens,
    input_text_tokens: update.input_text_tokens ?? item.input_text_tokens,
    input_audio_tokens: update.input_audio_tokens ?? item.input_audio_tokens,
    input_image_tokens: update.input_image_tokens ?? item.input_image_tokens,
    output_text_tokens: update.output_text_tokens ?? item.output_text_tokens,
    output_audio_tokens: update.output_audio_tokens ?? item.output_audio_tokens,
    output_image_tokens: update.output_image_tokens ?? item.output_image_tokens,
    reasoning_tokens: update.reasoning_tokens ?? item.reasoning_tokens,
    cache_creation_5m_input_tokens:
      update.cache_creation_5m_input_tokens ?? item.cache_creation_5m_input_tokens,
    cache_creation_1h_input_tokens:
      update.cache_creation_1h_input_tokens ?? item.cache_creation_1h_input_tokens,
    usage_source: update.usage_source ?? item.usage_source,
    usage_semantic: update.usage_semantic ?? item.usage_semantic,
    has_failover: item.has_failover || update.has_failover,
    has_retry: item.has_retry || update.has_retry,
    candidate_count: Math.max(item.candidate_count, update.candidate_count),
  };
}

function shouldRefreshRecords(ids: string[], updates: RequestRecord[], statusFilter: string) {
  const updatedIds = new Set(updates.map((record) => record.request_id));
  return (
    updates.some((record) => shouldBackfillAfterUpdate(record, statusFilter)) ||
    ids.some((id) => !updatedIds.has(id))
  );
}

function shouldBackfillAfterUpdate(record: RequestRecord, statusFilter: string) {
  if (statusFilter === REQUEST_RECORD_ALL_STATUS_FILTER) return !shouldPollRequestRecord(record);
  return !requestRecordMatchesStatusFilter(record, statusFilter);
}

function statusRank(status: string) {
  return REQUEST_STATUS_RANK[status] ?? 0;
}
