use axum::{
    body::Body,
    http::{self, Method},
};
use chrono::{Duration, Utc};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    changes::{self, RFCCreateset, RFCStatus, RFCUpdateset, RFC},
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

fn create_basic_createset() -> RFCCreateset {
    RFCCreateset {
        title: String::from("Testing RFC"),
        status: Some(RFCStatus::Open),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        finished_at: Some("2023-10-15T12:34:50Z".parse().unwrap()),
        requester: String::from("Testing Department"),
        description: String::from("This is a fictional RFC made for testing."),
    }
}

fn create_basic_updateset() -> RFCUpdateset {
    RFCUpdateset {
        title: Some(Some(String::from("Updated RFC"))),
        status: Some(Some(RFCStatus::InProgress)),
        created_at: Some(Some("2023-09-15T12:34:58Z".parse().unwrap())),
        finished_at: Some(Some("2023-11-15T12:34:58Z".parse().unwrap())),
        requester: Some(Some(String::from("Update Department"))),
        description: Some(Some(String::from(
            "This is a fictional RFC made for updating.",
        ))),
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(RFCCreateset {
        title: String::from(""),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        title: String::from(&"x".repeat(256)),
        ..createset.clone()
    });
    sets.push(entities::changes::RFCCreateset {
        requester: String::from(&"x".repeat(1025)),
        ..createset.clone()
    });
    sets.push(entities::changes::RFCCreateset {
        description: String::from(&"x".repeat(1025)),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/changes")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
    }
}

#[db_test]
async fn test_create_bad_payload(context: &DbTestContext) {
    let payload = "{}";

    let response = context
        .app
        .request("/api/changes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let payload = json!(createset);

    let response = context
        .app
        .request("/api/changes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let rfc: RFC = response.into_body().into_json::<RFC>().await;
    assert_that!(rfc.title, eq(&createset.title));
    assert_that!(rfc.status, eq(createset.status.unwrap()));
    assert_that!(rfc.created_at, eq(createset.created_at.unwrap()));
    assert_that!(rfc.finished_at, eq(createset.finished_at));
    assert_that!(rfc.requester, eq(&createset.requester));
    assert_that!(rfc.description, eq(&createset.description));

    let rfcs = changes::load_all(&context.db_pool).await.unwrap();
    assert_that!(rfcs, len(eq(1)));
}

#[db_test]
async fn test_create_no_creation_date(context: &DbTestContext) {
    let createset = RFCCreateset {
        created_at: None,
        ..create_basic_createset()
    };
    let payload = json!(createset);

    let t0 = Utc::now();
    let response = context
        .app
        .request("/api/changes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let rfc: RFC = response.into_body().into_json::<RFC>().await;
    let diff = (rfc.created_at - t0).num_seconds().abs();
    // Testing that diff is no bigger than 2 minutes, for putting a reasonable diff.
    assert_that!(diff, lt(Duration::seconds(120).num_seconds()));
}

#[db_test]
async fn test_create_border_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(RFCCreateset {
        title: String::from("x"),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        title: String::from(&"x".repeat(255)),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        status: None,
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        finished_at: None,
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        requester: String::from(""),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        requester: String::from(&"x".repeat(1024)),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        description: String::from(""),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        description: String::from(&"x".repeat(1024)),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/changes")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::CREATED));
    }
}

#[db_test]
async fn test_status(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(changes::RFCCreateset {
        status: Some(RFCStatus::Open),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        status: Some(RFCStatus::InProgress),
        ..createset.clone()
    });
    sets.push(RFCCreateset {
        status: Some(RFCStatus::Closed),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/changes")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let rfc = response.into_body().into_json::<RFC>().await;
        assert_that!(rfc.status, eq(set.status.unwrap()));
    }
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let response = context.app.request("/api/changes").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let rfcs: Vec<RFC> = response.into_body().into_json::<Vec<RFC>>().await;
    assert_that!(rfcs, len(eq(1)));
    assert_that!(rfcs.first().unwrap(), eq(&rfc));
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/changes/{}", Uuid::new_v4()))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let response = context
        .app
        .request(&format!("/api/changes/{}", &rfc.id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let rfc_read: RFC = response.into_body().into_json::<RFC>().await;
    assert_that!(rfc_read, eq(&rfc));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(RFCUpdateset {
        title: Some(Some(String::from(""))),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        title: Some(Some(String::from(&"x".repeat(256)))),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        requester: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        description: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/changes/{}", rfc.id))
            .method(Method::PUT)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

        let rfc_after = changes::load(rfc.id, &context.db_pool).await.unwrap();
        assert_that!(rfc_after, eq(&rfc));
    }
}

#[db_test]
async fn test_update_invalid_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(RFCUpdateset {
        title: Some(None),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        status: Some(None),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        created_at: Some(None),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        requester: Some(None),
        ..updateset.clone()
    });
    sets.push(RFCUpdateset {
        description: Some(None),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/changes/{}", rfc.id))
            .method(Method::PUT)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
    }
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let updateset = create_basic_updateset();
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/changes/{}", Uuid::new_v4()))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let updateset = create_basic_updateset();
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/changes/{}", rfc.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let rfc: RFC = response.into_body().into_json::<RFC>().await;
    assert_that!(rfc.title, eq(&updateset.title.unwrap().unwrap()));
    assert_that!(rfc.status, eq(updateset.status.unwrap().unwrap()));
    assert_that!(rfc.created_at, eq(updateset.created_at.unwrap().unwrap()));
    assert_that!(rfc.finished_at, eq(updateset.finished_at.unwrap()));
    assert_that!(rfc.requester, eq(&updateset.requester.unwrap().unwrap()));
    assert_that!(
        rfc.description,
        eq(&updateset.description.unwrap().unwrap())
    );

    let rfc_after = changes::load(rfc.id, &context.db_pool).await.unwrap();
    assert_that!(rfc_after, eq(&rfc));
}

#[db_test]
async fn test_update_set_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset, &context.db_pool).await.unwrap();

    let updateset = RFCUpdateset {
        finished_at: Some(None),
        ..create_basic_updateset()
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/changes/{}", rfc.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let rfc_after: RFC = response.into_body().into_json::<RFC>().await;
    assert!(rfc_after.finished_at.is_none());
}

#[db_test]
async fn test_update_nothing(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc_before = changes::create(createset, &context.db_pool).await.unwrap();

    let updateset = RFCUpdateset {
        title: None,
        status: None,
        created_at: None,
        finished_at: None,
        requester: None,
        description: None,
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/changes/{}", rfc_before.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let rfc_after: RFC = response.into_body().into_json::<RFC>().await;
    assert_that!(rfc_after, eq(&rfc_before));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/changes/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let rfc = changes::create(createset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/changes/{}", rfc.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = changes::load(rfc.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
