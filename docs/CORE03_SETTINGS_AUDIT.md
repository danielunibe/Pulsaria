# CORE-03 settings audit

**Date:** 2026-11-16
**Code baseline:** `e38731eef28551c55c86aae84560d87c27952be9`
**Documentation head when written:** `98a577e89ef9f77ba488e60460c8c4cc7653c5d6`
**Status:** Read-only source map. This is not Windows-native acceptance and does not close CORE-03.

This audit inspects the current React/Tauri settings paths and existing regression coverage. It does not mutate a user profile, install the application, or claim an unobserved UI/runtime result. No new reproducible P0 settings bug was established in this pass.

## Authority and restart flow

In Tauri, `AppSettingsSnapshot` is the native canonical settings record. `persist_app_settings_at` writes `app-settings.json` plus compatibility projections (download directory, retention, cookie browser, formats, search, processing, storage intent, quota, and reserve) through atomic writes with rollback if a later projection fails. These settings are files under `db::data_dir_path()`, not SQLite rows. At startup, `main.rs` loads the canonical snapshot and applies it to the process environment; the legacy projections are only a fallback for older installations.

The frontend `SystemSettings` is context state and a `localStorage` cache. In native mode, `SettingsProvider` reads `get_app_settings`; the Settings panel persists its form through Tauri commands on explicit Save. `updateSettings` alone updates React state and the browser cache. It is not a native persistence command.

## Current settings map

| Surface and React state | IPC and validation | Native persistence and effect | Error/readback evidence |
|---|---|---|---|
| General: locale, theme, formats, folder, tray, subtitles, playback | `SettingsPanel.handleSave` calls `save_app_settings`; Rust validates locale/theme/formats/path and processing fields in `validate_app_settings`. Folder selection uses `pick_folder`. | Canonical snapshot plus compatibility files; runtime application updates worker configuration, environment, and search config. Startup restores the snapshot. | Save errors are caught into `settingsError`; the form updates shared context only after the IPC sequence succeeds. `save_app_settings` returns a normalized snapshot, but the UI consumes only processing fields from it. |
| Storage: intent, quota, reserve, download root | Same Save path; native command expands the root, prepares the media root, measures free space, checks reserve/quota, then persists. | `worker_config` receives root, intent, quota, and reserve. Storage status is returned by `get_app_settings` and refreshed by `get_storage_status`. | Validation failures are surfaced to the form. The legacy registered `set_download_dir` command is not referenced by the current frontend; unlike `save_app_settings`, its source path does not compare the configured quota with measured free space. This is an unexercised code-path discrepancy, not a reproduced user-facing defect. |
| Retention | `StatsTab` calls `handleRetentionChange`; switching to `online` requests `preview_media_purge` and Save stays blocked while acknowledgement is pending. Actual purge requires selected jobs and `apply_media_purge`; undo and trash operations have separate IPC paths. | Retention default is saved in the canonical snapshot and worker config. Each admitted job carries its own retention snapshot, so a later default change does not rewrite an active job. Purge checks that media and generated outputs stay inside the selected managed root. | UI displays preview/error/success states. Rust regression `completion_retention_uses_the_admitted_snapshot_after_default_changes` covers the job snapshot; this does not replace drive-offline or installed-UI acceptance. |
| Browser cookies and processing consent | `EngineTab` selection is persisted by `save_app_settings`; values are restricted to Chrome, Edge, Firefox, or disabled. Legal and content-rights consent use separate `save_legal_consent` and `set_content_rights_consent` commands with current-version checks. | Consent files remain separate from application settings. The queue is authorized only when both consents are current. Queue admission and cookie scans use the current browser/authorization epoch; revocation invalidates older work. | Source tests cover both-consent gating, revoked work not resuming after rapid re-acceptance, cancellation of active work, current global cookie selection, and cancellation of a scan when consent/session changes. Consent save errors return to the modal; no real profile or cookies were inspected. |
| Whisper model and processing quality | `useProcessingSettings` reads processing settings and model status; onboarding uses `save_mvp_settings`, and later quality changes use `set_processing_settings`. The Settings panel refreshes actual processing/model state after a save. | Processing fields are part of the canonical snapshot and a compatibility projection; runtime settings update the worker configuration. Model files are prepared through separate commands. | Setup errors and model status are visible in the UI. This source pass did not download, prepare, or run a model. |
| Performance and autostart | The performance tab stages preferences. Settings Save calls `save_app_settings`, then separately calls `set_autostart` and `set_performance_policy`. | Performance settings are canonical and applied to worker/runtime policy. Autostart also changes the OS launch entry and persists the actual status. | Each IPC error reaches `settingsError`. The commands are separate writes; behavior when a later call fails after an earlier call succeeded has not been reproduced. The returned autostart boolean is not used to refresh the form in the Save handler. |
| AI providers and secrets | `AiProviderManager` calls provider config commands; API keys are kept in a password input and sent only on save. Empty key preserves the stored value. | Provider metadata is a separate native JSON config; keys use the secret-store adapter (Windows Credential Manager) rather than the metadata file. | The config view exposes whether a key exists, not the key value. Provider errors are displayed. This is a code-path inspection, not a leak test or native Credential Manager acceptance. |

## Verified source and existing tests

- Settings persistence validates a snapshot and rolls back earlier files if a later projection write fails: `commands.rs::persist_app_settings_at`, `write_settings_files_with_rollback`, and `failed_settings_projection_restores_the_canonical_snapshot_and_prior_projections`.
- Consent is fail-closed when a required document version is stale or either acknowledgement is absent: `commands.rs::processing_consent_is_current`, `legal_consent_is_not_current_until_accepted`, and `processing_requires_both_current_legal_and_content_rights_consent`.
- Queue admission monitors authorization changes while work runs, and the epoch prevents an older job or cookie scan from becoming valid after re-acceptance: `queue_service.rs::QueueAdmissionGate`, `revoked_queue_messages_do_not_resume_after_consent_is_restored`, `revoking_consent_cancels_work_already_running`, `cookie_authorization_uses_only_the_current_global_selection`, and `a_running_scan_is_cancelled_when_consent_or_session_changes`.
- The canonical CI for the unchanged code baseline, on documentation head `98a577e89ef9f77ba488e60460c8c4cc7653c5d6`, passed in run [43827346893](https://github.com/danielunibe/Pulsaria/actions/runs/43827346893). That source-only CI does not exercise the native Settings UI, a disconnected volume, or the Windows Credential Manager.

## Reproduction still required

These are validation scenarios, not confirmed defects:

1. Inject failure into each Settings Save IPC in turn; determine which earlier changes persist and what `get_app_settings` reports after reopening/restart.
2. Save a quota-normalized value and verify the form displays the returned native value rather than its pre-save input.
3. Exercise a missing/offline root with temporary fixtures; confirm the UI reports `path-error` and no media or queue state is removed.
4. Compare the legacy `set_download_dir` command with the current `save_app_settings` validation, or remove the unused command only after its callers are accounted for.
5. Exercise credential save/read/remove with a synthetic key in an isolated Windows profile and verify logs/UI never reveal it.
6. Review keyboard, screen-reader, 860×640 and 1280×800 layouts, and 200% scaling in the installed WebView.

Do not close CORE-03 from this source map alone. Its checklist still needs the IPC failure cases, isolated Windows/runtime evidence, and a reproduced bug before a fix is credited. P1-04 remains the entry gate for full human acceptance.
