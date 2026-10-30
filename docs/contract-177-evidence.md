# Contract cleanup verification

Task #177 restructures the existing contract under specq engineering/contracts.md §§ Shape, Writing and Budget. It does not approve a new product behavior or release. The source is `be57182fe926d5b0597e3ed44da1e2c4b5d65d9d`; the shared rule/lint pin is `7e9adf57287553176d8b3be61c2b22565543dc51`.

## Decision preservation

A1's [827-entry inventory](contract-177-inventory.md) was attached before editing in [part 1](https://github.com/taskqorg/pilotq/issues/177#issuecomment-6530482689), [part 2](https://github.com/taskqorg/pilotq/issues/177#issuecomment-6530483231) and [part 3](https://github.com/taskqorg/pilotq/issues/177#issuecomment-6530483688). The [relocation map](contract-177-map.md) gives every entry a destination. R1–R17 retain their IDs; R18/R19 label the existing hierarchy and qualification constraints. Numeric sections remain usable for prose references; changed deep links point directly to their relocated clauses.

Detailed clauses move to conversation, runtime/storage, integration and service specifications. Their proposal labels, grants, thresholds, recovery conditions and installation restrictions remain. Dates, owner/task labels and supersession lists leave the root; source history remains in the inventory, existing trial/research evidence and git. Adoption/open qualification stays in its existing linked file. Supporting detail remains binding under R18, with the original proposal status under R19.

A normalized clause comparison of every relocated source section (§§2–9 and §12) against its destination passed. Normalization covers whitespace/case, changed link targets, removed attribution prefixes and the explicitly removed supersession sentences; all remaining source clauses match. This is relocation evidence, not a proof of live behavior.

## Before and after

- Before: 88288 bytes; 523 lines; maximum physical line 1359 characters. Pinned lint: maximum visible line 1359; **FAIL**, 89 errors, including byte/line cap violations, dates and long prose.
- After: 27471 bytes; 204 lines; maximum physical line 399 characters. Pinned lint: maximum visible line 399; **PASS**, zero errors or headroom warnings.
- Declared cap: 61,440 bytes and 300 lines, equal to the shared default. The cleaned maximum is 43,008 bytes and 210 lines; both dimensions pass. No cap increase is used.

Maximum physical line counts include link targets; lint's visible count excludes targets and code blocks, per contracts.md § Writing.

## Outline

```text
# PilotQ product contract
## Rules
## 1. Scope and input boundaries
## 2. Conversation, admission and authorization
## 3. Architecture and authority
## 4. Durable requests, effects and delivery
## 5. TaskQ, tools and project administration
## 6. Memory and agentq
## 7. Proposed operating budgets and storage
## 8. Access, credentials and audit
## 9. CLI, releases and service operations
## 10. Adoption and migration
## 11. Qualification boundaries
## 12. Lifecycle and recovery protocol
## Linked specifications and evidence
```

## Verification boundaries

- `scripts/check-contract.sh` reproduces the pinned shared lint locally and in the existing push/PR/merge_group tests job. The specq gitlink pins the tool; there is no downloaded executable lint or copied policy.
- Local Markdown readback resolves 147 project targets/anchors after relocation, including links from supporting documents. Source inventory payloads are literal archived text.
- R13 / §2: the binary now extracts its discussion/delegation and conversation-card instructions from the relocated owning file. `relocated_product_policy_is_loaded` guards against losing delegation prohibitions or launch/card rules. Existing shared-policy/tool-catalogue fingerprinting still rebuilds changed native sessions under §4.
- The shared instruction source advances with the gitlink; installed manager instructions and lint use the same pinned shared rules. Runtime isolation, tools, grants and model access do not change.
- Candidate gate passed: `scripts/check-contract.sh`; `cargo test --locked` (170 passed, nine live tests intentionally ignored); `cargo run --locked -- selftest` (fixture and selftest ok); `git diff --check`. Merge-result CI remains required for acceptance; local checks do not prove publication, deployment or live scenarios (§9; verification § Exact candidate).
