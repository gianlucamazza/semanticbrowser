# Gap Analysis: Navigazione e Interazione Servizi Web

**Data**: 2025-10-22  
**Status**: 🔴 **Gap Critici Identificati**

## Executive Summary

Il progetto ha una **solida base** per navigazione headless e estrazione semantica, ma presenta **gap significativi** per interazioni avanzate con servizi web moderni.

---

## 1. Form Handling (❌ CRITICO)

### Scenario Reale
Un agente AI deve:
- Compilare form di ricerca
- Sottomettere form di login
- Interagire con dropdown multi-step
- Gestire validazione lato client

### Gap Identificati
| Feature | Status | Impatto |
|---------|--------|---------|
| Fill input fields | ❌ Assente | CRITICO |
| Select dropdown | ❌ Assente | ALTO |
| Checkbox/Radio | ❌ Assente | MEDIO |
| Form submit | ❌ Assente | CRITICO |
| File upload | ❌ Assente | BASSO |

### Esempio Caso d'Uso
```rust
// SCENARIO: Login a un servizio web
// ATTUALE: ❌ Non possibile
// RICHIESTO:
let form_data = FormData::new()
    .field("username", "admin")
    .field("password", "secret123")
    .field("remember_me", "true");

pool.fill_and_submit_form(
    "https://example.com/login",
    "#login-form",
    form_data
).await?;
```

### Soluzione Proposta
Implementare `FormInteraction` module:

```rust
// src/form_interaction.rs (NEW)
pub struct FormFiller {
    page: Arc<Page>,
}

impl FormFiller {
    pub async fn fill_input(&self, selector: &str, value: &str) -> Result<()> {
        let element = self.page.find_element(selector).await?;
        element.click().await?;
        element.type_str(value).await?;
        Ok(())
    }

    pub async fn select_dropdown(&self, selector: &str, value: &str) -> Result<()> {
        let element = self.page.find_element(selector).await?;
        element.select_option(value).await?;
        Ok(())
    }

    pub async fn submit_form(&self, form_selector: &str) -> Result<()> {
        let form = self.page.find_element(form_selector).await?;
        form.submit().await?;
        // Wait for navigation
        self.page.wait_for_navigation().await?;
        Ok(())
    }
}
```

**Effort**: 3-5 giorni  
**Priority**: 🔴 **ALTA**

---

## 2. Autenticazione Avanzata (⚠️ PARZIALE)

### Scenario Reale
- Login multi-step (email → password → 2FA)
- OAuth2 flow (Google, GitHub, Microsoft)
- Token refresh automatico
- Session persistence cross-restart

### Gap Identificati
| Feature | Status | Impatto |
|---------|--------|---------|
| Basic login (form) | ⚠️ Via cookies | PARZIALE |
| OAuth2 flow | ❌ Assente | ALTO |
| Token refresh | ❌ Assente | MEDIO |
| Session restore | ⚠️ Manuale | MEDIO |
| 2FA handling | ❌ Assente | BASSO |

### Esempio Caso d'Uso
```rust
// SCENARIO: Login OAuth2 a GitHub
// ATTUALE: ❌ Non supportato
// RICHIESTO:
let auth = OAuth2Authenticator::new()
    .provider("github")
    .client_id("...")
    .client_secret("...")
    .redirect_uri("http://localhost:8080/callback");

let session = auth.authenticate(pool).await?;
pool.set_session(session).await?;

// Successive navigazioni usano il session token automaticamente
let data = pool.navigate_and_extract("https://api.github.com/user", options).await?;
```

### Soluzione Proposta
Implementare `AuthenticationManager`:

```rust
// src/auth_manager.rs (NEW)
pub struct AuthenticationManager {
    sessions: HashMap<String, SessionData>,
}

pub struct SessionData {
    pub cookies: HashMap<String, String>,
    pub tokens: HashMap<String, String>,
    pub expires_at: Option<Instant>,
}

impl AuthenticationManager {
    pub async fn login_form(
        &mut self,
        page: &Page,
        username: &str,
        password: &str,
        form_config: &FormConfig,
    ) -> Result<SessionData> {
        // 1. Navigate to login page
        page.goto(form_config.login_url).await?;
        
        // 2. Fill credentials
        let username_field = page.find_element(&form_config.username_selector).await?;
        username_field.type_str(username).await?;
        
        let password_field = page.find_element(&form_config.password_selector).await?;
        password_field.type_str(password).await?;
        
        // 3. Submit
        let submit_btn = page.find_element(&form_config.submit_selector).await?;
        submit_btn.click().await?;
        
        // 4. Wait for redirect
        page.wait_for_navigation().await?;
        
        // 5. Extract session
        let cookies = page.get_cookies().await?;
        Ok(SessionData {
            cookies: cookies.into_iter().map(|c| (c.name, c.value)).collect(),
            tokens: HashMap::new(),
            expires_at: None,
        })
    }

    pub async fn oauth2_flow(
        &mut self,
        page: &Page,
        config: &OAuth2Config,
    ) -> Result<SessionData> {
        // 1. Start OAuth flow
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code",
            config.auth_endpoint, config.client_id, config.redirect_uri
        );
        
        page.goto(&auth_url).await?;
        
        // 2. User consent (if needed)
        if let Some(consent_selector) = &config.consent_button_selector {
            let consent_btn = page.find_element(consent_selector).await?;
            consent_btn.click().await?;
        }
        
        // 3. Extract authorization code from redirect
        page.wait_for_navigation().await?;
        let current_url = page.url().await?;
        let code = Self::extract_oauth_code(&current_url)?;
        
        // 4. Exchange code for token
        let token_response = self.exchange_code_for_token(config, &code).await?;
        
        Ok(SessionData {
            cookies: HashMap::new(),
            tokens: token_response.tokens,
            expires_at: Some(Instant::now() + Duration::from_secs(token_response.expires_in)),
        })
    }
}
```

**Effort**: 5-7 giorni  
**Priority**: 🔴 **ALTA**

---

## 3. API REST Avanzate (⚠️ LIMITATA)

### Gap Identificati
| Feature | Status | Impatto |
|---------|--------|---------|
| GET requests | ✅ Implementato | OK |
| POST/PUT/DELETE | ⚠️ Basico | MEDIO |
| JSON body | ⚠️ Basico | MEDIO |
| Multipart upload | ❌ Assente | BASSO |
| GraphQL | ❌ Assente | MEDIO |
| Custom headers | ⚠️ Via reqwest | PARZIALE |

### Soluzione Proposta
Estendere `reqwest` integration:

```rust
// src/api_client.rs (NEW)
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    default_headers: HeaderMap,
}

impl ApiClient {
    pub async fn post_json<T: Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.client
            .post(&url)
            .headers(self.default_headers.clone())
            .json(body)
            .send()
            .await?;
        
        Ok(response.json().await?)
    }

    pub async fn graphql_query(
        &self,
        query: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let body = json!({
            "query": query,
            "variables": variables
        });
        
        self.post_json("/graphql", &body).await
    }

    pub async fn upload_multipart(
        &self,
        endpoint: &str,
        files: Vec<(&str, Vec<u8>)>,
        fields: HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        let mut form = reqwest::multipart::Form::new();
        
        for (name, data) in files {
            let part = reqwest::multipart::Part::bytes(data)
                .file_name(name.to_string());
            form = form.part(name.to_string(), part);
        }
        
        for (key, value) in fields {
            form = form.text(key, value);
        }
        
        let response = self.client
            .post(&format!("{}{}", self.base_url, endpoint))
            .multipart(form)
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

**Effort**: 2-3 giorni  
**Priority**: 🟡 **MEDIA**

---

## 4. WebSocket Support (❌ CRITICO per Real-time)

### Scenario Reale
- Chat applications
- Live updates (stock prices, news feeds)
- Gaming/multiplayer
- Real-time collaboration tools

### Gap Identificati
| Feature | Status | Impatto |
|---------|--------|---------|
| WebSocket client | ❌ Assente | ALTO |
| Message handling | ❌ Assente | ALTO |
| Reconnection logic | ❌ Assente | MEDIO |
| SSE (Server-Sent Events) | ❌ Assente | BASSO |

### Soluzione Proposta
```rust
// src/websocket_client.rs (NEW)
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WebSocketClient {
    url: String,
    on_message: Box<dyn Fn(Message) + Send + Sync>,
}

impl WebSocketClient {
    pub async fn connect(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (write, read) = ws_stream.split();
        
        // Handle incoming messages
        read.for_each(|message| async {
            if let Ok(msg) = message {
                (self.on_message)(msg);
            }
        }).await;
        
        Ok(())
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

**Effort**: 3-4 giorni  
**Priority**: 🟠 **MEDIA-ALTA** (dipende da use case)

---

## 5. State Management Avanzato (⚠️ PARZIALE)

### Gap Identificati
| Feature | Status | Impatto |
|---------|--------|---------|
| LocalStorage access | ❌ Assente | MEDIO |
| SessionStorage | ❌ Assente | MEDIO |
| IndexedDB | ❌ Assente | BASSO |
| Service Workers | ❌ Assente | BASSO |

### Soluzione Proposta
```rust
// Aggiungere a BrowserPool
impl BrowserPool {
    pub async fn get_local_storage(&self, page: &Page) -> Result<HashMap<String, String>> {
        let js = r#"
            Object.keys(localStorage).reduce((acc, key) => {
                acc[key] = localStorage.getItem(key);
                return acc;
            }, {})
        "#;
        
        let result = page.evaluate(js).await?;
        Ok(serde_json::from_value(result.into_value()?)?)
    }

    pub async fn set_local_storage(&self, page: &Page, key: &str, value: &str) -> Result<()> {
        let js = format!(r#"localStorage.setItem("{}", "{}")"#, key, value);
        page.evaluate(js).await?;
        Ok(())
    }
}
```

**Effort**: 1-2 giorni  
**Priority**: 🟢 **BASSA**

---

## Roadmap Proposta

### Phase 1: Foundation (1-2 settimane) 🔴
1. ✅ Form Interaction module
2. ✅ Basic Authentication Manager
3. ✅ Enhanced API Client

### Phase 2: Advanced Auth (1 settimana) 🟡
1. ✅ OAuth2 flow support
2. ✅ Token refresh logic
3. ✅ Session persistence

### Phase 3: Real-time (1 settimana) 🟠
1. ✅ WebSocket client
2. ✅ SSE support
3. ✅ Reconnection handling

### Phase 4: State Management (3-5 giorni) 🟢
1. ✅ LocalStorage API
2. ✅ SessionStorage API
3. ✅ Cookie advanced management

---

## Esempi Pratici

### Use Case 1: E-commerce Bot
```rust
// 1. Login
auth_manager.login_form(page, "user@example.com", "pass123", login_config).await?;

// 2. Search product
let form_filler = FormFiller::new(page);
form_filler.fill_input("#search-box", "laptop").await?;
form_filler.submit_form("#search-form").await?;

// 3. Add to cart
let add_btn = page.find_element(".add-to-cart").await?;
add_btn.click().await?;

// 4. Checkout
let checkout_data = page.navigate_and_extract("/checkout", options).await?;
```

### Use Case 2: Real-time Dashboard
```rust
// WebSocket for live updates
let ws_client = WebSocketClient::new("wss://api.example.com/feed");
ws_client.on_message(|msg| {
    println!("New data: {:?}", msg);
    // Update Knowledge Graph in real-time
});
ws_client.connect().await?;
```

### Use Case 3: API Integration
```rust
// REST API with authentication
let api_client = ApiClient::new("https://api.github.com")
    .with_bearer_token("ghp_xxx");

// POST JSON
let pr_data = json!({
    "title": "Add new feature",
    "body": "Description",
    "head": "feature-branch",
    "base": "main"
});
let response = api_client.post_json("/repos/user/repo/pulls", &pr_data).await?;

// GraphQL
let query = r#"
    query($owner: String!, $repo: String!) {
        repository(owner: $owner, name: $repo) {
            issues(first: 10) {
                edges { node { title } }
            }
        }
    }
"#;
let variables = json!({ "owner": "rust-lang", "repo": "rust" });
let gql_response = api_client.graphql_query(query, &variables).await?;
```

---

## Conclusioni

### ✅ Punti di Forza Attuali
- Navigazione headless robusta
- Estrazione semantica completa
- Retry e error handling
- Health monitoring

### ❌ Gap Critici
1. **Form handling** - Blocca interazioni base
2. **Auth avanzata** - Limita accesso a servizi protetti
3. **WebSocket** - Impossibile real-time
4. **API avanzate** - POST/GraphQL limitati

### 🎯 Raccomandazione
Implementare **Phase 1** (Form + Basic Auth) entro **2 settimane** per sbloccare casi d'uso critici.

---

**Prossimi Step**:
1. Review e approvazione roadmap
2. Creazione task dettagliati
3. Implementazione Phase 1
4. Testing su scenari reali

**Maintainer**: @gianlucamazza  
**Reviewed**: Pending  
**Status**: 🔴 **Action Required**
