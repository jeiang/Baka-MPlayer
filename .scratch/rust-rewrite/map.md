# Map: Rust rewrite and modernization of Baka MPlayer

Tracker: local markdown (GitHub issues are disabled on this repo; the API returned 410).

## Destination

A locked set of decisions (toolkit, mpv integration, scope, fidelity bar, performance targets, workspace layout) for rewriting Baka MPlayer in Rust on Windows, Linux, and macOS. Implementation is a separate effort that starts from these decisions.

## Notes

- Domain: Qt5 Widgets C++ app (~7.5k LOC in `src/`), embeds mpv by window id (`src/mpvhandler.cpp`), platform shims in `src/platform/`, 13 translations, qmake build.
- Hard requirement: UI matches the original exactly (layouts in `src/ui/*.ui`, QSS set in `mainwindow.cpp`, `overlayhandler.cpp`, `dimdialog.cpp`, SVGs in `src/img`).
- Hard requirement: Windows, Linux, macOS. Focus on performance.
- Modernize means: drop legacy targets (WinXP, 32-bit, Qt4, X11-only paths), libmpv render API, new packaging and CI. Not new features.
- Parity: everything except dead weight. What counts as dead weight is a ticket.
- Initial lean (not a decision): Qt from Rust (cxx-qt or qmetaobject), because it is the only cheap route to an exact UI. Ticketed for verification.
- Skills for every session: grilling, domain-modeling, ponytail (smallest correct solution; challenge scope that does not serve the destination).

## Decisions so far

<!-- one line per resolved ticket: [title](issues/NN-slug.md): gist -->

## Not yet specified

- Workspace and crate layout (core vs UI vs mpv wrapper), once the toolkit is known.
- Settings and recent-files compatibility: read existing Qt config files or start clean.
- Command layer (`src/bakacommands.cpp`, `DOCS/commands.md`) and its Rust shape.
- Translation pipeline for the 13 existing `.ts` files in a Rust build.
- Packaging and CI per OS (installers, signing, Qt/libmpv bundling), after the Qt version decision.
- Test strategy, including how fidelity is checked (follows the fidelity ticket).
- Repo strategy: in-place rewrite vs new tree; fate of `website/`, `debian/`, `arch/`, `windows/`, `linux/`.
- Licensing effects of the chosen Qt linking mode on the project license.

## Out of scope

- New player features beyond current behavior.
- Mobile platforms.
- Implementing the port.
