use tower_http::trace::TraceLayer;
use anyhow::Result;
use axum::{
    Json, Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{HeaderValue, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use colored::Colorize;
use rust_embed::RustEmbed;
use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::handlers::query;
use crate::storage;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub data_dir: Arc<PathBuf>,
    pub token: Option<String>,
}

pub async fn run(host: &str, port: u16, data_dir: &Path) -> Result<()> {
    // Ensure ZNOTE_WEB_UI_TOKEN is set, generating a random one if missing
    let token = if let Ok(t) = env::var("ZNOTE_WEB_UI_TOKEN") {
        t
    } else {
        let new_token = uuid::Uuid::new_v4().to_string();
        // Safety: We are in a single-threaded startup context
        unsafe {
            env::set_var("ZNOTE_WEB_UI_TOKEN", &new_token);
        }
        new_token
    };

    let state = AppState {
        data_dir: Arc::new(data_dir.to_path_buf()),
        token: Some(token.clone()),
    };

    

    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!(
        "{} v{} server listening on http://{}",
        "znote".bold().cyan(),
        env!("CARGO_PKG_VERSION"),
        addr
    );
    println!(
        "{} authentication token: {}",
        "znote".bold().yellow(),
        token.bold()
    );
    println!(
        "{} access via: http://{}:{}/?token={}",
        "znote".bold().green(),
        host,
        port,
        token
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/notes", get(api_list_notes))
        .route("/bookmarks", get(api_list_bookmarks))
        .route("/tasks", get(api_list_tasks))
        .route("/tags", get(api_list_tags))
        .route("/config", get(api_get_config))
        .route("/links/{id}", get(api_get_links))
        .route("/note/{id}", get(api_get_note))
        .route("/bookmark/{id}", get(api_get_bookmark))
        .route("/task/{id}", get(api_get_task))
        .route("/resolve/{id}", get(api_resolve_id))
        .route("/search", get(api_search))
        .route("/query", get(api_query))
        .route("/graph", get(api_graph))
        .with_state(state.clone());

    Router::new()
        .nest("/api", api_routes)
        .route("/", get(serve_index))
        .route("/{*file}", get(serve_asset))
        .fallback(serve_index).layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
}

#[cfg(test)]
pub fn test_router(state: AppState) -> Router {
    create_router(state)
}

#[derive(serde::Serialize)]
struct ConfigResponse {
    starred_tag: String,
    version: String,
}

async fn api_get_config() -> impl IntoResponse {
    let starred_tag = env::var("ZNOTE_STARRED").unwrap_or_else(|_| "#starred".to_string());
    let version = env!("CARGO_PKG_VERSION").to_string();
    (StatusCode::OK, Json(ConfigResponse { starred_tag, version })).into_response()
}

#[derive(serde::Serialize)]
struct LinkItem {
    id: String,
    title: String,
    #[serde(rename = "type")]
    entity_type: String,
    rel: String,
}

#[derive(serde::Serialize)]
struct LinksResponse {
    outgoing: Vec<LinkItem>,
    incoming: Vec<LinkItem>,
}

async fn api_get_links(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut outgoing = Vec::new();
    let mut incoming = Vec::new();

    if crate::storage::is_dolt_backend() {
        let db = crate::storage::dolt::DoltStorage::new(&state.data_dir);
        
        // 1. Outgoing LINKS from metadata
        let sql_out = format!("SELECT l.rel_type, l.target_id, 
                                     n.id as n_id, n.title as n_title, 
                                     b.id as b_id, b.title as b_title, 
                                     t.id as t_id, t.title as t_title 
                             FROM links l 
                             LEFT JOIN notes n ON n.id LIKE CONCAT(l.target_id, '%')
                             LEFT JOIN bookmarks b ON b.id LIKE CONCAT(l.target_id, '%')
                             LEFT JOIN tasks t ON t.id LIKE CONCAT(l.target_id, '%')
                             WHERE l.source_id = '{}' OR l.source_id LIKE '{}%'", id, id);
        
        if let Some(rows) = db.run_sql(&sql_out).ok().and_then(|res| {
            res.get("rows").and_then(|r| r.as_array().map(|a| a.to_owned()))
        }) {
            for r in rows {
                let rel = r["rel_type"].as_str().unwrap_or("rel");
                
                let (target_id, title, etype) = if let Some(nid) = r["n_id"].as_str() {
                    (nid, r["n_title"].as_str().unwrap_or(nid), "note")
                } else if let Some(bid) = r["b_id"].as_str() {
                    (bid, r["b_title"].as_str().unwrap_or(bid), "bookmark")
                } else if let Some(tid) = r["t_id"].as_str() {
                    (tid, r["t_title"].as_str().unwrap_or(tid), "task")
                } else {
                    let tid = r["target_id"].as_str().unwrap_or("");
                    (tid, tid, "note")
                };

                outgoing.push(LinkItem {
                    id: target_id.to_string(),
                    title: title.to_string(),
                    entity_type: etype.to_string(),
                    rel: rel.to_string(),
                });
            }
        }

        // 2. Incoming LINKS
        let sql_in = format!("SELECT l.source_id, l.rel_type, 
                                    n.id as n_id, n.title as n_title, 
                                    b.id as b_id, b.title as b_title, 
                                    t.id as t_id, t.title as t_title 
                            FROM links l
                            LEFT JOIN notes n ON n.id LIKE CONCAT(l.source_id, '%')
                            LEFT JOIN bookmarks b ON b.id LIKE CONCAT(l.source_id, '%')
                            LEFT JOIN tasks t ON t.id LIKE CONCAT(l.source_id, '%')
                            WHERE l.target_id = '{}' OR l.target_id LIKE '{}%'", id, id);
        
        if let Some(rows) = db.run_sql(&sql_in).ok().and_then(|res| {
            res.get("rows").and_then(|r| r.as_array().map(|a| a.to_owned()))
        }) {
            for r in rows {
                let rel = r["rel_type"].as_str().unwrap_or("rel");
                
                let (source_id, title, etype) = if let Some(nid) = r["n_id"].as_str() {
                    (nid, r["n_title"].as_str().unwrap_or(nid), "note")
                } else if let Some(bid) = r["b_id"].as_str() {
                    (bid, r["b_title"].as_str().unwrap_or(bid), "bookmark")
                } else if let Some(tid) = r["t_id"].as_str() {
                    (tid, r["t_title"].as_str().unwrap_or(tid), "task")
                } else {
                    let sid = r["source_id"].as_str().unwrap_or("");
                    (sid, sid, "note")
                };

                incoming.push(LinkItem {
                    id: source_id.to_string(),
                    title: title.to_string(),
                    entity_type: etype.to_string(),
                    rel: rel.to_string(),
                });
            }
        }
        return (StatusCode::OK, Json(LinksResponse { outgoing, incoming })).into_response();
    }

    // Original FS fallback
    let mut outgoing = Vec::new();
    let mut incoming = Vec::new();

    // 1. Get Outgoing Links
    let outgoing_raw = if let Ok(n) = storage::load_note(&state.data_dir, &id) {
        n.links
    } else if let Ok(b) = storage::load_bookmark(&state.data_dir, &id) {
        b.links
    } else if let Ok(t) = storage::load_task(&state.data_dir, &id) {
        t.links
    } else {
        Vec::new()
    };

    for link in outgoing_raw {
        if let Some((rel, target_id)) = link.split_once(':') {
            if let Ok(n) = storage::load_note(&state.data_dir, target_id) {
                outgoing.push(LinkItem {
                    id: n.id,
                    title: n.title,
                    entity_type: "note".to_string(),
                    rel: rel.to_string(),
                });
            } else if let Ok(b) = storage::load_bookmark(&state.data_dir, target_id) {
                outgoing.push(LinkItem {
                    id: b.id,
                    title: b.title,
                    entity_type: "bookmark".to_string(),
                    rel: rel.to_string(),
                });
            } else if let Ok(t) = storage::load_task(&state.data_dir, target_id) {
                outgoing.push(LinkItem {
                    id: t.id,
                    title: t.title,
                    entity_type: "task".to_string(),
                    rel: rel.to_string(),
                });
            }
        }
    }

    // 2. Get Incoming Links
    // Search all entities for links pointing to this ID
    if let Ok(notes) = storage::list_notes(&state.data_dir) {
        for n in notes {
            for link in n.links {
                if let Some((rel, target_id_in_link)) = link.split_once(':')
                    && (id.starts_with(target_id_in_link) || target_id_in_link.starts_with(&id))
                {
                    incoming.push(LinkItem {
                        id: n.id.clone(),
                        title: n.title.clone(),
                        entity_type: "note".to_string(),
                        rel: rel.to_string(),
                    });
                }
            }
        }
    }
    if let Ok(bookmarks) = storage::list_bookmarks(&state.data_dir) {
        for b in bookmarks {
            for link in b.links {
                if let Some((rel, target_id_in_link)) = link.split_once(':')
                    && (id.starts_with(target_id_in_link) || target_id_in_link.starts_with(&id))
                {
                    incoming.push(LinkItem {
                        id: b.id.clone(),
                        title: b.title.clone(),
                        entity_type: "bookmark".to_string(),
                        rel: rel.to_string(),
                    });
                }
            }
        }
    }
    if let Ok(tasks) = storage::list_tasks(&state.data_dir) {
        for t in tasks {
            for link in t.links {
                if let Some((rel, target_id_in_link)) = link.split_once(':')
                    && (id.starts_with(target_id_in_link) || target_id_in_link.starts_with(&id))
                {
                    incoming.push(LinkItem {
                        id: t.id.clone(),
                        title: t.title.clone(),
                        entity_type: "task".to_string(),
                        rel: rel.to_string(),
                    });
                }
            }
        }
    }

    (StatusCode::OK, Json(LinksResponse { outgoing, incoming })).into_response()
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Only enforce authentication for API requests.
    // Frontend routes (including deep links) must remain public to load the UI
    // which then prompts for the token if missing.
    if !path.starts_with("/api/") {
        return next.run(req).await;
    }

    if let Some(expected) = &state.token {
        let mut authenticated = false;

        // 1. Check Header: X-ZNote-Token
        if let Some(token) = req.headers().get("X-ZNote-Token")
            && token.to_str().unwrap_or("") == expected
        {
            authenticated = true;
        }

        // 2. Check Header: Authorization: Bearer <token>
        if !authenticated
            && let Some(auth) = req.headers().get("Authorization")
            && let Ok(auth_str) = auth.to_str()
            && auth_str.starts_with("Bearer ")
            && &auth_str[7..] == expected
        {
            authenticated = true;
        }

        // 3. Check Query Param: token
        if !authenticated && let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let (Some("token"), Some(val)) = (parts.next(), parts.next())
                    && val == expected
                {
                    authenticated = true;
                    break;
                }
            }
        }

        if !authenticated {
            return (
                StatusCode::UNAUTHORIZED,
                "Unauthorized: ZNOTE_WEB_UI_TOKEN is required",
            )
                .into_response();
        }
    }

    next.run(req).await
}

async fn serve_index() -> impl IntoResponse {
    serve_asset(AxumPath("index.html".to_string())).await
}

async fn serve_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            // If asset not found, serve index.html for client-side routing.
            // In tests or if build is missing, return a simple placeholder instead of panicking.
            match Assets::get("index.html") {
                Some(index) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(index.data))
                    .unwrap(),
                None => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from("<html><body><h1>Znote UI</h1><p>UI assets not found. Please run <code>make ui-build</code>.</p></body></html>"))
                    .unwrap(),
            }
        }
    }
}

// --- API Handlers ---

#[tracing::instrument(skip_all)]
async fn api_list_notes(State(state): State<AppState>) -> impl IntoResponse {
    match storage::list_notes(&state.data_dir) {
        Ok(notes) => {
            let mut notes_with_starred: Vec<_> = notes
                .into_iter()
                .map(|n| {
                    let starred = is_starred(&n.tags);
                    serde_json::json!({
                        "id": n.id,
                        "title": n.title,
                        "content": n.content,
                        "tags": n.tags,
                        "links": n.links,
                        "created_at": n.created_at,
                        "updated_at": n.updated_at,
                        "starred": starred
                    })
                })
                .collect();

            notes_with_starred.sort_by(|a, b| {
                let a_starred = a["starred"].as_bool().unwrap_or(false);
                let b_starred = b["starred"].as_bool().unwrap_or(false);
                match (a_starred, b_starred) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let a_upd = a["updated_at"].as_str().unwrap_or("");
                        let b_upd = b["updated_at"].as_str().unwrap_or("");
                        b_upd.cmp(a_upd)
                    }
                }
            });

            (StatusCode::OK, Json(notes_with_starred)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_list_bookmarks(State(state): State<AppState>) -> impl IntoResponse {
    match storage::list_bookmarks(&state.data_dir) {
        Ok(bms) => {
            let mut bms_with_starred: Vec<_> = bms
                .into_iter()
                .map(|b| {
                    let starred = is_starred(&b.tags);
                    serde_json::json!({
                        "id": b.id,
                        "url": b.url,
                        "title": b.title,
                        "description": b.description,
                        "tags": b.tags,
                        "links": b.links,
                        "created_at": b.created_at,
                        "updated_at": b.updated_at,
                        "starred": starred
                    })
                })
                .collect();

            bms_with_starred.sort_by(|a, b| {
                let a_starred = a["starred"].as_bool().unwrap_or(false);
                let b_starred = b["starred"].as_bool().unwrap_or(false);
                match (a_starred, b_starred) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let a_upd = a["updated_at"].as_str().unwrap_or("");
                        let b_upd = b["updated_at"].as_str().unwrap_or("");
                        b_upd.cmp(a_upd)
                    }
                }
            });

            (StatusCode::OK, Json(bms_with_starred)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    match storage::list_tasks(&state.data_dir) {
        Ok(tasks) => {
            let mut tasks_with_starred: Vec<_> = tasks
                .into_iter()
                .map(|t| {
                    let starred = is_starred(&t.tags);
                    serde_json::json!({
                        "id": t.id,
                        "title": t.title,
                        "description": t.description,
                        "tags": t.tags,
                        "links": t.links,
                        "items": t.items,
                        "created_at": t.created_at,
                        "updated_at": t.updated_at,
                        "starred": starred
                    })
                })
                .collect();

            tasks_with_starred.sort_by(|a, b| {
                let a_starred = a["starred"].as_bool().unwrap_or(false);
                let b_starred = b["starred"].as_bool().unwrap_or(false);
                match (a_starred, b_starred) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let a_upd = a["updated_at"].as_str().unwrap_or("");
                        let b_upd = b["updated_at"].as_str().unwrap_or("");
                        b_upd.cmp(a_upd)
                    }
                }
            });

            (StatusCode::OK, Json(tasks_with_starred)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_list_tags(State(state): State<AppState>) -> impl IntoResponse {
    if crate::storage::is_dolt_backend() {
        let db = crate::storage::dolt::DoltStorage::new(&state.data_dir);
        let sql = "SELECT DISTINCT tag FROM tags ORDER BY tag";
        if let Ok(res) = db.run_sql(sql) {
            let mut tag_list = Vec::new();
            if let Some(rows) = res.get("rows").and_then(|r: &serde_json::Value| r.as_array()) {
                for r in rows {
                    if let Some(t) = r["tag"].as_str() {
                        tag_list.push(t.to_string());
                    }
                }
                return (StatusCode::OK, Json(tag_list)).into_response();
            }
        }
    }

    let mut tags = HashSet::new();

    if let Ok(notes) = storage::list_notes(&state.data_dir) {
        for n in notes {
            for t in n.tags {
                tags.insert(t);
            }
        }
    }

    if let Ok(bms) = storage::list_bookmarks(&state.data_dir) {
        for b in bms {
            for t in b.tags {
                tags.insert(t);
            }
        }
    }

    if let Ok(tasks) = storage::list_tasks(&state.data_dir) {
        for t in tasks {
            for tag in t.tags {
                tags.insert(tag);
            }
        }
    }

    let mut tag_list: Vec<String> = tags.into_iter().collect();
    tag_list.sort_by_key(|a| a.to_lowercase());

    (StatusCode::OK, Json(tag_list)).into_response()
}

#[tracing::instrument(skip_all)]
async fn api_get_note(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match storage::load_note(&state.data_dir, &id) {
        Ok(n) => {
            let starred = is_starred(&n.tags);
            let n_json = serde_json::json!({
                "id": n.id,
                "title": n.title,
                "content": render_wiki_links(&n.content),
                "tags": n.tags,
                "links": n.links,
                "created_at": n.created_at,
                "updated_at": n.updated_at,
                "starred": starred
            });
            (StatusCode::OK, Json(n_json)).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_resolve_id(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match storage::get_entity_type(&state.data_dir, &id) {
        Some((t, full_id)) => (StatusCode::OK, Json(serde_json::json!({ "type": t, "id": full_id }))).into_response(),
        None => (StatusCode::NOT_FOUND, "Entity not found").into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_get_bookmark(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match storage::load_bookmark(&state.data_dir, &id) {
        Ok(b) => {
            let starred = is_starred(&b.tags);
            let b_json = serde_json::json!({
                "id": b.id,
                "url": b.url,
                "title": b.title,
                "description": render_wiki_links(b.description.as_deref().unwrap_or("")),
                "tags": b.tags,
                "links": b.links,
                "created_at": b.created_at,
                "updated_at": b.updated_at,
                "starred": starred
            });
            (StatusCode::OK, Json(b_json)).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_get_task(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match storage::load_task(&state.data_dir, &id) {
        Ok(t) => {
            let starred = is_starred(&t.tags);
            let t_json = serde_json::json!({
                "id": t.id,
                "title": t.title,
                "description": render_wiki_links(t.description.as_deref().unwrap_or("")),
                "tags": t.tags,
                "links": t.links,
                "items": t.items,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
                "starred": starred
            });
            (StatusCode::OK, Json(t_json)).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct SearchParams {
    q: String,
}

#[tracing::instrument(skip_all)]
async fn api_search(
    axum::extract::Query(params): axum::extract::Query<SearchParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match storage::list_notes(&state.data_dir) {
        Ok(notes) => {
            let filtered: Vec<_> = notes
                .into_iter()
                .filter(|n| n.title.contains(&params.q) || n.content.contains(&params.q))
                .collect();
            (StatusCode::OK, Json(filtered)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct QueryParams {
    expr: String,
}

#[derive(serde::Serialize)]
struct UnifiedEntity {
    id: String,
    title: String,
    tags: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "type")]
    entity_type: String,
    url: Option<String>,
    starred: bool,
}

fn is_starred(tags: &[String]) -> bool {
    let starred_tag = env::var("ZNOTE_STARRED").unwrap_or_else(|_| "#starred".to_string());
    tags.iter().any(|t| t == &starred_tag)
}

#[tracing::instrument(skip_all)]
async fn api_query(
    axum::extract::Query(params): axum::extract::Query<QueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match query::find_files(&state.data_dir, &params.expr) {
        Ok(files) => {
            let mut results = Vec::new();
            for file in files {
                let mut parts = file.splitn(2, '/');
                let subdir = parts.next().unwrap_or("");
                let filename = parts.next().unwrap_or(&file);
                let id = filename.trim_end_matches(".md");

                match subdir {
                    "notes" => {
                        if let Ok(n) = storage::load_note(&state.data_dir, id) {
                            let starred = is_starred(&n.tags);
                            results.push(UnifiedEntity {
                                id: n.id,
                                title: n.title,
                                tags: n.tags,
                                created_at: n.created_at,
                                updated_at: n.updated_at,
                                entity_type: "note".to_string(),
                                url: None,
                                starred,
                            });
                        }
                    }
                    "bookmarks" => {
                        if let Ok(b) = storage::load_bookmark(&state.data_dir, id) {
                            let starred = is_starred(&b.tags);
                            results.push(UnifiedEntity {
                                id: b.id,
                                title: b.title,
                                tags: b.tags,
                                created_at: b.created_at,
                                updated_at: b.updated_at,
                                entity_type: "bookmark".to_string(),
                                url: Some(b.url),
                                starred,
                            });
                        }
                    }
                    "tasks" => {
                        if let Ok(t) = storage::load_task(&state.data_dir, id) {
                            let starred = is_starred(&t.tags);
                            results.push(UnifiedEntity {
                                id: t.id,
                                title: t.title,
                                tags: t.tags,
                                created_at: t.created_at,
                                updated_at: t.updated_at,
                                entity_type: "task".to_string(),
                                url: None,
                                starred,
                            });
                        }
                    }
                    _ => {}
                }
            }
            results.sort_by(|a, b| match (a.starred, b.starred) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            });
            (StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[tracing::instrument(skip_all)]
async fn api_graph(State(state): State<AppState>) -> impl IntoResponse {
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    let mut valid_ids = std::collections::BTreeSet::new();

    // 1. Load all nodes first to build the valid_ids set
    if let Ok(note_list) = storage::list_notes(&state.data_dir) {
        for n in note_list {
            nodes.push(serde_json::json!({
                "id": n.id,
                "name": n.title,
                "type": "note",
                "tags": n.tags,
                "links": n.links, // Keep these for processing
                "val": 10
            }));
            valid_ids.insert(n.id.clone());
        }
    }
    if let Ok(bm_list) = storage::list_bookmarks(&state.data_dir) {
        for b in bm_list {
            nodes.push(serde_json::json!({
                "id": b.id,
                "name": b.title,
                "type": "bookmark",
                "tags": b.tags,
                "links": b.links,
                "val": 10
            }));
            valid_ids.insert(b.id.clone());
        }
    }
    if let Ok(task_list) = storage::list_tasks(&state.data_dir) {
        for t in task_list {
            nodes.push(serde_json::json!({
                "id": t.id,
                "name": t.title,
                "type": "task",
                "tags": t.tags,
                "links": t.links,
                "val": 10
            }));
            valid_ids.insert(t.id.clone());
        }
    }

    // 2. Process links and resolve prefixes
    for node in &nodes {
        let source_id = node["id"].as_str().unwrap_or("");
        if let Some(links_array) = node["links"].as_array() {
            for link_val in links_array {
                if let Some(link_str) = link_val.as_str()
                    && let Some((rel, target_prefix)) = link_str.split_once(':')
                {
// Resolve prefix to full ID
                    let mut resolved_target = None;
                    if valid_ids.contains(target_prefix) {
                        resolved_target = Some(target_prefix.to_string());
                    } else {
                        // Find first ID that starts with target_prefix
                        use std::ops::Bound;
                        let mut range = valid_ids.range((Bound::Included(target_prefix.to_string()), Bound::Unbounded));
                        if let Some(found) = range.next() {
                            if found.starts_with(target_prefix) {
                                resolved_target = Some(found.clone());
                            }
                        }
                    }

                    if let Some(target_id) = resolved_target {
                        links.push(serde_json::json!({
                            "source": source_id,
                            "target": target_id,
                            "label": rel
                        }));
                    }
                }
            }
        }
    }

    // Cleanup: remove temporary labels from node objects before sending
    let clean_nodes: Vec<_> = nodes
        .into_iter()
        .map(|mut n| {
            if let Some(obj) = n.as_object_mut() {
                obj.remove("links");
            }
            n
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "nodes": clean_nodes, "links": links })),
    )
        .into_response()
}

fn render_wiki_links(content: &str) -> String {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"(?P<embed>!?)(?:\[\[)(?P<target>[^\]|#]+)(?:#(?P<header>[^\]|]+))?(?:\|(?P<alias>[^\]]+))?(?:\]\])").unwrap()
    });

    let mut result = String::new();
    let mut last_end = 0;

    for caps in re.captures_iter(content) {
        let m = caps.get(0).unwrap();
        result.push_str(&content[last_end..m.start()]);
        last_end = m.end();

        let is_embed = caps
            .name("embed")
            .map(|m| m.as_str() == "!")
            .unwrap_or(false);
        let target = caps.name("target").unwrap().as_str().trim();
        let header = caps.name("header").map(|m| m.as_str().trim());
        let alias = caps.name("alias").map(|m| m.as_str().trim());
        let label = alias.unwrap_or(target);

        if is_embed {
            // Keep it as ![[target#header]] for the frontend to handle
            if let Some(h) = header {
                result.push_str(&format!("![[{}#{}]]", target, h));
            } else {
                result.push_str(&format!("![[{}]]", target));
            }
        } else {
            // Convert WikiLink to Markdown link
            result.push_str(&format!("[{}]({})", label, target));
        }
    }

    result.push_str(&content[last_end..]);
    result
}
