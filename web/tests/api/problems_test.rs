use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::problems::{
    self, Problem, ProblemCreateset, ProblemStatus, ProblemUpdateset,
};
use itil_back_db::entity_helpers::PatchField;
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

fn create_basic_createset() -> ProblemCreateset {
    ProblemCreateset {
        title: String::from("Problem for Testing"),
        status: Some(ProblemStatus::Open),
        detection_timedate: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        description: String::from("This is a fake problem made for testing."),
        causes: String::from("I need to test this."),
        workarounds: Some(String::from("docs.local/workarounds/testing.pdf")),
        resolutions: Some(String::from("docs.local/resolutions/testing.pdf")),
    }
}

fn create_basic_updateset() -> ProblemUpdateset {
    ProblemUpdateset {
        title: Some(String::from("Problem for Testing")),
        status: Some(ProblemStatus::Open),
        detection_timedate: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        description: Some(String::from("This is a fake problem made for testing.")),
        causes: Some(String::from("I need to test this.")),
        workarounds: PatchField::Value(String::from("docs.local/workarounds/testing.pdf")),
        resolutions: PatchField::Value(String::from("docs.local/resolutions/testing.pdf")),
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let changeset = create_basic_createset();
    // Set of changset with invalid values (only too few characters)
    let mut sets = Vec::new();
    sets.push(ProblemCreateset {
        title: String::from(""),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        title: String::from(&"x".repeat(256)),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        description: String::from(&"x".repeat(1025)),
        ..changeset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/problems")
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
    let changeset = create_basic_createset();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/api/problems")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let problems = problems::load_all(&context.db_pool).await.unwrap();
    assert_that!(problems, len(eq(1)));
    assert_that!(problems.first().unwrap().title, eq(&changeset.title));
}

#[db_test]
async fn test_create_border_success(context: &DbTestContext) {
    let changeset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(ProblemCreateset {
        status: None,
        detection_timedate: None,
        workarounds: None,
        resolutions: None,
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        title: String::from("x"),
        description: String::from(""),
        causes: String::from(""),
        workarounds: Some(String::from("")),
        resolutions: Some(String::from("")),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        title: String::from(&"x".repeat(255)),
        description: String::from(&"x".repeat(1024)),
        causes: String::from(&"x".repeat(1024)),
        workarounds: Some(String::from(&"x".repeat(1024))),
        resolutions: Some(String::from(&"x".repeat(1024))),
        ..changeset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/problems")
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
    let changeset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(ProblemCreateset {
        status: Some(ProblemStatus::Open),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        status: Some(ProblemStatus::KnownError),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        status: Some(ProblemStatus::Resolved),
        ..changeset.clone()
    });
    sets.push(ProblemCreateset {
        status: Some(ProblemStatus::Closed),
        ..changeset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/problems")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let problem = response.into_body().into_json::<Problem>().await;
        assert_that!(problem.status, eq(&set.status.unwrap()));
    }
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let changeset = create_basic_createset();
    problems::create(changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/api/problems").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let problems: Vec<Problem> = response.into_body().into_json::<Vec<Problem>>().await;
    assert_that!(problems, len(eq(1)));
    assert_that!(problems.first().unwrap().title, eq(&changeset.title));
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/problems/{}", Uuid::new_v4()))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let problem_changeset = create_basic_createset();
    let problem = problems::create(problem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let problem_id = problem.id;

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem_id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let problem: Problem = response.into_body().into_json::<Problem>().await;
    assert_that!(problem.id, eq(problem_id));
    assert_that!(problem.title, eq(&problem_changeset.title));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let problem_createset = create_basic_createset();
    let problem = problems::create(problem_createset.clone(), &context.db_pool)
        .await
        .unwrap();

    let problem_updateset = create_basic_updateset();
    let payload = json!(ProblemUpdateset {
        title: Some(String::from("")),
        ..problem_updateset.clone()
    });

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let problem_after = problems::load(problem.id, &context.db_pool).await.unwrap();
    assert_that!(problem_after.title, eq(&problem.title));
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let problem_changeset = create_basic_updateset();
    let payload = json!(problem_changeset);

    let response = context
        .app
        .request(&format!("/api/problems/{}", Uuid::new_v4()))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let problem_createset = create_basic_createset();
    let problem = problems::create(problem_createset.clone(), &context.db_pool)
        .await
        .unwrap();

    let problem_updateset = create_basic_updateset();
    let payload = json!(problem_updateset);

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let problem: Problem = response.into_body().into_json::<Problem>().await;
    assert_that!(problem.title, eq(&problem_updateset.title.clone().unwrap()));

    let problem = problems::load(problem.id, &context.db_pool).await.unwrap();
    assert_that!(problem.title, eq(&problem_updateset.title.unwrap()));
}

#[db_test]
async fn test_update_nothing(context: &DbTestContext) {
    let problem_createset = create_basic_createset();
    let problem = problems::create(problem_createset.clone(), &context.db_pool)
        .await
        .unwrap();

    let problem_updateset = ProblemUpdateset {
        title: None,
        status: None,
        detection_timedate: None,
        description: None,
        causes: None,
        workarounds: PatchField::Missing,
        resolutions: PatchField::Missing,
    };
    let payload = json!(problem_updateset);

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let problem_after: Problem = response.into_body().into_json::<Problem>().await;
    assert_that!(problem_after.title, eq(&problem.title));
    assert!(problem_after.workarounds.is_some());

    let problem_after = problems::load(problem.id, &context.db_pool).await.unwrap();
    assert_that!(problem_after.title, eq(&problem.title));
}

#[db_test]
async fn test_update_set_nulls(context: &DbTestContext) {
    let problem_createset = create_basic_createset();
    let problem = problems::create(problem_createset.clone(), &context.db_pool)
        .await
        .unwrap();
    assert!(problem.workarounds.is_some());
    assert!(problem.resolutions.is_some());

    let problem_updateset = ProblemUpdateset {
        workarounds: PatchField::Null,
        resolutions: PatchField::Null,
        ..create_basic_updateset()
    };
    let payload = json!(problem_updateset);

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let problem_after: Problem = response.into_body().into_json::<Problem>().await;
    assert_that!(problem_after.title, eq(&problem.title));
    assert!(problem_after.workarounds.is_none());
    assert!(problem_after.resolutions.is_none());
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/problems/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let problem_changeset = create_basic_createset();
    let problem = problems::create(problem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/problems/{}", problem.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = problems::load(problem.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
