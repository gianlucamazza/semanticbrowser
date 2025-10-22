use semantic_browser::{
    annotator,
    auth::{self, Claims},
    kg, parser,
};

#[test]
fn core_pipeline_roundtrip() {
    // HTML di prova con microdata e testo per l'annotator
    let html = r#"
        <html>
        <head><title>Rust Semantic Browser</title></head>
        <body>
            <article itemscope itemtype="http://schema.org/Article">
                <h1 itemprop="headline">Rust Semantic Browser</h1>
                <p itemprop="about">Semantic extraction</p>
            </article>
            <p>John Doe from Example Labs contributed to the project.</p>
        </body>
        </html>
    "#;

    // Parsing HTML
    let parsed = parser::parse_html(html).expect("parse_html deve riuscire");
    assert_eq!(parsed.title.as_deref(), Some("Rust Semantic Browser"));
    assert!(!parsed.microdata.is_empty(), "microdata atteso");

    // Annotazione entità (regex fallback)
    let entities = annotator::annotate_html(html).expect("annotate_html deve riuscire");
    assert!(
        entities.iter().any(|e| e.contains("John Doe")),
        "l'annotator deve identificare entità di base"
    );

    // Inserimento nel knowledge graph usando SPARQL INSERT DATA
    let mut kg = kg::KnowledgeGraph::new();
    let mut statements = Vec::new();
    for item in &parsed.microdata {
        let subject = if item.item_type.is_empty() {
            "http://example.org/anon-item".to_string()
        } else {
            item.item_type.clone()
        };

        for (prop, values) in &item.properties {
            for value in values {
                statements.push(format!(
                    "<{subject}> <http://schema.org/{prop}> \"{value}\" .",
                    subject = subject,
                    prop = prop,
                    value = value.replace('"', "'")
                ));
            }
        }
    }

    let insert_query = format!("INSERT DATA {{ {} }}", statements.join("\n"));
    kg.update(&insert_query).expect("inserimento nel KG deve riuscire");

    // Query di verifica
    let results = kg
        .query(
            "SELECT ?headline WHERE { \
             <http://schema.org/Article> <http://schema.org/headline> ?headline \
             }",
        )
        .expect("query deve riuscire");
    assert!(
        results.iter().any(|r| r.contains("Rust Semantic Browser")),
        "la headline deve essere presente nel KG"
    );
}

#[tokio::test]
async fn jwt_token_roundtrip() {
    // Configura segreto valido e inizializza
    std::env::set_var("JWT_SECRET", "super-secure-jwt-secret-for-core-tests-123");
    if let Err(err) = auth::JwtConfig::init() {
        assert_eq!(err, "JWT config already initialized");
    }
    assert!(auth::JwtConfig::is_enabled(), "JWT deve essere abilitato con un segreto valido");

    // Genera e valida token
    let claims = Claims::new("core-agent".to_string(), None);
    let token = auth::generate_token(&claims).expect("generazione token");
    let decoded = auth::validate_token_async(&token).await.expect("validazione token");
    assert_eq!(decoded.sub, "core-agent");
}
