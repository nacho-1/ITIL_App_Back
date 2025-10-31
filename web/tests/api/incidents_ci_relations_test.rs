use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    incidents::ci_relations::{self, IncidentCIRelation},
};
use itil_back_macros::db_test;
use itil_back_web::{
    controllers::incidents::ci_relations::{ModifyIncidentCIRelation, RelateCIRequest},
    test_helpers::{BodyExt, DbTestContext, RouterExt},
};
use serde_json::json;
use uuid::Uuid;

async fn post_ci(context: &DbTestContext) -> Uuid {
    let changeset = entities::configitems::ConfigItemCreateset {
        name: String::from("Testing CI for Relations with Incidentes"),
        status: Some(entities::configitems::CIStatus::Active),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        r#type: Some(String::from("Test CI")),
        owner: Some(String::from("Me")),
        description: String::from("I'm for testing"),
    };

    let ci = entities::configitems::create(changeset, &context.db_pool)
        .await
        .unwrap();

    ci.id
}

async fn post_incident(context: &DbTestContext) -> Uuid {
    let changeset = entities::incidents::IncidentChangeset {
        title: String::from("Testing Incident"),
        status: entities::incidents::IncidentStatus::InProgress,
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        resolved_at: None,
        impact: entities::incidents::IncidentImpact::Low,
        urgency: entities::incidents::IncidentUrgency::Low,
        owner: Some(String::from("Me")),
        description: String::from("Testing yay!!"),
    };

    let incident = entities::incidents::create(changeset, &context.db_pool)
        .await
        .unwrap();

    incident.id
}

#[db_test]
async fn test_create_invalid_bad_incident(context: &DbTestContext) {
    let ci_id = post_ci(context).await;

    let request = RelateCIRequest { ci_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_invalid_bad_ci(context: &DbTestContext) {
    let incident_id = post_incident(context).await;

    let request = RelateCIRequest {
        ci_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", incident_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_invalid_both_bad(context: &DbTestContext) {
    let request = RelateCIRequest {
        ci_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let incident_id = post_incident(context).await;
    let ci_id = post_ci(context).await;

    let request = RelateCIRequest { ci_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", incident_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));
    let relation = response.into_body().into_json::<IncidentCIRelation>().await;
    assert_that!(relation.incident_id, eq(incident_id));
    assert_eq!(relation.ci_id, ci_id);
    assert_eq!(relation.description, String::from(""));
}

#[db_test]
async fn test_read_all_nonexistent_incident(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", Uuid::new_v4()))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let incident_id = post_incident(context).await;
    let ci_id = post_ci(context).await;

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", incident_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<IncidentCIRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(0)));

    ci_relations::create(incident_id, ci_id, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/incidents/{}/configitems", incident_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<IncidentCIRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(1)));
    assert_that!(relations.first().unwrap().ci_id, eq(ci_id));
}

#[db_test]
async fn test_update_nonexistent_both_bad(context: &DbTestContext) {
    let payload = json!(ModifyIncidentCIRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            Uuid::new_v4(),
            Uuid::new_v4()
        ))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_nonexistent_bad_ci(context: &DbTestContext) {
    let incident_id = post_incident(context).await;

    let payload = json!(ModifyIncidentCIRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            incident_id,
            Uuid::new_v4()
        ))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let incident_id = post_incident(context).await;
    let ci_id = post_ci(context).await;
    ci_relations::create(incident_id, ci_id, &context.db_pool)
        .await
        .unwrap();

    let payload = json!(ModifyIncidentCIRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            incident_id, ci_id
        ))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let relation: IncidentCIRelation = response.into_body().into_json().await;
    assert_that!(relation.description, eq(&String::from("Update!")));
}

#[db_test]
async fn test_delete_nonexistent_both_bad(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            Uuid::new_v4(),
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_nonexistent_bad_ci(context: &DbTestContext) {
    let incident_id = post_incident(context).await;

    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            incident_id,
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let incident_id = post_incident(context).await;
    let ci_id = post_ci(context).await;
    ci_relations::create(incident_id, ci_id, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!(
            "/api/incidents/{}/configitems/{}",
            incident_id, ci_id
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));
    let relations = ci_relations::load_all(incident_id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(relations, len(eq(0)));
}
