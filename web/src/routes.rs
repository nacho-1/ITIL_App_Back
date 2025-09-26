use crate::{
    apidoc::ApiDoc,
    controllers::{configitems, health},
    state::AppState,
};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use std::sync::Arc;

/// Initializes the application's routes.
///
/// This function maps paths (e.g. "/greet") and HTTP methods (e.g. "GET") to functions in [`crate::controllers`] as well as includes middlewares defined in [`crate::middlewares`] into the routing layer (see [`axum::Router`]).
pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .nest("/api/configitems", configitems_router())
        .with_state(shared_app_state)
        .split_for_parts();

    let cors = CorsLayer::new().allow_origin(Any);
    router
        .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
        .layer(cors)
}

pub fn configitems_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(configitems::create, configitems::read_all,))
        .routes(routes!(
            configitems::read_one,
            configitems::update,
            configitems::delete,
        ))
}
