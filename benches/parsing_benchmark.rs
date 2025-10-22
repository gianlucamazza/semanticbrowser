use criterion::{black_box, criterion_group, criterion_main, Criterion};
use semantic_browser::kg::KnowledgeGraph;
use semantic_browser::parser;
use std::sync::Arc;
use tokio::runtime::Runtime;

// HTML Parsing Benchmarks
fn bench_parse_html(c: &mut Criterion) {
    let html = r#"<html><body><h1>Test</h1><p>This is a test page.</p></body></html>"#;
    c.bench_function("parse_html", |b| b.iter(|| parser::parse_html(black_box(html))));
}

fn bench_parse_large_html(c: &mut Criterion) {
    let large_html = format!(
        r#"<html><head><title>Large Page</title></head><body>{}</body></html>"#,
        "<div>Content</div>".repeat(1000)
    );
    c.bench_function("parse_large_html", |b| b.iter(|| parser::parse_html(black_box(&large_html))));
}

// Knowledge Graph Benchmarks
fn bench_kg_insert(c: &mut Criterion) {
    let mut kg = KnowledgeGraph::new();
    c.bench_function("kg_insert", |b| {
        b.iter(|| {
            kg.insert(
                black_box("http://ex.org/s"),
                black_box("http://ex.org/p"),
                black_box("http://ex.org/o"),
            )
            .unwrap()
        })
    });
}

fn bench_kg_query(c: &mut Criterion) {
    let kg = KnowledgeGraph::new();
    c.bench_function("kg_query", |b| b.iter(|| kg.query(black_box("SELECT * WHERE { ?s ?p ?o }"))));
}

fn bench_kg_inference(c: &mut Criterion) {
    c.bench_function("kg_inference", |b| {
        b.iter(|| {
            let mut kg = KnowledgeGraph::new();
            // Setup test data with class hierarchy for inference
            kg.insert(
                "http://ex.org/person1",
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
                "http://ex.org/Person",
            )
            .unwrap();
            kg.insert(
                "http://ex.org/Person",
                "http://www.w3.org/2000/01/rdf-schema#subClassOf",
                "http://ex.org/Agent",
            )
            .unwrap();
            let _ = kg.infer();
        })
    });
}

// Authentication Benchmarks
fn bench_jwt_generation(c: &mut Criterion) {
    // Set up JWT environment
    std::env::set_var(
        "JWT_SECRET",
        "benchmark-secret-key-that-is-long-enough-for-jwt-validation-32chars-minimum",
    );
    let _ = semantic_browser::auth::JwtConfig::init();

    let claims =
        semantic_browser::auth::Claims::new("test_user".to_string(), Some("admin".to_string()));

    c.bench_function("jwt_generation", |b| {
        b.iter(|| {
            let _ = semantic_browser::auth::generate_token(black_box(&claims));
        })
    });
}

fn bench_jwt_validation(c: &mut Criterion) {
    // Set up JWT environment
    std::env::set_var(
        "JWT_SECRET",
        "benchmark-secret-key-that-is-long-enough-for-jwt-validation-32chars-minimum",
    );
    let _ = semantic_browser::auth::JwtConfig::init();

    let claims =
        semantic_browser::auth::Claims::new("test_user".to_string(), Some("admin".to_string()));
    let token = semantic_browser::auth::generate_token(&claims).unwrap();

    c.bench_function("jwt_validation", |b| {
        b.iter(|| {
            let _ = semantic_browser::auth::validate_token(black_box(&token));
        })
    });
}

// LangGraph Workflow Benchmarks
fn bench_langgraph_workflow(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let graph_definition = r#"
    {
        "nodes": {
            "extract": {
                "function": "extract",
                "config": {}
            },
            "query": {
                "function": "query",
                "config": {}
            }
        },
        "edges": [
            ["extract", "query"]
        ]
    }
    "#;

    let input = r#"<html><body><p>This is test content for benchmarking.</p></body></html>"#;
    let kg = Arc::new(tokio::sync::Mutex::new(KnowledgeGraph::new()));

    c.bench_function("langgraph_workflow", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = semantic_browser::external::run_langgraph_workflow(
                    black_box(graph_definition),
                    black_box(input),
                    black_box(kg.clone()),
                )
                .await;
            });
        })
    });
}

// ML Inference Benchmarks (when ONNX feature is enabled)
#[cfg(feature = "onnx-integration")]
#[allow(dead_code)]
fn bench_ml_inference(c: &mut Criterion) {
    // This would require actual ONNX model setup
    // For now, just benchmark the framework overhead
    c.bench_function("ml_inference_framework", |b| {
        b.iter(|| {
            // Placeholder for ML inference benchmark
            black_box(42)
        })
    });
}

// Browser Automation Benchmarks (when browser-automation feature is enabled)
#[cfg(feature = "browser-automation")]
#[allow(dead_code)]
fn bench_browser_automation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("browser_pool_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = semantic_browser::browser::BrowserConfig::default();
                let _ = semantic_browser::browser::BrowserPool::new(config).await;
            });
        })
    });
}

criterion_group!(
    benches,
    bench_parse_html,
    bench_parse_large_html,
    bench_kg_insert,
    bench_kg_query,
    bench_kg_inference,
    bench_jwt_generation,
    bench_jwt_validation,
    bench_langgraph_workflow
);
criterion_main!(benches);
