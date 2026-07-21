## 1. OpenSpec Progress Data

- [x] 1.1 Extend Rust OpenSpec scanning to parse `tasks.md` checklist progress and include `done` / `total` / `status` on each change payload
- [x] 1.2 Update Rust and TypeScript shared types so Plans & Specs consumers can read progress metadata safely
- [x] 1.3 Add or update focused tests for OpenSpec scan results covering change progress with tasks, without tasks, and without checklist items

## 2. Explorer View Models

- [x] 2.1 Enrich the Plans & Specs tree builder with icon, badge, tone, and progress metadata for OpenSpec and Sisyphus nodes
- [x] 2.2 Refactor PlansSpecsView selection state so `Tree`, `List`, and `Cols` modes share the same file loading and `tasks.md` write-back flow
- [x] 2.3 Default group expansion so only `Active Changes` is expanded on initial load, with other groups collapsed
- [x] 2.4 Persist the last-used view mode per project and restore it when reselecting that project (default `Tree`)
- [x] 2.5 Add localized labels for explorer view mode switching and any new left-panel UI affordances

## 3. Explorer UI Modes

- [x] 3.1 Implement the `Tree` mode redesign: artifact icons only on `proposal.md`/`design.md`/`tasks.md` leaves, no dot/letter icon on group or change nodes, plus progress badges on `tasks.md` and change rows
- [x] 3.2 Implement the `List` mode as per-change rows (not shadowed cards): change name with right-aligned spec count, second row of clickable `proposal`/`design`/`tasks` badges, `tasks` badge showing `done/total` and status color
- [x] 3.3 Implement the `Cols` mode as two columns with single-status expansion (default `Active Changes`), each change row showing a `done/total` badge and progress bar
- [x] 3.4 Update Plans & Specs CSS so the new header, mode switcher, list rows, two-column layout, colors, badges, and panel sizing match the design

## 4. Validation

- [x] 4.1 Verify `tasks.md` interactions still update file content correctly after the multi-mode refactor
- [x] 4.2 Run frontend build and targeted Rust tests for OpenSpec scanning and document any pre-existing failures outside this change
