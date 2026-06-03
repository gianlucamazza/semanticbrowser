# CLAUDE.md — semanticbrowser

Rust semantic browser for AI agents: HTML5 semantic extraction (microdata/JSON-LD), RDF knowledge
graph + SPARQL, headless browsing (chromiumoxide), ONNX ML (NER/KG embeddings), and a full MCP server
for agent integration. JWT/RBAC auth, Prometheus/tracing observability, seccomp sandboxing.
Remote: `gianlucamazza/semanticbrowser`.

## Stack & layout
- Rust (stable, `rust-toolchain.toml`), Cargo workspace + `xtask`. Lint clippy (`clippy.toml`),
  fmt rustfmt, deps audited via `deny.toml`.
- `src/` core · `benches/` · `tests/` · `examples/` · `models/` (ONNX) · `docker/` · `docs/` (mkdocs).
- Dev stack: Ollama (local LLM) + Redis + app, hot-reload, via Docker.

## Commands (Makefile — `make help` lists all)
```bash
make build / make build-release
make run   / make run-release
make test  / make test-unit / make test-integration
make lint        # clippy
make fmt / make fmt-check
make check       # cargo check
make bench
make docker-dev-up   # Ollama + Redis + app (recommended dev env); docker-dev-down/-logs/-status
```

## Conventions
- Keep clippy clean (`make lint`) and rustfmt-formatted (`make fmt`) before committing.
- `target/` is large (multi-GB) and gitignored — safe to `cargo clean` when reclaiming space.
- ML in `models/` (ONNX); SPARQL/KG is the semantic core; MCP server is the primary agent entrypoint.
