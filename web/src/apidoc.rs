use itil_back_db::entities::incidents::IncidentPrio;
use utoipa::OpenApi;

pub const CONFIG_ITEMS_TAG: &str = "configitems";
pub const INCIDENTS_TAG: &str = "incidents";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = CONFIG_ITEMS_TAG, description = "Configuration Management Endpoints"),
        (name = INCIDENTS_TAG, description = "Incident Management Endpoints"),
    ),
    components(
        // Manually add the schema so it generates it.
        schemas(IncidentPrio,)
    )
)]
pub struct ApiDoc;
