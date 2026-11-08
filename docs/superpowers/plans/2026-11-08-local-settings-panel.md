# Local Settings Panel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a persistent, local Settings panel with extensible seller/document models, safe per-type numbering, appearance and app preferences, and complete validated backup/restore.

**Architecture:** Rust/Tauri owns a versioned SQLite database, typed domain services, migrations, numbering, and backup/restore. React consumes explicit Tauri commands through a typed client; native dialogs only select user-approved paths. Settings are versioned and optional where business rules are not yet configured; issued-document snapshots and used numbers remain historical facts.

**Tech Stack:** Existing React 19, TypeScript, Vite 8, Tauri 2, Rust edition 2024; `rusqlite` 0.40 with `bundled` and `backup`; direct `serde`/`serde_json`; `image` with PNG/JPEG decoding and bounded dimensions; `base64`; a compatible stable ZIP crate plus `sha2`; Tauri 2 dialog and opener plugins; Vitest + jsdom + React Testing Library. Pin compatible versions in lockfiles during implementation; do not upgrade unrelated dependencies.

**Spec:** `docs/superpowers/specs/2026-11-08-local-settings-design.md`

## Global Constraints

- Local data stays in the per-user app data directory; never put the database in the install directory.
- The frontend does not execute arbitrary SQL; use explicit, typed Tauri commands.
- Every SQLite schema change uses a numbered, tested migration.
- VAT status, tax rates, NIP, REGON, and other registration data are optional and have no assumed values.
- Do not assume the user has a registered business or that every document type requires the same seller fields.
- Issued documents preserve their own seller/buyer/document snapshots; settings changes never rewrite them.
- Number allocation and document persistence share one transaction; duplicate issued numbers are forbidden.
- Backup restore replaces the complete database and managed resources only after explicit confirmation, and first creates a verified backup of current state.
- Invalid, incomplete, incompatible, or corrupted backups must not mutate active data.
- No company, customer, document, setting, or backup data is sent to an external service.
- Keep the UI Polish and follow the existing palette: dark surfaces, `#FF6F00` orange, `#9C27B0` purple.
- Do not implement KSeF, cloud sync, user accounts, online payments, or full accounting.

## Review Focus

- A numbering-format change, counter reset, or concurrent issuance collides with a previously issued number — Task 3 tests global per-type uniqueness, reset boundaries, and transactional allocation.
- A preview accidentally reserves a number or changes a counter — Task 3 tests preview as a read-only operation.
- A malformed archive, duplicate ZIP entry, path traversal, oversized payload, or invalid checksum damages live data — Task 4 tests rejection before confirmation and unchanged live database/assets.
- A failure while replacing the database or logo leaves a mixed restore — Task 4 tests rollback of the complete data set.
- A seller field that is absent for a non-VAT/unregistered profile blocks settings save or settings overwrite an old snapshot — Tasks 1–2 test optional fields, versioned snapshots, and that saving seller settings leaves a fixture's issued-document snapshot unchanged.

---

## File Map

- `src-tauri/src/lib.rs` — Tauri plugin registration, app-data initialization, managed state, and command registration.
- `src-tauri/src/errors.rs` — serializable application errors with stable error codes and field-addressable validation details.
- `src-tauri/src/db/mod.rs` — database state, opening/closing, transaction access, and location.
- `src-tauri/src/db/migrations.rs` — ordered, transactional migration runner using `PRAGMA user_version`.
- `src-tauri/migrations/0001_settings.sql` — initial settings, numbering-series, and issued-number schema.
- `src-tauri/src/models/settings.rs` — serializable settings, optional seller profile, versioned document seller snapshot, document-type, numbering, appearance, preference, and backup-summary models.
- `src-tauri/src/services/settings.rs` — settings defaults, persistence mapping, and domain validation.
- `src-tauri/src/services/numbering.rs` — format parser, preview, reset periods, and transaction-scoped allocation.
- `src-tauri/src/services/backup.rs` — ZIP manifest, bounded archive validation, database snapshot, managed assets, and safe restore.
- `src-tauri/src/commands/settings.rs` — typed settings and numbering Tauri commands.
- `src-tauri/src/commands/backup.rs` — typed backup, validation, restore, and data-location commands.
- `src/features/settings/types.ts` — frontend API/domain types matching Rust serde models.
- `src/features/settings/api.ts` — sole frontend boundary for settings Tauri `invoke` calls and native file dialogs.
- `src/features/settings/validation.ts` — synchronous field/format validation that mirrors backend rules for immediate feedback.
- `src/features/settings/backupFileName.ts` — timezone-stable timestamped default backup filename.
- `src/features/settings/SettingsPage.tsx` — settings navigation, loading/error/save states, and section composition.
- `src/features/settings/CompanySection.tsx` — optional seller details and logo selection/removal.
- `src/features/settings/DocumentDefaultsSection.tsx` — currency, due date, payment method, place, and notes.
- `src/features/settings/NumberingSection.tsx` — per-type series, reset policy, and non-reserving preview.
- `src/features/settings/AppearanceSection.tsx` — document visibility, footer, actual built-in variants, and live sample preview.
- `src/features/settings/AppPreferencesSection.tsx` — theme, currently actionable UI preferences, and app version.
- `src/features/settings/BackupSection.tsx` — backup creation, selected destination, validation summary, restore confirmation, and data location.
- `src/features/settings/settings.css` — responsive panel/form styles, reusing existing global tokens.
- `src/App.tsx` and `src/App.css` — replace the settings placeholder with the new page and preserve other navigation flows.
- `src-tauri/Cargo.toml`, `package.json`, lockfiles, `src-tauri/capabilities/default.json` — only required dependencies, scripts, and narrow Tauri permissions.
- `vitest.config.ts`, `src/test/setup.ts`, `src/features/settings/*.test.tsx`, Rust module tests — frontend and backend tests for the behavior owned by each task.

## Task 1: Add SQLite foundation and extensible settings schema

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/errors.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/settings.rs`
- Create: `src-tauri/src/services/mod.rs`
- Create: `src-tauri/migrations/0001_settings.sql`
- Test: Rust unit tests in `src-tauri/src/db/migrations.rs` and `src-tauri/src/models/settings.rs`

**Interfaces:**
- Produces `AppError` in `src-tauri/src/errors.rs`, with serializable `code`, `message`, and optional field errors.
- Produces `DatabaseState::open(data_dir: PathBuf) -> Result<DatabaseState, AppError>` and `DatabaseState::with_connection<T>(&self, operation: impl FnOnce(&mut Connection) -> Result<T, AppError>) -> Result<T, AppError>`; internally hold `Mutex<Option<Connection>>` so restore can close/reopen the DB under exclusive access. Store active files under `app_data_dir/data/` (`app.sqlite`, `assets/`) and recovery archives under the sibling `app_data_dir/recovery/`.
- Produces serde models: `AppSettings { company, document_defaults, numbering, appearance, preferences }`, `SellerProfile`, `DocumentSellerSnapshot { schema_version: u32, common: SellerProfile, document_extension: serde_json::Value }`, `DocumentDefaults`, `NumberingSeriesSettings { document_type, prefix, format, reset_policy, next_counter: u64 }`, `DocumentAppearanceSettings`, `AppPreferences`, and `DocumentTypeKey`.
- `SellerProfile` common fields are optional; keep document-specific snapshot data versioned and separate from common company profile. `AppSettings` must not synthesize tax or registration values.
- Initial document type key is the generic `invoice`, not `vat_invoice`; add no tax or registration defaults.

- [ ] **Step 1: Write named migration/model tests** `fresh_database_migrates_to_version_1`, `reopen_does_not_rerun_migration`, `seller_profile_allows_unregistered_optional_fields`, and `document_snapshot_supports_versioned_type_extension`; assert `invoice` serializes as a generic document key and no tax fields are filled by defaults.
- [ ] **Step 2: Run `cargo test --manifest-path src-tauri/Cargo.toml`** and confirm these tests fail before implementation.
- [ ] **Step 3: Add `rusqlite` 0.40 with `bundled` and `backup`, direct `serde`/`serde_json` dependencies, and `tempfile` as a dev dependency; implement `AppError`, serde models, and `DatabaseState`**; open the DB at `app_data_dir/data/app.sqlite` and assets at `app_data_dir/data/assets/`, then run embedded numbered migrations transactionally using `PRAGMA user_version`.
- [ ] **Step 4: Create `0001_settings.sql`** with a JSON settings key/value table, a `numbering_series` table keyed by `document_type` with `counter_period` and `next_counter`, and an `issued_numbers` table with `UNIQUE(document_type, formatted_number)`; store no presumed seller/tax values.
- [ ] **Step 5: Run `cargo test --manifest-path src-tauri/Cargo.toml`** and confirm fresh initialization, repeat initialization, optional values, and schema version pass.
- [ ] **Step 6: Commit** `git commit -m "feat: add local settings database foundation" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 2: Implement settings persistence, validation, and typed Tauri commands

**Files:**
- Modify: `src-tauri/Cargo.toml` (add bounded PNG/JPEG image decoding and Base64 dependencies)
- Create: `src-tauri/src/services/settings.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Test: `src-tauri/src/services/settings.rs` and `src-tauri/src/commands/settings.rs` unit tests

**Interfaces:**
- Commands: `get_settings() -> Result<AppSettings, AppError>`, `get_app_version() -> Result<String, AppError>`, `save_company_settings(input: SellerProfile) -> Result<SellerProfile, AppError>`, `set_company_logo(source: PathBuf) -> Result<SellerProfile, AppError>`, `remove_company_logo() -> Result<SellerProfile, AppError>`, `get_company_logo_data_url() -> Result<Option<String>, AppError>`, `save_document_defaults(input: DocumentDefaults) -> Result<DocumentDefaults, AppError>`, `save_appearance_settings(input: DocumentAppearanceSettings) -> Result<DocumentAppearanceSettings, AppError>`, and `save_app_preferences(input: AppPreferences) -> Result<AppPreferences, AppError>`.
- Validate uploaded logos by decoding PNG/JPEG (not extension alone), with a 10 MiB file limit and a 25-megapixel decoded-pixel limit; store under a generated asset name and return data URL for preview.
- Every save validates and commits the complete section in one transaction. Return field-addressable validation errors; do not return success for partial writes. A seller-settings save touches only settings data, never document snapshot rows.

- [ ] **Step 1: Write named service tests** `settings_round_trip_after_reopen`, `empty_unregistered_seller_is_valid`, `invalid_supplied_email_returns_field_error`, `invalid_theme_returns_field_error`, `rejects_logo_with_mismatched_extension_or_oversize_pixels`, and `save_company_settings_does_not_change_issued_snapshot`; in the last test insert an immutable fixture snapshot row and assert its bytes are unchanged after saving seller settings.
- [ ] **Step 2: Run `cargo test --manifest-path src-tauri/Cargo.toml services::settings::tests`** and confirm the new cases fail before service/command implementation.
- [ ] **Step 3: Implement settings read/write mapping** in `services/settings.rs`; preserve forward-compatible optional/versioned payloads and only validate values explicitly supplied by the user.
- [ ] **Step 4: Add bounded logo handling** using PNG/JPEG decoder and Base64 data URLs; copy validated files to generated names in `data/assets/`, atomically update the company profile reference, and remove only unreferenced managed logo files.
- [ ] **Step 5: Implement typed settings/logo commands and `get_app_version`** and register them in `lib.rs`; initialize `DatabaseState` from Tauri's app-data directory during setup and fail startup with a clear error if migrations fail.
- [ ] **Step 6: Run `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo check --manifest-path src-tauri/Cargo.toml`**; verify failed validation rolls back the section and successful saves persist after reopening the DB.
- [ ] **Step 7: Commit** `git commit -m "feat: add typed settings commands" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 3: Implement per-document numbering and non-reserving preview

**Files:**
- Create: `src-tauri/src/services/numbering.rs`
- Modify: `src-tauri/src/models/settings.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Test: unit/integration tests in `src-tauri/src/services/numbering.rs`

**Interfaces:**
- `NumberResetPolicy` is `Yearly | Monthly | Never`; `NumberPreview { document_type: DocumentTypeKey, formatted_number: String, period_key: String, counter: u64 }`; `IssuedNumber { document_type: DocumentTypeKey, formatted_number: String, period_key: String, counter: u64 }`.
- Number template tokens are exactly `{prefix}`, `{year}`, `{month}`, and `{counter}`; unknown, unmatched, or missing required tokens fail validation. `{counter}` renders the current positive integer in base 10; reject counters below `1`.
- `preview_next_number(document_type: DocumentTypeKey) -> Result<NumberPreview, AppError>` is read-only and returns formatted number plus period/counter; UI presents it as an estimate that may change.
- `allocate_number_in_transaction(tx: &Transaction, document_type: &DocumentTypeKey, issue_year: u16, issue_month: u8) -> Result<IssuedNumber, AppError>` advances only inside the transaction that also inserts the document/issued-number row; require year `1..=9999` and month `1..=12`. No standalone command reserves a number.
- Yearly reset requires `{year}`; monthly reset requires `{year}` and `{month}`. There is no reset when policy is `Never`.

- [ ] **Step 1: Write named numbering tests** `renders_all_four_tokens`, `rejects_unknown_or_unclosed_token`, `yearly_reset_requires_year_token`, `monthly_reset_requires_year_and_month`, `series_are_independent_by_document_type`, `preview_does_not_change_counter`, `format_change_skips_historical_collision`, `concurrent_allocations_commit_unique_numbers`, and `transaction_rollback_keeps_number_unissued`.
- [ ] **Step 2: Run `cargo test --manifest-path src-tauri/Cargo.toml services::numbering::tests`** and confirm the new cases fail before the numbering service exists.
- [ ] **Step 3: Implement template validation and rendering** with the exact four-token grammar; reject unknown tokens, reset formats lacking period identity, zero/negative next counters, and an empty format.
- [ ] **Step 4: Implement read-only preview** using a consistent read transaction; do not update `numbering_series` or `issued_numbers`.
- [ ] **Step 5: Implement transaction-scoped allocation**: compute candidate from series/reset period, check the global per-type issued-number constraint, advance past any historic collision, insert the number and update the counter in the caller's transaction. A failed transaction leaves both unchanged.
- [ ] **Step 6: Add Tauri commands** `get_numbering_settings()`, `save_numbering_settings(input)`, and `preview_next_number(document_type)`; only allow the registered generic `invoice` type until other types are implemented.
- [ ] **Step 7: Run `cargo test --manifest-path src-tauri/Cargo.toml services::numbering::tests` and the full `cargo test --manifest-path src-tauri/Cargo.toml`**; verify two concurrent allocation transactions cannot commit the same number.
- [ ] **Step 8: Commit** `git commit -m "feat: add safe per-document numbering" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 4: Implement complete backup, validation, and recoverable restore

**Files:**
- Create: `src-tauri/src/services/backup.rs`
- Create: `src-tauri/src/commands/backup.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: Rust tests in `src-tauri/src/services/backup.rs`

**Interfaces:**
- `BackupSummary { path: PathBuf, format_version: u16, schema_version: u32, app_version: String, created_at_utc: String, file_count: u32, size_bytes: u64 }`; `RestoreSummary { restored_schema_version: u32, recovery_backup_path: PathBuf }`; `DataLocation { app_data_dir: PathBuf, database_path: PathBuf, assets_dir: PathBuf }`.
- `create_backup(destination: PathBuf) -> Result<BackupSummary, AppError>` uses rusqlite online backup to stage a consistent database, copies every managed asset, writes a versioned manifest and SHA-256 checksums, then writes a ZIP using create-new semantics; if the chosen name exists, append `-01`, `-02`, etc. before `.zip`, never truncate the existing file, and return the actual path.
- `inspect_backup(source: PathBuf) -> Result<BackupSummary, AppError>` is read-only and validates the archive, manifest, expected entries, entry uniqueness, checksums, schema compatibility, and SQLite integrity.
- `restore_backup(source: PathBuf, confirmed_replace: bool) -> Result<RestoreSummary, AppError>` rejects `false`, revalidates the source to avoid time-of-check/time-of-use changes, snapshots current DB/assets into an app-data `recovery/` directory that is not replaced, stages the replacement, and swaps the complete data set with rollback on failure.
- `get_data_location() -> Result<DataLocation, AppError>` returns the managed DB/data paths; the frontend opens only the returned app-managed data directory through the Tauri opener plugin.
- Archive extraction uses an isolated temporary directory, rejects path traversal, symlink/special entries, duplicates, unknown required files, compressed archives over 512 MiB, expanded archives over 1 GiB, entries over 512 MiB, more than 1024 entries, and expansion ratios over 100:1.

- [ ] **Step 1: Write named backup tests** `round_trip_contains_database_and_logo`, `repeated_backups_get_unique_names_without_overwrite`, `rejects_truncated_archive`, `rejects_duplicate_entries`, `rejects_path_traversal`, `rejects_bad_checksum`, `rejects_newer_schema`, `rejects_each_archive_limit_boundary`, `rejects_corrupt_sqlite_before_confirmation`, `restore_requires_confirmation`, and `failed_restore_restores_database_and_assets`.
- [ ] **Step 2: Run `cargo test --manifest-path src-tauri/Cargo.toml services::backup::tests`** and verify invalid-input and failure-preservation cases fail before implementation.
- [ ] **Step 3: Add ZIP and SHA-256 dependencies** at compatible stable versions and implement staged SQLite online backup plus managed asset packaging; manifest is versioned and lists each expected file and checksum.
- [ ] **Step 4: Implement read-only archive inspection** with strict entry allow-list, path/size bounds, manifest and checksum validation, and `PRAGMA integrity_check` on the staged DB; no live paths are touched.
- [ ] **Step 5: Implement restore as a guarded state transition**: require confirmation, revalidate, create and verify rollback backup, stage DB/assets, close the live connection under exclusive state lock, swap the complete data directory, reopen/migrate, and restore the old set on any failure.
- [ ] **Step 6: Add destination/location commands** and keep automatic pre-restore recovery archives in `app_data_dir/recovery/`, outside the data set being replaced.
- [ ] **Step 7: Run `cargo test --manifest-path src-tauri/Cargo.toml services::backup::tests` and the full `cargo test --manifest-path src-tauri/Cargo.toml`**; assert byte-for-byte preservation of live DB and assets for every rejected archive and injected failure.
- [ ] **Step 8: Commit** `git commit -m "feat: add validated local backup and restore" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 5: Build typed frontend client and company/default/numbering sections

**Files:**
- Modify: `package.json` and lockfile
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`
- Create: `vitest.config.ts`
- Create: `src/test/setup.ts`
- Create: `src/features/settings/types.ts`
- Create: `src/features/settings/api.ts`
- Create: `src/features/settings/validation.ts`
- Create: `src/features/settings/CompanySection.tsx`
- Create: `src/features/settings/DocumentDefaultsSection.tsx`
- Create: `src/features/settings/NumberingSection.tsx`
- Create: `src/features/settings/SettingsPage.tsx`
- Create: `src/features/settings/settings.css`
- Test: `src/features/settings/CompanySection.test.tsx`, `DocumentDefaultsSection.test.tsx`, `NumberingSection.test.tsx`, `validation.test.ts`

**Interfaces:**
- Add `SettingsApi` methods mirroring Task 2/3 commands: `getSettings`, `saveCompanySettings`, `setCompanyLogo`, `removeCompanyLogo`, `getCompanyLogoDataUrl`, `saveDocumentDefaults`, `getNumberingSettings`, `saveNumberingSettings`, `previewNextNumber`.
- `SettingsPage` loads data once, owns `activeSettingsSection`, and passes typed values plus `onSave`/`onChange` callbacks to section components.
- Use Vitest, jsdom, React Testing Library, and `@testing-library/user-event`; expose `npm test` as `vitest run` and `npm run test:watch` as `vitest`.

- [ ] **Step 1: Add Vitest/jsdom/RTL setup and failing component tests** for loading, saving, backend validation errors, optional seller fields, invoice numbering form, and preview not changing saved values.
- [ ] **Step 2: Run `npm test -- src/features/settings/`** and confirm tests fail because the feature components/API are not implemented.
- [ ] **Step 3: Add/register Tauri 2 dialog plugin** and grant only `open`/`save` file-dialog permissions; update the frontend and Rust lockfiles without unrelated upgrades.
- [ ] **Step 4: Define TypeScript types** that match Rust serde structures, including optional company fields, generic `invoice` document type, reset policy and backup summaries.
- [ ] **Step 5: Implement `api.ts`** as the only frontend invoke boundary; map backend field errors into typed validation results and do not expose direct SQL.
- [ ] **Step 6: Implement company/default forms** with all in-scope fields, native PNG/JPEG selection, backend save, dirty/saved/error states, and no registration/tax configuration UI in this iteration; keep those future model extensions optional and unset.
- [ ] **Step 7: Implement numbering form** for the registered generic invoice type, four format tokens, reset policy, year/month compatibility validation, backend preview labelled “podgląd — numer może się zmienić”, and explicit save.
- [ ] **Step 8: Implement section navigation and responsive form shell** in `SettingsPage`; confirm keyboard labels, error announcements and unsaved-change state.
- [ ] **Step 9: Run `npm test` and `npm run build`**; verify user input persists through mocked API and all optional seller fields can remain empty.
- [ ] **Step 10: Commit** `git commit -m "feat: add company and numbering settings UI" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 6: Add appearance and application preferences

**Files:**
- Create: `src/features/settings/AppearanceSection.tsx`
- Create: `src/features/settings/AppPreferencesSection.tsx`
- Modify: `src/features/settings/types.ts`
- Modify: `src/features/settings/api.ts`
- Modify: `src/features/settings/SettingsPage.tsx`
- Modify: `src/App.css` and `src/features/settings/settings.css`
- Test: `src/features/settings/AppearanceSection.test.tsx`, `AppPreferencesSection.test.tsx`

**Interfaces:**
- `saveAppearanceSettings(input: DocumentAppearanceSettings)` and `saveAppPreferences(input: AppPreferences)` call Task 2 commands.
- `AppPreferences` contains `theme: system | light | dark` and `density: comfortable | compact`; theme and density apply to the current app shell. App version comes from Tauri package metadata and is displayed read-only.
- Appearance variants have stable IDs and only appear when implemented with a working local sample preview.

- [ ] **Step 1: Write tests** for saved footer/visibility/variant, live preview updates, theme selection including system mode, persisted preferences, and read-only version display.
- [ ] **Step 2: Run `npm test -- src/features/settings/AppearanceSection.test.tsx src/features/settings/AppPreferencesSection.test.tsx`** and verify they fail before components are present.
- [ ] **Step 3: Implement appearance section** with footer, individual visibility toggles, built-in variants and a clearly watermarked sample preview; keep sample data separate from seller settings.
- [ ] **Step 4: Implement preferences section**; apply selected theme to the app root and persist it through the typed backend; show actual version metadata.
- [ ] **Step 5: Run component tests and `npm run build`**; verify no theme or preview state is reported saved before backend success.
- [ ] **Step 6: Commit** `git commit -m "feat: add document appearance and app preferences" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 7: Add backup UI and connect the panel to the app

**Files:**
- Create: `src/features/settings/BackupSection.tsx`
- Create: `src/features/settings/backupFileName.ts`
- Create: `src/features/settings/backupFileName.test.ts`
- Create: `src/features/settings/BackupSection.test.tsx`
- Modify: `src/features/settings/api.ts`
- Modify: `src/features/settings/SettingsPage.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/Cargo.toml` and `src-tauri/src/lib.rs` (register the opener plugin)

**Interfaces:**
- Add frontend calls `createBackup(destination)`, `inspectBackup(source)`, `restoreBackup(source, confirmReplace)`, and `getDataLocation()`; `openDataFolder()` passes only the returned managed directory to the opener plugin.
- `timestampedBackupFileName(now: Date): string` returns `dualzone-garage-backup-YYYYMMDD-HHMMSSZ.zip` using UTC components.
- Dialog cancellation is a no-op. Restore requires a displayed summary and a second explicit “replace all local data” confirmation, then invokes `restoreBackup(source, true)`; backend revalidation remains authoritative.

- [ ] **Step 1: Write tests** that `timestampedBackupFileName` formats UTC date/time; UI tests cover multiple destinations, dialog cancellation, invalid archive with no confirmation, explicit restore confirmation, display of the actual collision-suffixed destination returned by backend, and failure/success messaging.
- [ ] **Step 2: Run `npm test -- src/features/settings/BackupSection.test.tsx src/features/settings/backupFileName.test.ts`** and verify they fail before the backup section exists.
- [ ] **Step 3: Add/register `@tauri-apps/plugin-opener` and its Rust plugin** at a compatible Tauri 2 version; grant only the permission needed to open the backend-returned app-data folder path.
- [ ] **Step 4: Implement backup section** with native save/open dialogs, a fresh timestamped filename for each backup, data path display, open-folder action, validation summary, and explicit restore confirmation.
- [ ] **Step 5: Replace only the settings placeholder in `App.tsx`** with `SettingsPage`; leave dashboard and other planned module placeholders unchanged.
- [ ] **Step 6: Run Vitest and `npm run build`**; verify cancellation or validation errors never invoke restore and no error is shown as success.
- [ ] **Step 7: Commit** `git commit -m "feat: connect backup tools to settings panel" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.

## Task 8: End-to-end verification and documentation

**Files:**
- Modify: `README.md`
- Modify: test files only where integration gaps are found
- Test: Rust and frontend suites; manual Tauri smoke scenario

- [ ] **Step 1: Run `npm test`** and confirm all frontend unit/component tests pass.
- [ ] **Step 2: Run `npm run build` and `cargo test --manifest-path src-tauri/Cargo.toml`**; fix any regression and rerun each full command.
- [ ] **Step 3: Run `cargo check --manifest-path src-tauri/Cargo.toml`** and `npm run tauri dev`; manually exercise every settings section, restart the app, and verify values persist.
- [ ] **Step 4: Perform backup/restore smoke test** using a temporary test profile: create two backups to different destinations, alter settings/logo, inspect and restore one, then verify restored data and the automatic pre-restore backup; test a corrupted copy and verify active data remains unchanged.
- [ ] **Step 5: Update README** with data directory location behavior, manual backup/export/import flow, restore replacement warning, and current scope/limitations.
- [ ] **Step 6: Review the complete diff, run `git diff --check`, and commit** `git commit -m "test: verify settings persistence and recovery" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"`.
- [ ] **Step 7: Push the feature branch** after each accepted logical milestone, following the repository workflow; do not merge or rewrite history without explicit authorization.
