# Progress

## Log

- Created task from user request to replace spawn workaround with structural refactor.

## Recovery

Task: nonstream audit stack refactor
Current: inspect response/audit data flow.

- Owned audit DTO compiled, but removing spawn directly still reproduced stack overflow. Next strategy: box the non-stream full_response future to reduce parent future stack/state pressure without task scheduling.

- Removing record_attempt forwarding and boxing full_response did not fix the real curl. Next strategy: bubble non-stream full_response args up to execute_proxy_request so response/audit is polled from the top-level proxy future instead of nested attempt_once stack.

- Top-level full_response handling without spawn passed the reproduced curl and usage flush completed.

- Final validation passed: cargo fmt/check, focused storage tests, no full_response spawn workaround, reproduced curl returned 200 and usage flush completed.
