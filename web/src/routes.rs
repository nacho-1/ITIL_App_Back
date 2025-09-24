use crate::{controllers::configitems, state::AppState};
use axum::{routing, Router};

use std::sync::Arc;

/// Initializes the application's routes.
///
/// This function maps paths (e.g. "/greet") and HTTP methods (e.g. "GET") to functions in [`crate::controllers`] as well as includes middlewares defined in [`crate::middlewares`] into the routing layer (see [`axum::Router`]).
pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);
    Router::new()
        .route("/configitems", routing::post(configitems::create))
        .route("/configitems", routing::get(configitems::read_all))
        .route("/configitems/{id}", routing::get(configitems::read_one))
        .route("/configitems/{id}", routing::put(configitems::update))
        .route("/configitems/{id}", routing::delete(configitems::delete))
        .with_state(shared_app_state)
}
