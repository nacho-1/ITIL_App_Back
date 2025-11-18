use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    changes::incident_relations::{self, RFCIncidentCreateset, RFCIncidentRelation},
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

async fn post_incident(context: &DbTestContext) -> Uuid {
    let createset = entities::incidents::IncidentCreateset {
        title: String::from("Testing Incident"),
        status: Some(entities::incidents::IncidentStatus::InProgress),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        resolved_at: None,
        impact: entities::incidents::IncidentImpact::Low,
        urgency: entities::incidents::IncidentUrgency::Low,
        owner: Some(String::from("Me")),
        asignee: Some(String::from("Employee 567")),
        description: String::from("Testing yay!!"),
    };

    let incident = entities::incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    incident.id
}

async fn post_rfc(context: &DbTestContext) -> Uuid {
    let createset = entities::changes::RFCCreateset {
        title: String::from("RFC for Testing"),
        status: Some(entities::changes::RFCStatus::Open),
        created_at: None,
        finished_at: None,
        requester: String::from("Me the dev"),
        description: String::from("This is a fake rfc made for testing."),
    };

    let rfc = entities::changes::create(createset, &context.db_pool)
        .await
        .unwrap();

    rfc.id
}

#[db_test]
async fn test_create_invalid_bad_rfc(context: &DbTestContext) {
    let incident_id = post_incident(context).await;

    let request = RFCIncidentCreateset { incident_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_invalid_bad_incident(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;

    let request = RFCIncidentCreateset {
        incident_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", rfc_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_invalid_both_bad(context: &DbTestContext) {
    let request = RFCIncidentCreateset {
        incident_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;
    let incident_id = post_incident(context).await;

    let request = RFCIncidentCreateset { incident_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", rfc_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));
    let relation = response
        .into_body()
        .into_json::<RFCIncidentRelation>()
        .await;
    assert_that!(relation.rfc_id, eq(rfc_id));
    assert_that!(relation.incident_id, eq(incident_id));
}

#[db_test]
async fn test_read_all_nonexistent_rfc(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", Uuid::new_v4()))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;
    let incident_id = post_incident(context).await;

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", rfc_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<RFCIncidentRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(0)));

    let createset = RFCIncidentCreateset { incident_id };
    incident_relations::create(rfc_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/changes/{}/incidents", rfc_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<RFCIncidentRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(1)));
    assert_that!(relations.first().unwrap().incident_id, eq(incident_id));
}

#[db_test]
async fn test_delete_nonexistent_both_bad(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!(
            "/api/changes/{}/incidents/{}",
            Uuid::new_v4(),
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_nonexistent_bad_incident(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;

    let response = context
        .app
        .request(&format!(
            "/api/changes/{}/incidents/{}",
            rfc_id,
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;
    let incident_id = post_incident(context).await;
    let createset = RFCIncidentCreateset { incident_id };
    let relation = incident_relations::create(rfc_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!(
            "/api/changes/{}/incidents/{}",
            rfc_id, relation.id
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));
    let relations = incident_relations::load_all(rfc_id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(relations, len(eq(0)));
}
