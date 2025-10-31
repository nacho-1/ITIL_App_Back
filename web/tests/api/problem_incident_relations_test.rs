use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    problems::incident_relations::{self, ProblemIncidentRelation},
};
use itil_back_macros::db_test;
use itil_back_web::{
    controllers::problems::incident_relations::{CreateIncidentRelation, UpdateIncidentRelation},
    test_helpers::{BodyExt, DbTestContext, RouterExt},
};
use serde_json::json;
use uuid::Uuid;

async fn post_incident(context: &DbTestContext) -> Uuid {
    let createset = entities::incidents::IncidentChangeset {
        title: String::from("Testing Incident"),
        status: entities::incidents::IncidentStatus::InProgress,
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        resolved_at: None,
        impact: entities::incidents::IncidentImpact::Low,
        urgency: entities::incidents::IncidentUrgency::Low,
        owner: Some(String::from("Me")),
        description: String::from("Testing yay!!"),
    };

    let incident = entities::incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    incident.id
}

async fn post_problem(context: &DbTestContext) -> Uuid {
    let createset = entities::problems::ProblemCreateset {
        title: String::from("Problem for Testing"),
        status: Some(entities::problems::ProblemStatus::Open),
        detection_timedate: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        description: String::from("This is a fake problem made for testing."),
        causes: String::from("I need to test this."),
        workarounds: Some(String::from("docs.local/workarounds/testing.pdf")),
        resolutions: Some(String::from("docs.local/resolutions/testing.pdf")),
    };

    let problem = entities::problems::create(createset, &context.db_pool)
        .await
        .unwrap();

    problem.id
}

#[db_test]
async fn test_create_invalid_bad_problem(context: &DbTestContext) {
    let incident_id = post_incident(context).await;

    let request = CreateIncidentRelation { incident_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_invalid_bad_incident(context: &DbTestContext) {
    let problem_id = post_problem(context).await;

    let request = CreateIncidentRelation {
        incident_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", problem_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_invalid_both_bad(context: &DbTestContext) {
    let request = CreateIncidentRelation {
        incident_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let problem_id = post_problem(context).await;
    let incident_id = post_incident(context).await;

    let request = CreateIncidentRelation { incident_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", problem_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));
    let relation = response
        .into_body()
        .into_json::<ProblemIncidentRelation>()
        .await;
    assert_that!(relation.problem_id, eq(problem_id));
    assert_that!(relation.incident_id, eq(incident_id));
    assert_that!(relation.description, eq(&String::from("")));
}

#[db_test]
async fn test_read_all_nonexistent_problem(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", Uuid::new_v4()))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let problem_id = post_problem(context).await;
    let incident_id = post_incident(context).await;

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", problem_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<ProblemIncidentRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(0)));

    incident_relations::create(problem_id, incident_id, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/problems/{}/incidents", problem_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<ProblemIncidentRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(1)));
    assert_that!(relations.first().unwrap().incident_id, eq(incident_id));
}

#[db_test]
async fn test_update_nonexistent_both_bad(context: &DbTestContext) {
    let payload = json!(UpdateIncidentRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
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
async fn test_update_nonexistent_bad_incidents(context: &DbTestContext) {
    let problem_id = post_problem(context).await;

    let payload = json!(UpdateIncidentRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
            problem_id,
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
    let problem_id = post_problem(context).await;
    let incident_id = post_incident(context).await;
    incident_relations::create(problem_id, incident_id, &context.db_pool)
        .await
        .unwrap();

    let payload = json!(UpdateIncidentRelation {
        description: String::from("Update!"),
    });

    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
            problem_id, incident_id
        ))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let relation: ProblemIncidentRelation = response.into_body().into_json().await;
    assert_that!(relation.description, eq(&String::from("Update!")));
}

#[db_test]
async fn test_delete_nonexistent_both_bad(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
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
    let problem_id = post_problem(context).await;

    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
            problem_id,
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let problem_id = post_problem(context).await;
    let incident_id = post_incident(context).await;
    incident_relations::create(problem_id, incident_id, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!(
            "/api/problems/{}/incidents/{}",
            problem_id, incident_id
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));
    let relations = incident_relations::load_all(problem_id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(relations, len(eq(0)));
}
