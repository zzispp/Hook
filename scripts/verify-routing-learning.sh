#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:5555}"
ADMIN_BEARER="${ADMIN_BEARER:-}"
DB_URL="${DB_URL:-postgres://postgres:123456@127.0.0.1:5433/postgres}"
GROUP_CODE="${GROUP_CODE:-}"
MODEL_NAME="${MODEL_NAME:-}"
API_FORMAT="${API_FORMAT:-}"
WINDOW="${WINDOW:-5m}"
PROFILE_ID="${PROFILE_ID:-balanced}"
INCLUDE_EXCLUDED="${INCLUDE_EXCLUDED:-true}"
SKIP_DB_CHECKS="${SKIP_DB_CHECKS:-0}"
CONTEXT_SOURCE=()

if [[ "${1:-}" == "--help" ]]; then
  cat <<'EOF'
Usage:
  ADMIN_BEARER=... scripts/verify-routing-learning.sh

Optional env:
  BASE_URL=http://127.0.0.1:5555
  DB_URL=postgres://postgres:123456@127.0.0.1:5433/postgres
  GROUP_CODE=default
  MODEL_NAME=gpt-5.5
  API_FORMAT=openai:chat
  WINDOW=5m
  PROFILE_ID=balanced
  INCLUDE_EXCLUDED=true
  SKIP_DB_CHECKS=1
EOF
  exit 0
fi

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing command: $1" >&2
    exit 1
  }
}

need_cmd curl
need_cmd jq
if [[ -z "$ADMIN_BEARER" ]]; then
  echo "ADMIN_BEARER is required" >&2
  exit 1
fi

if [[ "$ADMIN_BEARER" == Bearer\ * ]]; then
  ADMIN_BEARER="${ADMIN_BEARER#Bearer }"
fi

if [[ "$SKIP_DB_CHECKS" != "1" || -z "$GROUP_CODE" || -z "$MODEL_NAME" || -z "$API_FORMAT" ]]; then
  need_cmd psql
fi

auth_header=("Authorization: Bearer ${ADMIN_BEARER}")

api_get() {
  local url="$1"
  curl -fsS "$url" -H "${auth_header[@]}"
}

url_encode() {
  jq -rn --arg value "$1" '$value|@uri'
}

infer_group() {
  api_get "${BASE_URL}/api/admin/groups?skip=0&limit=200" \
    | jq -r '.data.groups[] | select(.is_active == true) | .code' \
    | head -n1
}

infer_model() {
  api_get "${BASE_URL}/api/admin/models/global?skip=0&limit=200&is_active=true" \
    | jq -r '.data.models[] | select(.is_active == true) | .name' \
    | head -n1
}

infer_group_db() {
  psql "$DB_URL" -Atqc "
    select code
    from billing_groups
    where is_active = true
    order by case when code = 'default' then 0 else 1 end, code
    limit 1;
  "
}

infer_model_and_format_db() {
  psql "$DB_URL" -F $'\t' -Atqc "
    select gm.name, rrs.client_api_format
    from routing_route_states rrs
    join global_models gm on gm.id = rrs.global_model_id
    group by gm.name, rrs.client_api_format
    order by count(*) filter (where rrs.sample_count >= 20) desc,
             sum(rrs.sample_count) desc,
             gm.name,
             rrs.client_api_format
    limit 1;
  "
}

if [[ -z "$GROUP_CODE" ]]; then
  GROUP_CODE="$(infer_group_db)"
  CONTEXT_SOURCE+=("group=db")
fi

if [[ -z "$MODEL_NAME" || -z "$API_FORMAT" ]]; then
  IFS=$'\t' read -r inferred_model inferred_format <<<"$(infer_model_and_format_db)"
  if [[ -z "$MODEL_NAME" ]]; then
    MODEL_NAME="$inferred_model"
    CONTEXT_SOURCE+=("model=db")
  fi
  if [[ -z "$API_FORMAT" ]]; then
    API_FORMAT="$inferred_format"
    CONTEXT_SOURCE+=("api_format=db")
  fi
fi

if [[ -z "$GROUP_CODE" ]]; then
  GROUP_CODE="$(infer_group)"
  CONTEXT_SOURCE+=("group=api")
fi

if [[ -z "$MODEL_NAME" ]]; then
  MODEL_NAME="$(infer_model)"
  CONTEXT_SOURCE+=("model=api")
fi

if [[ -z "$API_FORMAT" ]]; then
  API_FORMAT="openai:chat"
  CONTEXT_SOURCE+=("api_format=default")
fi

if [[ -z "$GROUP_CODE" || -z "$MODEL_NAME" || -z "$API_FORMAT" ]]; then
  echo "unable to infer GROUP_CODE, MODEL_NAME, or API_FORMAT" >&2
  exit 1
fi

RANKINGS_URL="${BASE_URL}/api/admin/routing/rankings?profile_id=$(url_encode "$PROFILE_ID")&group_code=$(url_encode "$GROUP_CODE")&model=$(url_encode "$MODEL_NAME")&api_format=$(url_encode "$API_FORMAT")&window=$(url_encode "$WINDOW")&include_excluded=$(url_encode "$INCLUDE_EXCLUDED")"

echo "== Context =="
echo "base_url: ${BASE_URL}"
echo "group_code: ${GROUP_CODE}"
echo "model_name: ${MODEL_NAME}"
echo "api_format: ${API_FORMAT}"
echo "window: ${WINDOW}"
echo "profile_id: ${PROFILE_ID}"
if [[ "${#CONTEXT_SOURCE[@]}" -gt 0 ]]; then
  echo "context_source: ${CONTEXT_SOURCE[*]}"
fi
echo

echo "== Profile =="
api_get "${BASE_URL}/api/admin/routing/profiles" \
  | jq --arg profile_id "$PROFILE_ID" '
      .data.profiles[]
      | select(.id == $profile_id)
      | {
          id,
          version,
          auto_tune_enabled,
          weights,
          learning
        }'
echo

echo "== Rankings Top 10 =="
api_get "$RANKINGS_URL" \
  | jq '
      .data.items[:10]
      | map({
          rank,
          state,
          provider: .provider_name,
          key: .key_name,
          endpoint: .endpoint_name,
          final_score,
          sample_count: .raw_metrics.sample_count,
          selected_reason,
          priority_component: (
            (.components[]? | select(.code == "priority") | .contribution) // 0
          ),
          exploration_component: (
            (.components[]? | select(.code == "exploration") | .contribution) // 0
          )
        })'
echo

echo "== Priority Sanity =="
api_get "$RANKINGS_URL" \
  | jq --arg profile_id "$PROFILE_ID" '
      .data.items[:20]
      | map({
          rank,
          key: .key_name,
          priority_component: (
            (.components[]? | select(.code == "priority") | .contribution) // 0
          )
        }) as $rows
      | {
          profile_id: $profile_id,
          non_zero_priority_rows: (
            if $profile_id == "fixed_priority_plus"
            then []
            else $rows | map(select((.priority_component | tonumber) != 0))
            end
          )
        }'
echo

echo "== UCB / Exploration Evidence =="
api_get "$RANKINGS_URL" \
  | jq '
      .data.items
      | map({
          key: .key_name,
          state,
          sample_count: .raw_metrics.sample_count,
          exploration_component: (
            (.components[]? | select(.code == "exploration") | .contribution) // 0
          ),
          selected_reason
        })
      | {
          warming_routes: map(select(.state == "warming")) | length,
          routes_with_exploration_signal: map(select((.exploration_component | tonumber) > 0)) | length,
          sample_rows: .[:10]
        }'
echo

if [[ "$SKIP_DB_CHECKS" != "1" ]]; then
  echo "== Routing Metric Buckets =="
  psql "$DB_URL" -F $'\t' -Atqc "
  select count(*) from routing_metric_buckets;
  select count(*) from routing_route_states;
  select count(*) from routing_profile_versions;
  " | awk '
  NR==1 {print "routing_metric_buckets=" $0}
  NR==2 {print "routing_route_states=" $0}
  NR==3 {print "routing_profile_versions=" $0}
  '
  echo

  echo "== Latest Route State EMA =="
  psql "$DB_URL" -F $'\t' -Atqc "
  select provider_id,key_id,endpoint_id,global_model_id,
         round(ema_success_rate::numeric, 4),
         round(coalesce(ema_ttfb_ms, 0)::numeric, 2),
         round(coalesce(ema_latency_ms, 0)::numeric, 2),
         round(coalesce(ema_output_tps, 0)::numeric, 2),
         sample_count,state,last_updated_at
  from routing_route_states
  order by last_updated_at desc
  limit 10;
  "
  echo

  echo "== Latest Learning Snapshots =="
  psql "$DB_URL" -F $'\t' -Atqc "
  select profile_id, profile_version, reward_window, sample_count, created_at
  from routing_profile_versions
  order by created_at desc
  limit 10;
  "
  echo
fi

echo "== Interpretation =="
cat <<EOF
1. 权重生效：
   看 Profile 段的 weights / learning.effective_weights。
   如果 profile_id 不是 fixed_priority_plus，Priority Sanity 里 non_zero_priority_rows 应该为空。

2. UCB 探索生效：
   看 UCB / Exploration Evidence。
   warming 路由里如果 exploration_component > 0，说明探索项已经进分。

3. EMA / 学习生效：
   routing_route_states 有数据，说明 EMA 状态在持续更新。
   routing_profile_versions 有最新快照，且 learning 不为空，说明自动调权快照已生成。
EOF
