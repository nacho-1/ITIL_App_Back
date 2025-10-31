use axum::{
    body::Body,
    http::{self, Method},
};
use chrono::{Duration, Utc};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{
    self,
    configuration::{self, CIStatus, ConfigItem, ConfigItemCreateset, ConfigItemUpdateset},
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

fn create_basic_createset() -> ConfigItemCreateset {
    ConfigItemCreateset {
        name: String::from("Testing Configuration Item"),
        status: Some(CIStatus::Maintenance),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        r#type: Some(String::from("Testing Item")),
        owner: Some(String::from("Testing Department")),
        description: String::from("This is a fictional item made for testing."),
    }
}

fn create_basic_updateset() -> ConfigItemUpdateset {
    ConfigItemUpdateset {
        name: Some(Some(String::from("Updated Configuration Item"))),
        status: Some(Some(CIStatus::Testing)),
        created_at: Some(Some("2023-09-15T12:34:58Z".parse().unwrap())),
        r#type: Some(Some(String::from("Updating Item"))),
        owner: Some(Some(String::from("Update Department"))),
        description: Some(Some(String::from(
            "This is a fictional item made for updating.",
        ))),
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(entities::configuration::ConfigItemCreateset {
        name: String::from(""),
        ..createset.clone()
    });
    sets.push(entities::configuration::ConfigItemCreateset {
        name: String::from(&"x".repeat(256)),
        ..createset.clone()
    });
    sets.push(entities::configuration::ConfigItemCreateset {
        r#type: Some(String::from(&"x".repeat(1025))),
        ..createset.clone()
    });
    sets.push(entities::configuration::ConfigItemCreateset {
        owner: Some(String::from(&"x".repeat(1025))),
        ..createset.clone()
    });
    sets.push(entities::configuration::ConfigItemCreateset {
        description: String::from(&"x".repeat(1025)),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/configitems")
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
        .request("/api/configitems")
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
        .request("/api/configitems")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let ci: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    assert_that!(ci.name, eq(&createset.name));
    assert_that!(ci.status, eq(createset.status.unwrap()));
    assert_that!(ci.created_at, eq(createset.created_at.unwrap()));
    assert_that!(ci.r#type, eq(&createset.r#type));
    assert_that!(ci.owner, eq(&createset.owner));
    assert_that!(ci.description, eq(&createset.description));

    let configitems = configuration::load_all(&context.db_pool).await.unwrap();
    assert_that!(configitems, len(eq(1)));
}

#[db_test]
async fn test_create_no_creation_date(context: &DbTestContext) {
    let createset = ConfigItemCreateset {
        created_at: None,
        ..create_basic_createset()
    };
    let payload = json!(createset);

    let t0 = Utc::now();
    let response = context
        .app
        .request("/api/configitems")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let ci: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    let diff = (ci.created_at - t0).num_seconds().abs();
    // Testing that diff is no bigger than 2 minutes, for putting a reasonable diff.
    assert_that!(diff, lt(Duration::seconds(120).num_seconds()));
}

#[db_test]
async fn test_create_border_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(ConfigItemCreateset {
        name: String::from("x"),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        name: String::from(&"x".repeat(255)),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        status: None,
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        r#type: None,
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        r#type: Some(String::from(&"x".repeat(1024))),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        r#type: Some(String::from("")),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        owner: None,
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        owner: Some(String::from("")),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        owner: Some(String::from(&"x".repeat(1024))),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        description: String::from(""),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        description: String::from(&"x".repeat(1024)),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/configitems")
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
    sets.push(configuration::ConfigItemCreateset {
        status: Some(CIStatus::Testing),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        status: Some(CIStatus::Active),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        status: Some(CIStatus::Inactive),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        status: Some(CIStatus::Retired),
        ..createset.clone()
    });
    sets.push(ConfigItemCreateset {
        status: Some(CIStatus::Maintenance),
        ..createset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request("/api/configitems")
            .method(Method::POST)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let ci = response.into_body().into_json::<ConfigItem>().await;
        assert_that!(ci.status, eq(set.status.unwrap()));
    }
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/api/configitems").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let configitems: Vec<ConfigItem> = response.into_body().into_json::<Vec<ConfigItem>>().await;
    assert_that!(configitems, len(eq(1)));
    assert_that!(configitems.first().unwrap(), eq(&ci));
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/configitems/{}", Uuid::new_v4()))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/configitems/{}", &ci.id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let ci_read: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    assert_that!(ci_read, eq(&ci));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(ConfigItemUpdateset {
        name: Some(Some(String::from(""))),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        name: Some(Some(String::from(&"x".repeat(256)))),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        r#type: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        owner: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        description: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/configitems/{}", ci.id))
            .method(Method::PUT)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

        let ci_after = configuration::load(ci.id, &context.db_pool).await.unwrap();
        assert_that!(ci_after, eq(&ci));
    }
}

#[db_test]
async fn test_update_invalid_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let configitem = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(ConfigItemUpdateset {
        name: Some(None),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        status: Some(None),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        created_at: Some(None),
        ..updateset.clone()
    });
    sets.push(ConfigItemUpdateset {
        description: Some(None),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/configitems/{}", configitem.id))
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
        .request(&format!("/api/configitems/{}", Uuid::new_v4()))
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
    let ci = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}", ci.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let ci: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    assert_that!(ci.name, eq(&updateset.name.unwrap().unwrap()));
    assert_that!(ci.status, eq(updateset.status.unwrap().unwrap()));
    assert_that!(ci.created_at, eq(updateset.created_at.unwrap().unwrap()));
    assert_that!(ci.r#type, eq(&updateset.r#type.unwrap()));
    assert_that!(ci.owner, eq(&updateset.owner.unwrap()));
    assert_that!(ci.description, eq(&updateset.description.unwrap().unwrap()));

    let ci_after = configuration::load(ci.id, &context.db_pool).await.unwrap();
    assert_that!(ci_after, eq(&ci));
}

#[db_test]
async fn test_update_set_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = ConfigItemUpdateset {
        r#type: Some(None),
        owner: Some(None),
        ..create_basic_updateset()
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}", ci.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let ci_after: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    assert!(ci_after.r#type.is_none());
    assert!(ci_after.owner.is_none());
}

#[db_test]
async fn test_update_nothing(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci_before = configuration::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = ConfigItemUpdateset {
        name: None,
        status: None,
        created_at: None,
        r#type: None,
        owner: None,
        description: None,
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}", ci_before.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let ci_after: ConfigItem = response.into_body().into_json::<ConfigItem>().await;
    assert_that!(ci_after, eq(&ci_before));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/api/configitems/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let ci = configuration::create(createset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/configitems/{}", ci.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = configuration::load(ci.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
