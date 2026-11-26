# Nodalia migration: Toma

Category: secondary app.
Status: Base Nodalia preparada, as of 2026-11-13. Evidence: the pull request that adds this file.

## Current state

Pure-Rust video editor (egui), MIT OR Apache-2.0. Cargo git dependencies come from getnodalia/foundation (the `nodalia-*` libraries) and nodalia-design. Windows ARM64 calls Foundation's reusable workflow. The gate, FreeBSD,
Nix, packaging lint, AGENTS.md and git-sources checks call getnodalia/foundation's reusable
workflows pinned by commit. SonarCloud was removed; CodeQL and Scorecard remain.

## Desired state

No workflow, dependency or document depends on storytold; Toma has its own release workflow; the
interface follows Foundation's [NDL standard](https://github.com/getnodalia/foundation/blob/main/standards/ndl.md).

## Inherited dependencies still pending

| Dependency | Where | Replacement | Blocked on |
|---|---|---|---|
| `storytold/craft-libs`, `storytold/vectorcraft` (links) | `packaging/web/README.md`, `packaging/macos/dmg/README.md` | prose only; the code is `nodalia-*` from getnodalia/foundation | rewording |

## Technical debt

- None from the name: every name is Toma's, internal ones included. Only file signatures keep their old names, so older files keep opening.

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, translations, reusable workflows);
getnodalia/nodalia-design (theme, icons).

## NDL work pending

Check the interface against Foundation's NDL standard: hamburger menu with About, Preferences and
Keyboard shortcuts; theme selector only in Preferences; NDL window controls on Linux only; global
menu exported over D-Bus on Linux.

## Risks and priorities

1. Releases still run the inherited release workflow.
2. Windows ARM64 packaging still runs the inherited workflow.

## Recommended order

1. A Foundation release workflow, tested with a dry run, then Toma's release on it.
2. A Foundation Windows ARM64 workflow.
3. NDL review of the interface.

## What the first agent should do

1. Read AGENTS.md and this file.
2. Pick the first item of "Recommended order".

## Releases

Toma releases through `release-auto.yml` like every Nodalia app (Foundation's `release-tag` and
`release-publish`, with `release-linux.yml`, `release-macos.yml` and `release-windows.yml`). The
packages are still named `toma-X.Y.Z-…`: rename them (packaging scripts, the release workflows,
`app=` in `scripts/check-release-notes.sh`) before the first release. Until then the notes check
refuses any notes, since they would have to name `toma` files.
