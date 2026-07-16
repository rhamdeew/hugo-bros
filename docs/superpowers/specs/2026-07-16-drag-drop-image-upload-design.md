# Drag & Drop Image Upload — Design

## Problem

The editor's "Insert Image" flow requires clicking a toolbar button, then either
picking an existing image from the gallery or using a native file dialog to
upload a new one. There is no way to drag an image file from the OS file
manager directly into the editor.

## Goal

Dropping an image file onto the markdown content textarea opens the existing
`ImageGallery` modal so the user can navigate to a destination folder, then
upload the dropped file there and insert it into the content at the cursor —
reusing the app's existing "file manager for gallery" (the `ImageGallery`
component) rather than building a new picker.

## Why not HTML5 drag-and-drop

Tauri v2's webview intercepts native OS file drags at the window level
(`dragDropEnabled`, on by default and not disabled in this app's
`tauri.conf.json`). Standard `ondrop`/`DataTransfer` handlers do not receive
real filesystem paths under this mode. Instead, the Tauri JS API
`getCurrentWebviewWindow().onDragDropEvent()` must be used — it emits
window-level `drag-enter` / `drag-over` / `drop` events carrying the physical
cursor position and, on drop, the absolute file paths of the dragged files.

## Architecture

**`src/routes/editor/+page.svelte`**

- In `onMount`, register `getCurrentWebviewWindow().onDragDropEvent(...)`;
  unregister the listener on destroy.
- On `over`/`enter` events: convert `event.payload.position` (physical
  pixels) to logical pixels via `window.devicePixelRatio` and test whether it
  falls inside the content textarea's `getBoundingClientRect()`. Toggle a
  `dragActive` boolean used to add a dashed-border highlight class to the
  textarea wrapper (visual feedback only).
- On `drop` events: repeat the bounds check. Ignore the event if the drop is
  outside the textarea, or if `showImageGallery` is already `true`.
- If inside the textarea: take `event.payload.paths[0]` (first path only —
  multi-file drops are not supported in this iteration; extra files are
  silently ignored). Validate its extension against
  `png/jpg/jpeg/gif/webp/svg` (case-insensitive); on failure, `alert()` with a
  message naming the supported extensions, matching this file's existing
  error-reporting style.
- On success: set `pendingImageField = { kind: 'content' }` (the existing
  mechanism used by `openImageGalleryForContent`) and a new
  `droppedImagePath` state to the dropped file's path, then set
  `showImageGallery = true`.
- Pass `dropUploadPath={droppedImagePath}` into the `<ImageGallery>` instance.
  Clear `droppedImagePath` to `null` whenever the gallery closes (both on
  successful insert and on cancel), so a stale path can't leak into a later,
  unrelated gallery open.

**`src/lib/components/ImageGallery.svelte`**

- New optional prop `dropUploadPath?: string | null`.
- When non-null, render a banner (above or alongside the existing controls
  row) showing the dropped file's basename and a primary "Upload Here"
  button, styled like the existing `.upload-btn`.
- New handler `handleUploadDropped()`:
  - Calls `backend.copyImageToProject(dropUploadPath, currentDir)` — the
    existing Rust command; no backend changes needed.
  - On success, builds a `StaticEntry`-shaped object from the returned URL
    and the dropped file's basename (`name`, `url`; `path`/`fullPath` empty,
    `size`/timestamps zeroed — these fields aren't read by the insert path)
    and calls `onSelect?.(entry)`, which is wired to the existing
    `handleImageSelect` in `+page.svelte` — the same function used by the
    manual pick-an-existing-image flow. Then sets `open = false`.
  - On failure, `alert()` with the error message, matching `handleUpload`'s
    existing error handling.
- The existing native-file-dialog "Upload" button is unchanged and remains
  available as a manual fallback alongside the new banner.

## Non-goals

- Dragging an `<img>` element from inside a web page (not a native OS file
  drag) will not trigger `onDragDropEvent` and is not supported.
- Dropping multiple files at once: only the first is used, silently.
- Drop zones outside the content textarea (frontmatter fields, sidebar,
  etc.) are out of scope for this iteration.

## Error handling

- Non-image extension dropped → `alert()`, no gallery opens.
- Drop outside the textarea bounds → ignored entirely, no UI change.
- Gallery closed/cancelled without clicking "Upload Here" → dropped file is
  never copied; no side effects.
- `copyImageToProject` failure (e.g. permissions, disk error) → `alert()`
  with the underlying error message; gallery stays open so the user can
  retry or pick a different folder.

## Testing

No JS test runner is configured in this repo (`make check` covers
TS/Svelte types, `make lint` covers ESLint). This feature will be verified
manually via `make dev-app`: drag a real image file from Finder onto the
editor's content textarea, confirm the gallery opens with the drop banner,
upload into a chosen folder, and confirm the markdown image tag is inserted
at the cursor position.
