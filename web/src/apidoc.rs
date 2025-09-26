use utoipa::OpenApi;

pub const CONFIG_ITEMS_TAG: &str = "configitems";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = CONFIG_ITEMS_TAG, description = "Configuration Management Endpoints"),
    )
)]
pub struct ApiDoc;
