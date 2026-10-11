# ParseKit — Flow + metrics tasks

Branch: `feat/flow-metrics-2026-07`  
Source: `~/Desktop/parsekit-task-list-plan.md`

| ID | Task | Status | Notes |
|----|------|--------|-------|
| PK-1 | First successful convert as hero of first-run | **Done** | First-run hero on main panel; onboarding points to convert; activation constant |
| PK-2 | Today + all-time token summary scoreboard | **Done** | BatchScoreboard + banner period includes **today** |
| PK-3 | Failure path + success rate | **Done** | Job outcomes (last 20 jobs, local); partial/fail copy on scoreboard |
| PK-4 | Optional destination prompt | **Done** | Soft prompt after first success; skippable; local only |
| PK-5 | Privacy strip on stats surfaces | **Done** | Scoreboard + token panel + first-run copy |
| README | Novice + technical dual sections | **Done** | See README.md |
| Ship | PR against master | Pending | |

## Activation metric

- **Name:** `first_successful_convert_with_token_estimate` (`ACTIVATION_EVENT` in `src/lib/activation.ts`)
- **Rule:** ≥1 successful file in a batch (token estimate recorded for the batch, may be 0)
- **Storage:** `activatedAt` in local Tauri settings store — never uploaded

## Privacy

Token events and job outcome counts stay on this Mac. No file paths or names in aggregate metrics.
