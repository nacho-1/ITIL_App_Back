use axum::{
    body::Body,
    http::{self, Method},
};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::{self, configitems::ConfigItemChangeset};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

/// Create basic fake changeset for testing.
/// Values are border maximum as to test system's behaviour.
fn create_basic_changeset() -> entities::configitems::ConfigItemChangeset {
    let mut name = String::from("Testing Configuration Item - ");
    name.push_str(&"x".repeat(255 - name.len()));
    let mut type_content = String::from("Testing Item - ");
    type_content.push_str(&"x".repeat(31 - type_content.len()));
    let mut owner_content = String::from("Testing Department - ");
    owner_content.push_str(&"x".repeat(63 - owner_content.len()));
    let mut description = String::from("This is a fictional item made for testing. ");
    description.push_str(&"x".repeat(255 - description.len()));

    entities::configitems::ConfigItemChangeset {
        name,
        status: entities::configitems::CIStatus::Maintenance,
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        r#type: Some(type_content),
        owner: Some(owner_content),
        description,
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    // Set of changset with invalid values (only too few characters)
    let mut sets = Vec::new();
    sets.push(entities::configitems::ConfigItemChangeset {
        name: String::from(""),
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        name: String::from(&"x".repeat(256)),
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        r#type: Some(String::from("")),
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        r#type: Some(String::from(&"x".repeat(32))),
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        owner: Some(String::from("")),
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        owner: Some(String::from(&"x".repeat(64))),
        ..changeset.clone()
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
async fn test_create_success(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/api/configitems")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let configitems = entities::configitems::load_all(&context.db_pool)
        .await
        .unwrap();
    assert_that!(configitems, len(eq(1)));
    assert_that!(configitems.first().unwrap().name, eq(&changeset.name));
}

#[db_test]
async fn test_create_border_success(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    let mut sets = Vec::new();
    sets.push(ConfigItemChangeset {
        name: String::from("x"),
        ..changeset.clone()
    });
    sets.push(ConfigItemChangeset {
        created_at: None,
        ..changeset.clone()
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
    let changeset = create_basic_changeset();
    let mut sets = Vec::new();
    sets.push(entities::configitems::ConfigItemChangeset {
        status: entities::configitems::CIStatus::Testing,
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        status: entities::configitems::CIStatus::Active,
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        status: entities::configitems::CIStatus::Inactive,
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        status: entities::configitems::CIStatus::Retired,
        ..changeset.clone()
    });
    sets.push(entities::configitems::ConfigItemChangeset {
        status: entities::configitems::CIStatus::Maintenance,
        ..changeset.clone()
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
        let ci = response
            .into_body()
            .into_json::<entities::configitems::ConfigItem>()
            .await;
        assert_that!(ci.status, eq(set.status));
    }
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let changeset = create_basic_changeset();
    entities::configitems::create(changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/api/configitems").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let configitems: Vec<entities::configitems::ConfigItem> = response
        .into_body()
        .into_json::<Vec<entities::configitems::ConfigItem>>()
        .await;
    assert_that!(configitems, len(eq(1)));
    assert_that!(configitems.first().unwrap().name, eq(&changeset.name));
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
    let configitem_changeset = create_basic_changeset();
    let configitem = entities::configitems::create(configitem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let configitem_id = configitem.id;

    let response = context
        .app
        .request(&format!("/api/configitems/{}", configitem_id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let configitem: entities::configitems::ConfigItem = response
        .into_body()
        .into_json::<entities::configitems::ConfigItem>()
        .await;
    assert_that!(configitem.id, eq(configitem_id));
    assert_that!(configitem.name, eq(&configitem_changeset.name));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let configitem_changeset = create_basic_changeset();
    let configitem = entities::configitems::create(configitem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(entities::configitems::ConfigItemChangeset {
        name: String::from(""),
        ..configitem_changeset.clone()
    });

    let response = context
        .app
        .request(&format!("/api/configitems/{}", configitem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let configitem_after = entities::configitems::load(configitem.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(configitem_after.name, eq(&configitem.name));
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let configitem_changeset = create_basic_changeset();
    let payload = json!(configitem_changeset);

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
    let configitem_changeset = create_basic_changeset();
    let configitem = entities::configitems::create(configitem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let configitem_changeset = entities::configitems::ConfigItemChangeset {
        name: String::from("Testing Configuration Item - New Name"),
        ..configitem_changeset
    };
    let payload = json!(configitem_changeset);

    let response = context
        .app
        .request(&format!("/api/configitems/{}", configitem.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let configitem: entities::configitems::ConfigItem = response
        .into_body()
        .into_json::<entities::configitems::ConfigItem>()
        .await;
    assert_that!(configitem.name, eq(&configitem_changeset.name.clone()));

    let configitem = entities::configitems::load(configitem.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(configitem.name, eq(&configitem_changeset.name));
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
    let configitem_changeset = create_basic_changeset();
    let configitem = entities::configitems::create(configitem_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/configitems/{}", configitem.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = entities::configitems::load(configitem.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
