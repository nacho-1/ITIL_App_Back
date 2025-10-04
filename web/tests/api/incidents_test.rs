use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities;
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

/// Create basic fake changeset for testing.
/// Values are border maximum as to test system's behaviour.
fn create_basic_changeset() -> entities::incidents::IncidentChangeset {
    let mut title = String::from("Testing Incident - ");
    title.push_str(&"x".repeat(255 - title.len()));
    let mut owner_content = String::from("Testing Department - ");
    owner_content.push_str(&"x".repeat(63 - owner_content.len()));
    let mut description = String::from("This is a fictional item made for testing. ");
    description.push_str(&"x".repeat(255 - description.len()));

    entities::incidents::IncidentChangeset {
        title,
        status: entities::incidents::IncidentStatus::InProgress,
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        impact: entities::incidents::IncidentImpact::Low,
        urgency: entities::incidents::IncidentUrgency::Low,
        owner: Some(owner_content),
        description,
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    // Set of changset with invalid values (only too few characters)
    let mut sets = Vec::new();
    sets.push(entities::incidents::IncidentChangeset {
        title: String::from(""),
        ..changeset.clone()
    });
    sets.push(entities::incidents::IncidentChangeset {
        owner: Some(String::from("")),
        ..changeset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/incidents")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
    }
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/api/incidents")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let incidents = entities::incidents::load_all(&context.db_pool)
        .await
        .unwrap();
    assert_that!(incidents, len(eq(1)));
    assert_that!(incidents.first().unwrap().title, eq(&changeset.title));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    entities::incidents::create(changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/api/incidents").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incidents: Vec<entities::incidents::Incident> = response
        .into_body()
        .into_json::<Vec<entities::incidents::Incident>>()
        .await;
    assert_that!(incidents, len(eq(1)));
    assert_that!(incidents.first().unwrap().title, eq(&changeset.title));
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/incidents/{}", Uuid::new_v4()))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let incident_changeset = create_basic_changeset();
    let incident = entities::incidents::create(incident_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let incident_id = incident.id;

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident_id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incident: entities::incidents::Incident = response
        .into_body()
        .into_json::<entities::incidents::Incident>()
        .await;
    assert_that!(incident.id, eq(incident_id));
    assert_that!(incident.title, eq(&incident_changeset.title));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let incident_changeset = create_basic_changeset();
    let incident = entities::incidents::create(incident_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(entities::incidents::IncidentChangeset {
        title: String::from(""),
        ..incident_changeset.clone()
    });

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let incident_after = entities::incidents::load(incident.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(incident_after.title, eq(&incident.title));
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let incident_changeset = create_basic_changeset();
    let payload = json!(incident_changeset);

    let response = context
        .app
        .request(&format!("/api/incidents/{}", Uuid::new_v4()))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let incident_changeset = create_basic_changeset();
    let incident = entities::incidents::create(incident_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let incident_changeset = entities::incidents::IncidentChangeset {
        title: String::from("New Title for Testing Incident"),
        ..incident_changeset
    };
    let payload = json!(incident_changeset);

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incident: entities::incidents::Incident = response
        .into_body()
        .into_json::<entities::incidents::Incident>()
        .await;
    assert_that!(incident.title, eq(&incident_changeset.title.clone()));

    let incident = entities::incidents::load(incident.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(incident.title, eq(&incident_changeset.title));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/incidents/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let incident_changeset = create_basic_changeset();
    let incident = entities::incidents::create(incident_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = entities::incidents::load(incident.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
