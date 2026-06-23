import type {
  ProviderModelReasoningEffort,
  ProviderKeyModelMappingCandidate,
  ProviderKeyModelMappingsForKeyResponse,
} from 'src/types/provider';

export type MappingDraft = {
  global_model_id: string;
  upstream_model_name: string;
  reasoning_effort?: ProviderModelReasoningEffort | null;
};

const MANUAL_MAPPING_PREFIX = 'manual:';

export function associationMappings(response: ProviderKeyModelMappingsForKeyResponse) {
  return Object.fromEntries(
    response.mappings.map((item) => [
      item.provider_model_id,
      {
        global_model_id: item.global_model_id,
        upstream_model_name: item.upstream_model_name,
        reasoning_effort: item.reasoning_effort ?? null,
      },
    ])
  );
}

export function associationPayload(mappings: Record<string, MappingDraft>) {
  return Object.values(mappings).map((item) => ({
    global_model_id: item.global_model_id,
    upstream_model_name: item.upstream_model_name.trim(),
    reasoning_effort: item.reasoning_effort ?? null,
  }));
}

export function visibleCandidates(
  response: ProviderKeyModelMappingsForKeyResponse | null,
  mappings: Record<string, MappingDraft>
) {
  return response?.candidates.filter((candidate) => !candidateUsed(candidate, mappings)) ?? [];
}

export function addCandidateMapping(
  candidate: ProviderKeyModelMappingCandidate,
  current: Record<string, MappingDraft>
) {
  return {
    ...current,
    [`candidate:${candidate.upstream_model_name}`]: {
      global_model_id: candidate.suggested_global_model_id ?? '',
      upstream_model_name: candidate.upstream_model_name,
      reasoning_effort: null,
    },
  };
}

export function addManualMapping(current: Record<string, MappingDraft>) {
  const nextKey = nextManualMappingKey(current);
  return {
    ...current,
    [nextKey]: emptyMappingDraft(),
  };
}

export function removeMappingDraft(
  providerModelId: string,
  current: Record<string, MappingDraft>
) {
  const next = { ...current };
  delete next[providerModelId];
  return next;
}

function emptyMappingDraft(): MappingDraft {
  return {
    global_model_id: '',
    upstream_model_name: '',
    reasoning_effort: null,
  };
}

function nextManualMappingKey(mappings: Record<string, MappingDraft>) {
  let index = 1;
  while (`${MANUAL_MAPPING_PREFIX}${index}` in mappings) {
    index += 1;
  }
  return `${MANUAL_MAPPING_PREFIX}${index}`;
}

function candidateUsed(
  candidate: ProviderKeyModelMappingCandidate,
  mappings: Record<string, MappingDraft>
) {
  return Object.values(mappings).some((item) => item.upstream_model_name === candidate.upstream_model_name);
}
