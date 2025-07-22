use std::{collections::HashMap, path::PathBuf, time::Duration};

use axum::{
    extract::{Query, WebSocketUpgrade},
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    Json, Router,
};
use error::Error;
use log::{info, trace};
use reload::reload_req;
use search::{Body, SearchResult};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::{pff::PffManager, reload::AutoReload, search::SearchClient};

mod error;
mod pff;
mod reload;
mod search;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Config {
    pff_file: PathBuf,
    listen_url: Option<String>,
    search_endpoint: String,
    search_api_key: String,
    search_index_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let config = envy::prefixed("PFF_WEB_").from_env::<Config>()?;
    trace!("App Config: {:#?}", config);

    let pff_manager = PffManager::new(config.pff_file);

    let auto_reload = AutoReload::new();
    let search_client = SearchClient::new(
        config.search_endpoint,
        config.search_api_key,
        config.search_index_name,
    )?;

    let app = Router::new()
        .fallback(get_service(ServeDir::new("www")))
        .route(
            "/search",
            get({
                let search_client = search_client.clone();
                move |query| handle_search(query, search_client)
            }),
        )
        .route(
            "/locate-message",
            get({
                let pff_manager = pff_manager.clone();
                move |query| handle_locate_message(query, pff_manager)
            }),
        )
        .route(
            "/show",
            get({
                let pff_manager = pff_manager.clone();
                move |query| handle_show_message(query, pff_manager)
            }),
        )
        .route(
            "/reload-notify",
            get({
                let auto_reload = auto_reload.clone();
                move || async move { auto_reload.notify() }
            }),
        )
        .route(
            "/reload",
            get({
                let auto_reload = auto_reload.clone();
                move |ws: WebSocketUpgrade| reload_req(ws, auto_reload)
            }),
        );

    let addr = config
        .listen_url
        .unwrap_or_else(|| "0.0.0.0:8800".to_string());

    info!("Listening on URL http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_search(
    Query(mut params): Query<HashMap<String, String>>,
    client: SearchClient,
) -> Result<Json<SearchResult>, Error> {
    let query = params.remove("q").unwrap_or_default();
    let results = client.search(query).await?;

    Ok(Json(results))
}

async fn handle_locate_message(
    Query(mut params): Query<HashMap<String, String>>,
    pff_manager: PffManager,
) -> Result<Json<Body>, StatusCode> {
    if let Some(id) = params.remove("id") {
        pff_manager
            .get_body(id, Duration::from_secs(1))
            .await
            .map(Json)
            .map_err(|err| match err {
                Error::BodyNotFound => StatusCode::NOT_FOUND,
                Error::BodyTimeout => StatusCode::REQUEST_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn handle_show_message(
    Query(mut params): Query<HashMap<String, String>>,
    pff_manager: PffManager,
) -> Result<Html<String>, StatusCode> {
    if let Some(id) = params.remove("id") {
        let body = pff_manager
            .get_body(id, Duration::from_secs(1))
            .await
            .map_err(|err| match err {
                Error::BodyNotFound => StatusCode::NOT_FOUND,
                Error::BodyTimeout => StatusCode::REQUEST_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })?;
        trace!("body.value = {}", body.value);
        Ok(Html(body.value))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
