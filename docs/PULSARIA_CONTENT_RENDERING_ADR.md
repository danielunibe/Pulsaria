# Pulsaria content projections: Markdown and Mermaid

**Status:** Draft for UX, security, and engineering review; not an implementation approval

**Gate:** Trello C07-00

**Baseline inspected:** `f67563edf767fedce3ce3befccf41e2591f08d15` (`codex/pulsaria-beta4-prep`)

**Scope:** Documentation and proposed contracts only. No application code, dependency, SQLite schema, CSP, transcript, or profile change is included.

## Decision summary

This ADR recommends a bounded, article-only first version of editorial content projections. Markdown and Mermaid are derived, reviewable views of existing article/evidence data. They do not replace transcripts, provider JSON, article-version JSON, or evidence records.

1. **Authority:** `transcript_segments` and existing job artifacts remain the source evidence. `magazine_articles.structured_content_json` and `magazine_article_versions.structured_content_json` remain the canonical article content and history. A projection is derived from one explicit article version and its stored evidence references.
2. **Markdown:** Parse GFM into an AST and render that AST through React elements. Raw HTML is disabled. No Markdown string is passed to `dangerouslySetInnerHTML`.
3. **Mermaid:** Retain diagram source text and an explicit diagram kind. The proposed first renderer accepts only `flowchart` and `sequence` diagrams. Other diagram kinds, directives, callbacks, click/link actions, HTML labels, or invalid input fall back to escaped source text. SVG is disposable output, never authority.
4. **Persistence:** Use a separate, additive article-projection record tied by foreign key to `magazine_article_versions.id`, rather than embedding unapproved drafts in the active article JSON. This supports `draft` / `requires_review` / `approved` / `rejected` states without changing what the existing reader renders. Video and note projections are outside V1 until their owning records and foreign-key paths are identified.
5. **Approval:** AI-created projections start as `draft` or `requires_review`; only an explicit human action may approve one. A renderer displays a projection only when it is approved and bound to the active article version. This lifecycle does not alter the existing article compiler's publication behavior.
6. **Evidence:** References are stored as structured IDs, validated against the article and bound version on write, and resolved from SQLite. They are never extracted from Markdown, HTML, SVG, or diagram labels.
7. **Legacy behavior:** Existing JSON shapes and article versions remain readable and unchanged. Unknown projection versions fail closed to the existing reader/fallback behavior.
8. **Resource limits:** Parsing and rendering must have hard byte, AST, diagram, SVG, and time bounds. Numeric caps remain **UNKNOWN** pending measurement against synthetic fixtures; they must be set and tested before C07-01 implementation is mergeable.
9. **Adoption:** Markdown and Mermaid each get a separate default-off feature switch. Neither ships enabled in Beta 4. A single kill switch disables both render paths and returns to the existing reader.
10. **CSP:** Keep the current production and development CSP unchanged. No `unsafe-eval`, wildcard source, wider network access, or frame allowance may be added for either renderer. If a selected renderer cannot work under the existing policy, do not enable it.

These are recommendations for review, not approved architectural decisions. C07-01 remains blocked pending UX, security, and engineering sign-off, numeric resource budgets, and the P1-04 installed-WebView acceptance gate.

## Current baseline and ownership map

Line numbers below were checked at the baseline SHA above. They are navigation anchors and must be refreshed if the implementation baseline changes.

| Stage / owner | Current responsibility | Baseline anchors |
|---|---|---|
| Transcript extraction | `transcriber.py` creates transcript text and time-bounded segments. | `python-workers/transcriber.py:374-451` |
| Transcript export | `output_generator.py` writes selected TXT/SRT/VTT/JSON-style output formats; this is an export path, not a Markdown renderer. | `python-workers/output_generator.py:134-205` |
| Transcript reader | `ExpandedVideoModal.tsx` reads persisted transcript segments and supports copy/export; it is not a universal Markdown renderer. | `components/ExpandedVideoModal.tsx:293-342, 730-743` |
| Editorial evidence package | Reads existing source, transcript, keyframe, OCR, and annotation records, preserving source IDs and timestamps. | `src-tauri/src/application/editorial_evidence.rs:181-231`; record types at `:54-108` |
| Provider contract | Provider prompt requests JSON schema 1.0; response parsing deserializes into the editorial payload. | `src-tauri/src/application/editorial_provider.rs:248-305` |
| Contract and provenance | The Rust payload carries `content`, `sources`, `evidence`, conflicts, and confidence; contract validation checks required metadata and evidence/source consistency. | `src-tauri/src/domain/editorial.rs:482-559, 619`; provenance in `src-tauri/src/application/editorial_compiler.rs:946` |
| Compilation pipeline | Collects evidence, requests a provider proposal, validates contract and provenance, then persists an article/version. | `src-tauri/src/application/editorial_compiler.rs:284, 392, 515` |
| SQLite authority | Articles and versions each store `structured_content_json`; evidence and sources are separate article-linked records. | `src-tauri/src/application/magazine_service.rs:76, 96, 130`; types in `src-tauri/src/domain/editorial.rs:360-418` |
| Existing version writes | Article revisions append to `magazine_article_versions`; article-level evidence attachment is not version-scoped today. | `src-tauri/src/application/magazine_service.rs:933, 1309` |
| Frontend projection parser | `StructuredContentBlock` and `parseStructuredBlocks` normalize arrays, `{blocks: []}`, recipe-shaped objects, and legacy text. | `lib/magazines.ts:578-735` |
| Reader rendering | `BlockView` renders known block types as React text/elements; the reader calls the shared parser before pagination. | `components/KioscoReader.tsx:103-187, 400` |
| Existing SVG insertion | `coverArtSVG` and `softArtSVG` build procedural local SVG from numeric seed/hue; current insertion sites use those generated values. | `lib/kiosco.ts:52, 87`; `components/KioscoReader.tsx:662, 874`; `components/MagazinesBookshelf.tsx:315, 367` |
| CSP and security tests | Tauri production CSP has explicit sources, `script-src 'self' 'unsafe-inline'`, and no `unsafe-eval`; the security test asserts restrictions and opener behavior. | `src-tauri/tauri.conf.json:84`; `scripts/tests/tauri-security.test.mjs` |
| Frontend dependency baseline | React and Next are direct dependencies; no direct Markdown/GFM or Mermaid dependency is declared. | `package.json:36` |

`dangerouslySetInnerHTML` currently appears at the cited SVG sites. The inspected strings come from the local procedural SVG generators, not article or provider text. This observation is limited to those data paths and is not a general approval for future SVG insertion.

### Existing flow

```text
job media and transcript segments
    → editorial evidence package (source IDs, segment times, artifact IDs)
    → JSON schema 1.0 provider proposal (article content + sources/evidence)
    → Rust contract and provenance validation
    → SQLite article JSON + immutable article-version JSON; sources/evidence in linked records
    → parseStructuredBlocks(structured_content_json)
    → typed React BlockView + pagination; separate evidence pages resolve stored records
```

The proposed Markdown/Mermaid path is additive and attaches to a specific article-version record:

```text
stored article version + validated evidence IDs
    → draft projection record (Markdown AST source or Mermaid source text)
    → user review and explicit approval
    → bounded parser → plain-text index projection and safe React/diagram view
```

The transcript and JSON article flow remains authoritative and usable if projection parsing fails or the feature switch is off.

## Proposed contracts

### Projection record (not present in the current schema)

The table name and schema below are a proposal, not existing code. The exact DDL, indexes, constraints, and migration still require C07-01 review.

| Field | Proposed meaning / constraint |
|---|---|
| `id` | Local primary key. |
| `article_version_id` | Required FK to `magazine_article_versions.id`; this is the source revision and pins the projection to an immutable article version. |
| `projection_kind` | `markdown` or `diagram`. |
| `content_text` | Markdown source or Mermaid source code; never generated SVG. |
| `language` | Optional language tag for Markdown labels/code presentation. |
| `diagram_type` | Required for `diagram`; V1 allowlist is `flowchart` and `sequence`. |
| `alt_text`, `caption` | Required for diagrams; plain text, displayed as accessible description/caption. |
| `created_by` | `human`, `local_model`, or `remote_optin`; remote generation requires the existing explicit remote-provider opt-in. |
| `status` | `draft`, `requires_review`, `approved`, or `rejected`; generation cannot set `approved`. |
| `approved_at`, `approved_by` | Nullable approval audit fields set only by the explicit human approval action. |
| `renderer_version` | Renderer/parser version used for preview; does not change source authority. |
| `content_hash` | Lowercase SHA-256 of `UTF8(projection_schema_version + LF + projection_kind + LF + content_text)`, not SVG output. |
| `created_at`, `updated_at` | Local record timestamps. |
| evidence references | Separate child rows referencing `magazine_evidence.id`; the write path must verify each evidence row belongs to the same article as `article_version_id`. |

This article-only table is preferred over embedding a draft block inside active `structured_content_json`: today the active reader renders text blocks and there is no projection approval state, while article JSON is already the canonical published/versioned article. A separate row keeps drafts isolated until approval and gives the new renderer a clear kill switch. The cost is a new table, Tauri commands, validation, and a version-aware join. Do not add a polymorphic `parent_type + parent_id` without real foreign keys. Video and note support require a follow-up ADR when their authoritative version records are identified.

On approval, the stored projection remains pinned to its original `article_version_id`. It is rendered only if that version is active. A later article version does not rewrite or silently rebind an older approved projection. Re-approval against a later version creates a new projection record. The existing article compiler and version writes remain unchanged in C07-01.

### Markdown contract

- Parse GFM to an AST using a reviewed, pinned parser; render only an explicit AST node allowlist as React components.
- Disable raw HTML. Do not pass provider/user Markdown or renderer HTML to `dangerouslySetInnerHTML`.
- V1 supports text, headings, emphasis, lists, block quotes, and code. Markdown images are disabled.
- Link destinations must parse as absolute HTTPS URLs and pass the existing external URL opener policy. Unsupported or rejected destinations render as non-clickable text; no `javascript:`, `data:`, `file:`, custom scheme, or raw HTML navigation.
- Derive searchable plain text from the validated AST. Do not index markup, hidden HTML, diagram source, or URLs as article facts.

### Mermaid contract

- Input is source code plus `diagram_type`, not an SVG string. The client cannot submit trusted SVG, Mermaid configuration, CSS, script, callback, or URL fields.
- V1 accepts only `flowchart` and `sequence` grammar. Reject Mermaid initialization/config directives, `click` and link directives, HTML labels, unsupported diagram kinds, external resources, and malformed input. On rejection, show escaped source text with a non-rendered status.
- Render asynchronously with cancellation and bounded work. Treat the renderer's SVG as untrusted output: allowlist inert SVG elements/attributes before building React SVG nodes. Disallow scripts, event attributes, links, stylesheets, `foreignObject`, external references, and embedded data URLs.
- SVG is a transient cache/output. Persist source code and alt text/caption only. Require useful alt text and retain a text fallback for assistive technology and renderer failure.

### Approval and evidence contract

- A provider may create a projection proposal, never an approved record. A local model follows the same rule.
- Approval is an explicit user action after preview. The service validates the article-version relation, approval state transition, content hash, diagram type, and each evidence ID's article ownership before persisting approval.
- Projection evidence references resolve only through `magazine_evidence` and its source/job relationships. Timestamps and frame references are read from those records. Markdown/HTML/SVG cannot create or alter citations.
- A projection whose article version is not active is historical and not rendered in the active reader. Rejection disables that projection without touching source transcripts, evidence, or article JSON.

### Synthetic contract fixtures

These examples describe a future projection-record API/table. They are not accepted by provider schema 1.0 today and do not claim current implementation. Test fixtures must use an isolated SQLite database with synthetic article/version/evidence rows and placeholder text only.

Valid Markdown proposal (the fixture database supplies article version `1`; evidence references may be empty):

```json
{
  "projection_schema_version": "1",
  "article_version_id": 1,
  "projection_kind": "markdown",
  "content_text": "# TEST ONLY\n\nPlain fixture text.",
  "language": "en",
  "created_by": "local_model",
  "status": "draft",
  "evidence_refs": [],
  "renderer_version": "fixture",
  "content_hash": "4b06f44ca5b9aef40d214e073ab72602dd283116b5be1612d6337d707a180c91"
}
```

Valid diagram proposal (synthetic labels only; the fixture builder computes the hash):

```json
{
  "projection_schema_version": "1",
  "article_version_id": 1,
  "projection_kind": "diagram",
  "diagram_type": "flowchart",
  "content_text": "flowchart LR\n  A[TEST A] --> B[TEST B]",
  "alt_text": "Test-only two-node flowchart.",
  "caption": "Fixture, not product content.",
  "created_by": "local_model",
  "status": "draft",
  "evidence_refs": [],
  "renderer_version": "fixture",
  "content_hash": "d7e768dd9e308b4c1dff0fcda52db1cfb02df543759f7fdc2cf6b11e5b83f717"
}
```

Invalid contract fixtures and required result:

| Input | Expected result |
|---|---|
| Missing `article_version_id`, unknown `projection_kind`, or unsupported projection schema version | Reject before persistence; active article is unchanged. |
| `status: "approved"` from `created_by: "local_model"` | Reject; only the explicit approval command may set approval metadata. |
| Evidence ID absent or belonging to a different article | Reject the reference; do not persist a partial approval. |
| Markdown source containing `<script>` or an HTML event handler | Treat as literal/unsupported text; rendered DOM contains no active HTML element or handler. |
| Markdown link to `javascript:`, `data:`, `file:`, or an unapproved HTTPS host | Render as non-clickable text; no navigation or fetch occurs. |
| Diagram with `click`, `%%{init...}`, HTML label, link, external URL, unsupported type, malformed syntax, or resource-budget overrun | Do not render; show escaped source and an accessible failure status. |
| Malformed JSON or a future schema version | Reject; preserve the legacy reader and all stored source data. |

`content_hash` is produced by the persistence service from canonical source bytes and projection schema version. Both sample hashes use the documented formula; executable fixtures must recompute and assert them rather than copy unchecked constants.

## Compatibility and rollback

| Stored/read case | Proposed behavior |
|---|---|
| Existing array, `{ "blocks": [...] }`, flat recipe object, or legacy text in article JSON | Continue through the current parser and `BlockView` unchanged. |
| Existing article version predating projections | Read with current behavior; no backfill or rewrite. |
| New projection table absent (older DB or feature disabled) | Reader behaves exactly as it does today. |
| New draft/review projection | Not visible in the active published reader. |
| Approved projection bound to active article version | Render only when the relevant per-surface feature switch is enabled and every validation succeeds. |
| Unknown renderer/schema version or parse failure | Fail closed to plain text/status; article and transcript remain available. |
| Article version advances | Old projection remains historical and is not silently rebound; a new projection is required for the new version. |
| Rollback after migration | Turn switches off and keep projection rows. Do not drop user-created rows in a down migration; dropping requires a separately reviewed export/backup plan. |

Migration is additive: add the projection table and evidence-reference relation only after schema tests; do not alter transcript rows, source IDs, article JSON, old version rows, or profile permissions. Rollout begins disabled. Rollback is the default-off switch plus the legacy reader; data is retained for a later compatible build.

## Threat model and controls

| Threat | Boundary / control | Required evidence before enablement |
|---|---|---|
| XSS from Markdown HTML, SVG, labels, or attributes | Raw HTML disabled; AST/diagram allowlists; no raw HTML sink; React node construction from validated data. | Hostile fixtures prove no scriptable nodes, event handlers, navigation, or data execution in DOM/WebView. |
| URL scheme abuse or remote fetch | HTTPS links only through the existing Rust allowlist; Markdown images disabled; diagrams cannot reference remote assets. | Reject scheme/lookalike/credential/port cases and assert no network request. |
| SVG active content or parser escape | Source-only persistence; strict element/attribute allowlist; disallow `foreignObject`, scripts, event attributes, references, CSS, and links. | Fuzzed malformed SVG/parser output corpus; snapshot/DOM assertions. |
| Evidence forgery or cross-article reference | Persist IDs, resolve from SQLite, verify article ownership and version binding on write; never parse refs from render text. | Foreign/missing/stale evidence fixtures rejected atomically. |
| Provider prompt injection or false approval | Provider output is proposal only; server sets initial state; explicit human approval command enforces transition. | Provider fixture that requests approval remains draft/review; UI test proves approval is a deliberate action. |
| CPU/memory exhaustion | Hard measured limits for bytes, AST depth/node count, Mermaid code/graph size, SVG nodes/bytes, render time and cache. Async work must be cancellable. | Limits established from synthetic workload and tested at boundary and over-limit. **Numeric values UNKNOWN.** |
| CSP regression | Keep current CSP; do not add `unsafe-eval`, wildcards, new external origins, or frames. Test real packaged WebView. | Existing CSP test remains green and installed WebView smoke verifies the enabled renderer. P1-04 human acceptance remains pending. |
| Stale renderer/projection | Pin source to immutable article version and parser version; on mismatch use fallback, do not rewrite source. | Backward/forward fixtures across projection schema and parser versions. |

Current Tauri CSP includes `script-src 'self' 'unsafe-inline'` and does not include `unsafe-eval`; the separate policy test asserts the no-wildcard/no-`unsafe-eval` constraints. This ADR does not claim CSP acceptance in every packaged WebView. If a library requires a CSP relaxation, it is disqualified until a separate security review approves an alternative that preserves the existing policy boundary.

## Test map

Current code tests useful for preserving adjacent contracts:

- `src-tauri/src/application/editorial_provider.rs`: JSON parsing, schema handling, invalid provider responses, and prompt budget tests (`test_mock_valid_round_trips_through_contract`, `test_invalid_json_is_an_explicit_error`, etc.).
- `src-tauri/src/domain/editorial.rs`: contract and timestamp/source validation tests.
- `src-tauri/src/application/editorial_evidence.rs`: evidence package, truncation, source identity, and deterministic hash tests.
- `src-tauri/src/application/magazine_service.rs`: article creation, version immutability, and evidence attachment tests.
- `scripts/tests/tauri-security.test.mjs`: CSP/source policy and external URL allowlist checks.
- `scripts/tests/installed-webview-smoke.mjs`: existing packaged WebView smoke harness; current P1-04 acceptance is still a human gate.

The repository search found no unit test alongside `lib/magazines.ts::parseStructuredBlocks` for the new projection AST/render path. C07-01 must add focused frontend tests before enabling any projection. Required new tests: schema/version parse; old-shape compatibility; safe Markdown AST rendering; hostile raw-HTML and URL fixtures; Mermaid type/syntax allowlist; sanitized SVG allowlist; evidence ownership/approval state transitions; cancellation and resource boundaries; feature-switch off/on behavior; and packaged WebView CSP/a11y acceptance. Run Rust/service tests, frontend tests, canonical CI, and exact-SHA installed smoke appropriate to the touched files. Do not record human visual/security acceptance as automated CI.

## Unknowns and review required

- Numeric parser, AST, SVG, cache, and time limits need measured synthetic workload results.
- The selected Markdown and Mermaid packages, exact parser versions, licenses, supply-chain posture, bundle cost, and compatibility with the current Tauri WebView/CSP have not been audited. No package is selected or installed by this ADR.
- The effective CSP/runtime behavior must be verified in the packaged Windows WebView2 build; source config tests alone are not sufficient.
- Article-level evidence rows are not currently version-scoped. The proposed reference writer must validate article ownership and pin its projection to an article-version row; reviewers must confirm whether this model satisfies expected citation history.
- Product owners must confirm whether C07 V1 may be article-only and whether Markdown, diagrams, or both should later be exposed per surface.
- UX/security/engineering reviewers must approve the contract, lifecycle, accessibility behavior, allowlists, measured limits, and rollback path before C07-01.

## Gate status

This document completes a read-only architecture proposal for C07-00. It is not a security certification, UX sign-off, implementation, test result, or QA acceptance. P1-04 remains `PARTIAL/DOING` with installed-WebView, physical-device, and human acceptance items outstanding; C07-01 remains blocked pending review and its dependency gates. No Markdown/Mermaid package, table, frontend renderer, or feature switch has been added.
