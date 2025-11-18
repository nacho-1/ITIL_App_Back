use crate::{
    apidoc::ApiDoc,
    controllers::{
        changes::{self},
        configuration, health,
        incidents::{self, ci_relations},
        problems::{self, incident_relations},
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
        .nest("/api/changes", changes_router())
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
        .routes(routes!(
            configuration::create_ci,
            configuration::read_all_ci,
        ))
        .routes(routes!(
            configuration::read_one_ci,
            configuration::update_ci,
            configuration::delete_ci,
        ))
        .routes(routes!(
            configuration::changes::create_ci_change,
            configuration::changes::read_all_ci_changes,
        ))
        .routes(routes!(
            configuration::changes::read_one_ci_change,
            configuration::changes::update_ci_change,
            configuration::changes::delete_ci_change,
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
        .routes(routes!(incidents::read_all_incidents_by_ci,))
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
        .routes(routes!(
            incident_relations::create_problem_incident_relation,
            incident_relations::read_all_problem_incident_relations,
        ))
        .routes(routes!(
            incident_relations::update_problem_incident_relation,
            incident_relations::delete_problem_incident_relation,
        ))
}

fn changes_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(changes::create_rfc, changes::read_all_rfcs,))
        .routes(routes!(
            changes::read_one_rfc,
            changes::update_rfc,
            changes::delete_rfc,
        ))
}
