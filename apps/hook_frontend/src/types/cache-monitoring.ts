export type CacheAffinityItem = {
  affinity_key: string;
  user_id?: string | null;
  username?: string | null;
  user_email?: string | null;
  token_name?: string | null;
  token_prefix?: string | null;
  provider_id: string;
  provider_name?: string | null;
  endpoint_id: string;
  endpoint_base_url?: string | null;
  provider_key_id: string;
  provider_key_name?: string | null;
  model_id: string;
  model_name?: string | null;
  api_format: string;
  ttl_seconds: number;
  request_count: number;
};

export type CacheAffinityPageResponse = {
  items: CacheAffinityItem[];
  total: number;
  page: number;
  page_size: number;
};

export type CacheAffinityIdentity = Pick<
  CacheAffinityItem,
  'affinity_key' | 'endpoint_id' | 'model_id' | 'api_format'
>;
