use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    changes::problem_relations::{self, RFCProblemCreateset, RFCProblemRelation},
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

async fn post_problem(context: &DbTestContext) -> Uuid {
    let createset = entities::problems::ProblemCreateset {
        title: String::from("Testing Problem"),
        status: Some(entities::problems::ProblemStatus::Open),
        detection_timedate: None,
        description: String::from("Testing yay!!"),
        causes: String::from("Who knows..."),
        workarounds: None,
        resolutions: None,
    };

    let problem = entities::problems::create(createset, &context.db_pool)
        .await
        .unwrap();

    problem.id
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
    let problem_id = post_problem(context).await;

    let request = RFCProblemCreateset { problem_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_invalid_bad_problem(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;

    let request = RFCProblemCreateset {
        problem_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", rfc_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_invalid_both_bad(context: &DbTestContext) {
    let request = RFCProblemCreateset {
        problem_id: Uuid::new_v4(),
    };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", Uuid::new_v4()))
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
    let problem_id = post_problem(context).await;

    let request = RFCProblemCreateset { problem_id };

    let payload = json!(request);

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", rfc_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));
    let relation = response.into_body().into_json::<RFCProblemRelation>().await;
    assert_that!(relation.rfc_id, eq(rfc_id));
    assert_that!(relation.problem_id, eq(problem_id));
}

#[db_test]
async fn test_read_all_nonexistent_rfc(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", Uuid::new_v4()))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;
    let problem_id = post_problem(context).await;

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", rfc_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<RFCProblemRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(0)));

    let createset = RFCProblemCreateset { problem_id };
    problem_relations::create(rfc_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems", rfc_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let relations: Vec<RFCProblemRelation> = response.into_body().into_json().await;
    assert_that!(relations, len(eq(1)));
    assert_that!(relations.first().unwrap().problem_id, eq(problem_id));
}

#[db_test]
async fn test_delete_nonexistent_both_bad(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!(
            "/api/changes/{}/problems/{}",
            Uuid::new_v4(),
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_nonexistent_bad_problem(context: &DbTestContext) {
    let rfc_id = post_rfc(context).await;

    let response = context
        .app
        .request(&format!(
            "/api/changes/{}/problems/{}",
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
    let problem_id = post_problem(context).await;
    let createset = RFCProblemCreateset { problem_id };
    let relation = problem_relations::create(rfc_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/changes/{}/problems/{}", rfc_id, relation.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));
    let relations = problem_relations::load_all(rfc_id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(relations, len(eq(0)));
}
