use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    configuration::changes::{self, CIChange, CIChangeCreateset, CIChangeUpdateset},
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

async fn post_ci(context: &DbTestContext) -> Uuid {
    let changeset = entities::configuration::ConfigItemCreateset {
        name: String::from("Testing CI for Relations with Incidentes"),
        status: Some(entities::configuration::CIStatus::Active),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        r#type: Some(String::from("Test CI")),
        owner: Some(String::from("Me")),
        description: String::from("I'm for testing"),
    };

    let ci = entities::configuration::create(changeset, &context.db_pool)
        .await
        .unwrap();

    ci.id
}

fn create_basic_createset() -> CIChangeCreateset {
    CIChangeCreateset {
        implementation_timedate: "2023-09-15T12:34:56Z".parse().unwrap(),
        documentation: String::from("docs.local/testing/001.pdf"),
    }
}

fn create_basic_updateset() -> CIChangeUpdateset {
    CIChangeUpdateset {
        implementation_timedate: Some(Some("2023-09-14T12:34:36Z".parse().unwrap())),
        documentation: Some(Some(String::from("docs.local/testing/002.pdf"))),
    }
}

#[db_test]
async fn test_create_invalid_bad_ci(context: &DbTestContext) {
    let createset = create_basic_createset();

    let payload = json!(createset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes", Uuid::new_v4()))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let ci_id = post_ci(context).await;

    let createset = create_basic_createset();

    let mut sets = Vec::new();
    sets.push(CIChangeCreateset {
        documentation: String::from(&"x".repeat(1025)),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/configitems/{}/changes", ci_id))
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
    let ci_id = post_ci(context).await;

    let createset = create_basic_createset();

    let payload = json!(createset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes", ci_id))
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));
    let change = response.into_body().into_json::<CIChange>().await;
    assert_that!(change.ci_id, eq(ci_id));
    assert_that!(
        change.implementation_timedate,
        eq(createset.implementation_timedate)
    );
    assert_that!(change.documentation, eq(&createset.documentation));
}

#[db_test]
async fn test_read_all_nonexistent_ci(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes", Uuid::new_v4()))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let ci_id = post_ci(context).await;

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes", ci_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let changes: Vec<CIChange> = response.into_body().into_json().await;
    assert_that!(changes, len(eq(0)));

    let createset = create_basic_createset();
    changes::create(ci_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes", ci_id))
        .send()
        .await;
    assert_that!(response.status(), eq(StatusCode::OK));
    let changes: Vec<CIChange> = response.into_body().into_json().await;
    assert_that!(changes, len(eq(1)));
    assert_that!(changes.first().unwrap().ci_id, eq(ci_id));
}

#[db_test]
async fn test_update_nonexistent_bad_change(context: &DbTestContext) {
    let ci_id = post_ci(context).await;

    let payload = json!(create_basic_updateset());

    let response = context
        .app
        .request(&format!(
            "/api/configitems/{}/changes/{}",
            ci_id,
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
    let payload = json!(create_basic_updateset());

    let response = context
        .app
        .request(&format!(
            "/api/configitems/{}/changes/{}",
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
async fn test_update_invalid_nulls(context: &DbTestContext) {
    let ci_id = post_ci(context).await;
    let createset = create_basic_createset();
    let change = changes::create(ci_id, createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(CIChangeUpdateset {
        implementation_timedate: Some(None),
        ..updateset.clone()
    });
    sets.push(CIChangeUpdateset {
        documentation: Some(None),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/configitems/{}/changes/{}", ci_id, change.id))
            .method(Method::PUT)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
    }
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let ci_id = post_ci(context).await;
    let createset = create_basic_createset();
    let change = changes::create(ci_id, createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes/{}", ci_id, change.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let change: CIChange = response.into_body().into_json::<CIChange>().await;
    assert_that!(
        change.implementation_timedate,
        eq(updateset.implementation_timedate.unwrap().unwrap())
    );
    assert_that!(
        change.documentation,
        eq(&updateset.documentation.unwrap().unwrap())
    );

    let change_after = changes::load(change.id, ci_id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(change_after, eq(&change));
}

#[db_test]
async fn test_update_nothing(context: &DbTestContext) {
    let ci_id = post_ci(context).await;
    let createset = create_basic_createset();
    let change_before = changes::create(ci_id, createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = CIChangeUpdateset {
        implementation_timedate: None,
        documentation: None,
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!(
            "/api/configitems/{}/changes/{}",
            ci_id, change_before.id
        ))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let change_after: CIChange = response.into_body().into_json::<CIChange>().await;
    assert_that!(change_after, eq(&change_before));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!(
            "/api/configitems/{}/changes/{}",
            Uuid::new_v4(),
            Uuid::new_v4()
        ))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let ci_id = post_ci(context).await;
    let createset = create_basic_createset();
    let change = changes::create(ci_id, createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/configitems/{}/changes/{}", ci_id, change.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = changes::load(change.id, ci_id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
