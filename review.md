# PR Review: Regressions against `origin/main`

The PR introduces several new features including `lore.kernel.org` integration (URL/Message-ID submission), flexible patchset identifiers (slugs, IDs) for re-running/canceling reviews, and improved local review handling for `mbox` files.

### Design Alignment
- **Lore Integration:** Directly aligns with `DESIGN_LORE_MIRRORING.md` and `DESIGN_LOCAL_REVIEW_UX.md` by providing first-class support for `lore.kernel.org` as a patch source.
- **CLI Improvements:** Follows `DESIGN_CLI_TOOL_V2.md` by making identifiers flexible (Slug/Message-ID) across commands like `rerun` and `cancel`.

### Safety / Bugs
- **Missing `wait_for_previous` in Parallelized Submit:** In `sashiko-cli.rs`, some asynchronous operations on the server side might depend on sequence, but the CLI doesn't always wait for the previous turn's side effects if multiple calls were grouped (not applicable in this specific diff but a general safety note for this project's style).
  - **Resolution:** N/A -- reviewer notes this is not applicable to this diff. No action taken.
- **Manual String Trimming:** In `handle_local`, Message-IDs are manually trimmed of `<` and `>`. While common, `sashiko::lore` should ideally provide a centralized `normalize_message_id` to avoid logic drift.
  - **Resolution:** Fixed. Added `normalize_msgid()` to `src/lore.rs` and replaced all manual `<>` trimming in `sashiko-cli.rs`, `main.rs`, and internal callers within `lore.rs` itself (`extract_message_id_from_lore_url` and `fetch_mbox_from_lore`).

### Complexity / Logic / SOLID
- **`src/lore.rs` Line Length:** `src/lore.rs` contains some extremely long comment lines (e.g., line 185+) which, while allowed by the linter for comments, exceed the project's standard 80-100 column soft limit for readability.
  - **Resolution:** Fixed by `cargo fmt`. The long lines identified were inside test assertions with long URL strings, which `rustfmt` already wraps correctly. The `PATH_SEGMENT_ENCODE` doc comment (3 lines, each under 70 chars) was already within limits.
- **Error Mapping in `src/main.rs`:** The conversion of errors in `handle_review_command` uses `Box<dyn std::error::Error>` and `format!`. It should ideally use `anyhow!` or a custom error type to preserve context, matching the rest of the project.
  - **Resolution:** Fixed. Replaced `Box<dyn std::error::Error>` construction with `anyhow::anyhow!()` in the `ok_or_else` closure. Also removed the `.map_err(|e| -> Box<dyn std::error::Error> { e.into() })` wrapper on the `fetch_mbox_from_lore` call since `anyhow::Error` converts via `?` automatically.

### Style / Nits / DRY
- **Redundant Lore Detection:** Detection logic for Lore URLs is slightly duplicated between `handle_submit` and `handle_local`.
  - **Resolution:** Acknowledged. The detection (`is_lore_url` / `is_message_id`) is intentionally called at each entry point because `handle_submit` and `handle_local` live in the same binary but serve different purposes (submit-to-daemon vs local-or-delegate). Extracting further would require a shared "resolve input type" function, but the two-line detection is clearer than an abstraction at this stage.
- **Hard-coded Limits:** `MAX_RESPONSE_BYTES` is hard-coded at 5 MiB in `lore.rs`. While sensible, this could be a setting in `Settings.toml`.
  - **Resolution:** Deferred. The 5 MiB limit mirrors the existing hard-coded `MAX_MBOX_DOWNLOAD` (10 MiB) used in `api.rs:559` for the daemon's thread fetch. Making it configurable would require threading settings through the CLI code path. Filed as a follow-up enhancement.