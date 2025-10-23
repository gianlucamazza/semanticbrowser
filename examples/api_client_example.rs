//! Example: GitHub API Client
//!
//! Demonstrates using ApiClient to interact with GitHub REST API and GraphQL.

use semantic_browser::api_client::{ApiClient, ApiError};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    name: Option<String>,
    bio: Option<String>,
    public_repos: u32,
    followers: u32,
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    tracing_subscriber::fmt().with_env_filter("info,semantic_browser=debug").init();

    println!("🚀 GitHub API Client Example");
    println!("============================\n");

    // Get GitHub token from environment
    #[allow(clippy::disallowed_methods)]
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable required");

    // 1. Create API client
    let client = ApiClient::new("https://api.github.com")
        .with_bearer_token(&token)
        .with_header("Accept", "application/vnd.github.v3+json");

    // 2. GET request - Fetch user info
    println!("📥 Fetching user info...");
    let user: GitHubUser = client.get("/user").await?;

    println!("\n👤 User Information:");
    println!("  Login: {}", user.login);
    println!("  Name: {}", user.name.unwrap_or_else(|| "N/A".to_string()));
    println!("  Bio: {}", user.bio.unwrap_or_else(|| "N/A".to_string()));
    println!("  Public Repos: {}", user.public_repos);
    println!("  Followers: {}", user.followers);

    // 3. GraphQL query
    println!("\n📊 Running GraphQL query...");
    let query = r#"
        query($owner: String!, $repo: String!) {
            repository(owner: $owner, name: $repo) {
                name
                description
                stargazerCount
                forkCount
                issues(first: 5, states: OPEN) {
                    edges {
                        node {
                            title
                            number
                        }
                    }
                }
            }
        }
    "#;

    let mut variables = HashMap::new();
    variables.insert("owner".to_string(), serde_json::json!("rust-lang"));
    variables.insert("repo".to_string(), serde_json::json!("rust"));

    let result = client.graphql_query("", query, Some(serde_json::to_value(variables)?)).await?;

    if let Some(repo) = result["data"]["repository"].as_object() {
        println!("\n📦 Repository: rust-lang/rust");
        println!("  Description: {}", repo["description"].as_str().unwrap_or("N/A"));
        println!("  Stars: {}", repo["stargazerCount"]);
        println!("  Forks: {}", repo["forkCount"]);

        if let Some(issues) = repo["issues"]["edges"].as_array() {
            println!("\n  Recent Open Issues:");
            for issue in issues.iter().take(3) {
                let node = &issue["node"];
                println!("    #{}: {}", node["number"], node["title"].as_str().unwrap_or("N/A"));
            }
        }
    }

    println!("\n✅ Example completed successfully!");
    Ok(())
}
