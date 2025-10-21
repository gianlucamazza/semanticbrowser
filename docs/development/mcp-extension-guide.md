# Model Context Protocol Extension Guide

This note summarizes best practices for packaging our Semantic Browser capabilities as an MCP extension and outlines the implementation steps we should follow, without tying the work to any specific editor or client.

## Best Practices
- **Manifest discipline**: Supply a clear manifest for the extension (ID, semver version, compatibility notes, description, maintainer contact). Align the manifest format with the target MCP client’s expectations and avoid client-specific naming collisions.
- **Permissive licensing**: Include a permissive license (MIT or Apache-2.0) at the repository root so downstream clients can redistribute compiled artifacts without friction.
- **Server lifecycle**: Build the MCP server as a standalone binary or script that boots quickly, validates configuration on startup, and terminates with informative errors so clients surface actionable diagnostics.
- **Tool definitions**: Advertise only the tools, prompts, and resources we actually implement. Provide precise JSON schemas, minimize optional fields, document argument limits, and keep tool names self-explanatory.
- **Configuration surface**: Expose user-editable settings (API tokens, base URLs, feature flags) via environment variables or configuration files. Never hardcode secrets; document required variables and reasonable defaults.
- **Distribution strategy**: Decide whether to ship prebuilt binaries, build from source during installation, or fetch artifacts on demand. Use checksums for downloaded assets and cache them in a predictable location.
- **Logging and observability**: Emit structured logs (stdout/stderr) at sensible verbosity levels, and document how to inspect them when developing or debugging.
- **Security posture**: Validate inbound requests, enforce timeouts, and isolate external dependencies (e.g., sandbox subprocesses or network calls) to prevent untrusted input from compromising the host.
- **Testing loop**: Automate smoke tests that exercise each MCP tool end-to-end. Continuously test on the major platforms we intend to support (macOS, Linux, Windows).
- **Release hygiene**: Tag releases, maintain a changelog, and version-breaking changes according to semantic versioning so clients can pin to compatible builds.

## Implementation Plan
1. **Define extension scope**  
   - Select the Semantic Browser workflows we want to surface (HTML parsing, knowledge graph queries, browsing).  
   - Draft the MCP tool and prompt schemas that correspond to our existing APIs.
2. **Implement MCP server**  
   - Add a binary (e.g., `src/bin/semantic_browser_mcp.rs`) that speaks the MCP spec and bridges to our internal logic.  
   - Support configuration via CLI flags and environment variables; include health checks.
3. **Create extension packaging**  
   - Provide the manifest, build scripts, and supporting assets required by target MCP clients.  
   - Ensure build artifacts land in a reproducible output directory (consider `cargo xtask` helpers).
4. **Handle distribution assets**  
   - Decide between embedding the MCP binary, compiling on install, or downloading from release archives.  
   - Ship checksum files and verify them before execution when downloading.
5. **Document configuration & usage**  
   - Author README instructions covering installation, configuration snippets, environment variables, and available tools.  
   - Include troubleshooting tips (log locations, common error messages).
6. **Testing and CI**  
   - Add automated tests or scripts that invoke the MCP server against representative inputs.  
   - Integrate checks into CI (lint, format, unit tests) to keep builds reliable across platforms.
7. **Publish & maintain**  
   - Prepare release notes, tag versions, and share installation guidance with partner clients.  
   - Establish a maintenance cadence for dependency updates, security patches, and compatibility reviews.

Following these steps will let us build and ship a portable MCP extension that wraps our Semantic Browser features for any compliant client.

## Current Server Entry Point
- Binary `semantic_browser_mcp` (`cargo run --bin semantic_browser_mcp`) exposes HTML parsing, knowledge graph queries, and browsing utilities as MCP tools over STDIN/STDOUT.
- Tools currently implemented:
  - `semanticbrowser.parse_html`: validates and parses raw HTML, returning semantic annotations and updating the local knowledge graph.
  - `semanticbrowser.query_kg`: executes SPARQL queries or updates with built-in validation and summarises results.
- `semanticbrowser.browse_url`: fetches a URL, produces a semantic summary plus a structured snapshot (`SemanticSnapshot`), and stores findings in the knowledge graph.
- The server announces MCP protocol version `2025-06-18`, advertises tool capabilities only, and emits structured summaries via `CallToolResult` payloads (`content` + `structuredContent`). The `structuredContent` for `browse_url` contains the `summary`, original request metadata, and a full `snapshot` (title, canonical URL, Open Graph/Twitter maps, JSON-LD/microdata counts, text preview, query matches).
