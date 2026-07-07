# Efficiency rules

- Default to Sonnet. Only use Opus for hard refactors, deep debugging, or architecture decisions.
- For multi-file changes, use Plan Mode first: list files + intended changes, wait for confirmation, then execute.
- Reference files by path, not `@file` — `@` pulls in the whole file plus its CLAUDE.md tree.
- Trim pasted logs/errors to the relevant lines only.
- Don't spin up Agent Teams unless parallel work is explicitly needed — they multiply token use ~7x.
- Prefer small, targeted diffs over full-file rewrites.