use criterion::{black_box, criterion_group, criterion_main, Criterion};
use semantic_browser::kg::KnowledgeGraph;
use semantic_browser::parser;

fn bench_parse_html(c: &mut Criterion) {
    let html = r#"<html><body><h1>Test</h1><p>This is a test page.</p></body></html>"#;
    c.bench_function("parse_html", |b| b.iter(|| parser::parse_html(black_box(html))));
}

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

criterion_group!(benches, bench_parse_html, bench_kg_insert, bench_kg_query);
criterion_main!(benches);
