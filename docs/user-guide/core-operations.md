# Core Operations Guide

Questo documento descrive come eseguire il Semantic Browser con la configurazione minima,
come validare rapidamente le API principali e come far fluire i dati lungo una pipeline
agente → knowledge graph senza attivare feature opzionali.

## Configurazione Minima
- Build di default (`cargo build`) senza feature flag extra.
- Variabili richieste:
  - `JWT_SECRET`: stringa di almeno 32 caratteri per abilitare l'autenticazione.
  - `KG_PERSIST_PATH` (opzionale): percorso directory per persistenza del knowledge graph.
- Logging: usa `RUST_LOG=semantic_browser=info` per vedere solo i log core.

## Smoke Test API
1. Avvia il server: `JWT_SECRET="..." cargo run`.
2. Esegui parse di una pagina:
   ```bash
   curl -X POST http://127.0.0.1:3000/parse \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret" \
     -d '{"html":"<html><head><title>Demo</title></head><body>Acme Corp.</body></html>"}'
   ```
3. Inserisci triple con SPARQL:
   ```bash
   curl -X POST http://127.0.0.1:3000/query \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret" \
     -d '{"query":"INSERT DATA { <http://example.org/acme> <http://schema.org/name> \"Acme\" }"}'
   ```
4. Conferma che le triple sono presenti:
   ```bash
   curl -X POST http://127.0.0.1:3000/query \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret" \
     -d '{"query":"SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5"}'
   ```
5. Verifica il rate limiting inviando più di 10 richieste/minuto allo stesso endpoint:
   ```bash
   for _ in {1..12}; do
     curl -s -o /dev/null -w "%{http_code}\n" \
       -H "Authorization: Bearer secret" \
       -H "Content-Type: application/json" \
       -d '{"html":"<html></html>"}' \
       http://127.0.0.1:3000/parse;
   done
   ```
   Le ultime risposte devono ritornare `429` con messaggio "Rate limit exceeded".

## Pipeline Agente di Base
1. **Input**: l'agente ottiene HTML (da crawler o hook browser).
2. **Parsing**: invia il payload a `/parse` per ricavare titolo, microdata e JSON-LD.
3. **Annotazione**: se necessario, usa `annotate_html` (in locale) per estrarre entità dal testo.
4. **Persistenza**: inserisce triple nel KG (`kg.insert` oppure SPARQL `INSERT` via `/query`).
5. **Query**: esegue query `SELECT`/`ASK` per generare insight contestuali.
6. **Azioni**: restituisce risposta all'utente o triggera automazioni esterne.

## SPARQL Pronto all'Uso
- Ultime entità inserite:
  ```sparql
  SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY DESC(?o) LIMIT 10
  ```
- Risorse legate ad un soggetto:
  ```sparql
  SELECT ?predicate ?object
  WHERE {
    <http://example.org/resource> ?predicate ?object
  }
  ```
- Esistenza relazione:
  ```sparql
  ASK {
    <http://example.org/person> <http://schema.org/worksFor> <http://example.org/company>
  }
  ```

## Feedback Rapido
- Attiva log dettagliati: `RUST_LOG=debug cargo run` per collezionare feedback.
- Apri issue con snippet di log e query utilizzate per riprodurre eventuali problemi.
