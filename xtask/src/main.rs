use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build and packaging tasks for Semantic Browser MCP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build MCP server binaries for distribution
    Build {
        /// Target triple (e.g., x86_64-unknown-linux-gnu)
        #[arg(long)]
        target: Option<String>,
        /// Release mode
        #[arg(long)]
        release: bool,
        /// Output directory
        #[arg(long, default_value = "dist")]
        output: String,
    },
    /// Create distribution package with checksums
    Package {
        /// Version to package
        #[arg(long)]
        version: Option<String>,
        /// Output directory
        #[arg(long, default_value = "dist")]
        output: String,
    },
    /// Generate MCP manifest
    Manifest {
        /// Output file
        #[arg(long, default_value = "mcp-manifest.json")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { target, release, output } => build_mcp(target, release, &output),
        Commands::Package { version, output } => package_mcp(version, &output),
        Commands::Manifest { output } => generate_manifest(&output),
    }
}

fn build_mcp(target: Option<String>, release: bool, output: &str) -> Result<()> {
    println!("Building MCP server...");

    // Ensure output directory exists
    fs::create_dir_all(output)?;

    // Build the MCP binary from parent directory
    let mut cmd = Command::new("cargo");
    cmd.current_dir("..")
        .arg("build")
        .arg("--bin")
        .arg("semantic_browser_mcp");

    let target_path = if let Some(ref target_val) = target {
        cmd.arg("--target").arg(target_val);
        target_val.as_str()
    } else {
        "target"
    };

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Build failed");
    }

    // Copy binary to output directory
    let target_dir = if release { "release" } else { "debug" };
    let binary_name = format!("../{}/{}", target_path, target_dir);
    let binary_path = Path::new(&binary_name).join("semantic_browser_mcp");

    if binary_path.exists() {
        let output_binary = Path::new(output).join("semantic_browser_mcp");
        fs::copy(&binary_path, &output_binary)?;
        println!("Binary copied to: {}", output_binary.display());

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_binary, perms)?;
        }
    } else {
        println!("Warning: Binary not found at {}", binary_path.display());
    }

    Ok(())
}

fn package_mcp(version: Option<String>, output: &str) -> Result<()> {
    println!("Creating distribution package...");

    let version = version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let package_name = format!("semantic-browser-mcp-{}", version);
    let package_dir = Path::new(output).join(&package_name);

    // Create package directory
    fs::create_dir_all(&package_dir)?;

    // Copy binary
    let binary_src = Path::new(output).join("semantic_browser_mcp");
    if binary_src.exists() {
        let binary_dst = package_dir.join("semantic_browser_mcp");
        fs::copy(&binary_src, &binary_dst)?;
    }

    // Generate manifest
    let manifest_path = package_dir.join("mcp-manifest.json");
    generate_manifest(manifest_path.to_str().unwrap())?;

    // Create checksums
    create_checksums(&package_dir)?;

    // Create archive
    let archive_name = format!("{}.tar.gz", package_name);
    let archive_path = Path::new(output).join(&archive_name);

    create_tarball(&package_dir, &archive_path)?;

    println!("Package created: {}", archive_path.display());

    Ok(())
}

fn generate_manifest(output: &str) -> Result<()> {
    println!("Generating MCP manifest...");

    let manifest = serde_json::json!({
        "mcpVersion": "2025-06-18",
        "name": "semantic-browser-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Semantic Browser MCP Server - HTML parsing, knowledge graph, and web browsing tools",
        "server": {
            "command": "./semantic_browser_mcp",
            "args": [],
            "env": {
                "RUST_LOG": "info"
            }
        },
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "tools": [
            {
                "name": "semanticbrowser.parse_html",
                "description": "Parse HTML content and extract semantic annotations",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "html": {
                            "type": "string",
                            "description": "Raw HTML content to parse"
                        }
                    },
                    "required": ["html"]
                }
            },
            {
                "name": "semanticbrowser.query_kg",
                "description": "Execute SPARQL queries against the knowledge graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "SPARQL query or update statement"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "semanticbrowser.browse_url",
                "description": "Browse a URL and extract semantic information",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "format": "uri",
                            "description": "Target URL to browse"
                        },
                        "query": {
                            "type": "string",
                            "description": "Optional focus query",
                            "default": ""
                        }
                    },
                    "required": ["url"]
                }
            }
        ]
    });

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(output, json)?;

    println!("Manifest written to: {}", output);

    Ok(())
}

fn create_checksums(package_dir: &Path) -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    let checksums_path = package_dir.join("SHA256SUMS");

    let mut checksums = String::new();

    for entry in WalkDir::new(package_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.file_name() != Some(std::ffi::OsStr::new("SHA256SUMS")) {
            let content = fs::read(path)?;
            let hash = Sha256::digest(&content);
            let hash_hex = hash.iter().fold(String::new(), |mut acc, b| {
                write!(&mut acc, "{:02x}", b).unwrap();
                acc
            });

            let relative_path = path.strip_prefix(package_dir)?.to_string_lossy();
            writeln!(checksums, "{}  {}", hash_hex, relative_path)?;
        }
    }

    fs::write(&checksums_path, checksums)?;
    println!("Checksums written to: {}", checksums_path.display());

    Ok(())
}

fn create_tarball(source_dir: &Path, archive_path: &Path) -> Result<()> {
    use std::process::Stdio;

    let mut tar_cmd = Command::new("tar");
    tar_cmd
        .arg("-czf")
        .arg(archive_path)
        .arg("-C")
        .arg(source_dir.parent().unwrap())
        .arg(source_dir.file_name().unwrap())
        .stdout(Stdio::piped());

    let status = tar_cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to create tarball");
    }

    Ok(())
}