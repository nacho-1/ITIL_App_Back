use crate::{
    apidoc::ApiDoc,
    controllers::{
        configitems, health,
        incidents::{self, ci_relations},
        problems,
    },
    state::AppState,
};
use axum::Router;
use tower_http::cors::CorsLayer;
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
        .nest("/api/incidents", incidents_router())
        .nest("/api/configitems", configitems_router())
        .nest("/api/problems", problems_router())
        .with_state(shared_app_state)
        .split_for_parts();

    let cors = CorsLayer::permissive();
    router
        .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
        .layer(cors)
}

// Important: Controllers must have different names even if they are in different modules.
// Otherwise utoipa gets dizzy.
// See https://github.com/juhaku/utoipa/issues/1298

fn configitems_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(configitems::create_ci, configitems::read_all_ci,))
        .routes(routes!(
            configitems::read_one_ci,
            configitems::update_ci,
            configitems::delete_ci,
        ))
}

fn incidents_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(
            incidents::create_incident,
            incidents::read_all_incidents,
        ))
        .routes(routes!(
            incidents::read_one_incident,
            incidents::update_incident,
            incidents::delete_incident,
        ))
        .routes(routes!(
            ci_relations::create_incident_ci_relation,
            ci_relations::read_all_incident_ci_relations,
        ))
        .routes(routes!(
            ci_relations::update_incident_ci_relation,
            ci_relations::delete_incident_ci_relation,
        ))
}

fn problems_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(
            problems::create_problem,
            problems::read_all_problems,
        ))
        .routes(routes!(
            problems::read_one_problem,
            problems::update_problem,
            problems::delete_problem,
        ))
}
