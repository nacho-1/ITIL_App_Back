use axum::{
    body::Body,
    http::{self, Method},
};
use chrono::{Duration, Utc};
use googletest::prelude::*;
use hyper::StatusCode;
use itil_back_db::entities::incidents::{
    self, Incident, IncidentCreateset, IncidentImpact, IncidentStatus, IncidentUpdateset,
    IncidentUrgency,
};
use itil_back_macros::db_test;
use itil_back_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use uuid::Uuid;

fn create_basic_createset() -> IncidentCreateset {
    IncidentCreateset {
        title: String::from("Testing Incident"),
        status: Some(IncidentStatus::InProgress),
        created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
        resolved_at: None,
        impact: IncidentImpact::Low,
        urgency: IncidentUrgency::Low,
        owner: Some(String::from("Testing Department")),
        description: String::from("This is a fictional incident made for testing."),
    }
}

fn create_basic_updateset() -> IncidentUpdateset {
    IncidentUpdateset {
        title: Some(Some(String::from("Updated Incident"))),
        status: Some(Some(IncidentStatus::Closed)),
        created_at: Some(Some("2023-09-15T12:34:58Z".parse().unwrap())),
        resolved_at: Some(Some("2023-10-15T12:34:50Z".parse().unwrap())),
        impact: Some(Some(IncidentImpact::Medium)),
        urgency: Some(Some(IncidentUrgency::Medium)),
        owner: Some(Some(String::from("Update Department"))),
        description: Some(Some(String::from(
            "This is a fictional incident made for updating.",
        ))),
    }
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(IncidentCreateset {
        title: String::from(""),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        title: String::from(&"x".repeat(256)),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        owner: Some(String::from(&"x".repeat(1025))),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        description: String::from(&"x".repeat(1025)),
        ..createset.clone()
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
async fn test_create_bad_payload(context: &DbTestContext) {
    let payload = "{}";

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

#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let payload = json!(createset);

    let response = context
        .app
        .request("/api/incidents")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let incident: Incident = response.into_body().into_json::<Incident>().await;
    assert_that!(incident.title, eq(&createset.title));
    assert_that!(incident.status, eq(createset.status.unwrap()));
    assert_that!(incident.created_at, eq(createset.created_at.unwrap()));
    assert_that!(incident.resolved_at, eq(createset.resolved_at));
    assert_that!(incident.impact, eq(createset.impact));
    assert_that!(incident.urgency, eq(createset.urgency));
    assert_that!(incident.owner, eq(&createset.owner));
    assert_that!(incident.description, eq(&createset.description));

    let incidents = incidents::load_all(&context.db_pool).await.unwrap();
    assert_that!(incidents, len(eq(1)));
}

#[db_test]
async fn test_create_no_creation_date(context: &DbTestContext) {
    let createset = IncidentCreateset {
        created_at: None,
        ..create_basic_createset()
    };
    let payload = json!(createset);

    let t0 = Utc::now();
    let response = context
        .app
        .request("/api/incidents")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let incident: Incident = response.into_body().into_json::<Incident>().await;
    let diff = (incident.created_at - t0).num_seconds().abs();
    // Testing that diff is no bigger than 2 minutes, for putting a reasonable diff.
    assert_that!(diff, lt(Duration::seconds(120).num_seconds()));
}

#[db_test]
async fn test_create_border_success(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(IncidentCreateset {
        title: String::from("x"),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        title: String::from(&"x".repeat(255)),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        status: None,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        owner: None,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        owner: Some(String::from("")),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        owner: Some(String::from(&"x".repeat(1024))),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        description: String::from(""),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        description: String::from(&"x".repeat(1024)),
        ..createset.clone()
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

        assert_that!(response.status(), eq(StatusCode::CREATED));
    }
}

#[db_test]
async fn test_status(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(IncidentCreateset {
        status: Some(IncidentStatus::InProgress),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        status: Some(IncidentStatus::Open),
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        status: Some(IncidentStatus::Closed),
        ..createset.clone()
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

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let incident = response.into_body().into_json::<Incident>().await;
        assert_that!(incident.status, eq(set.status.unwrap()));
    }
}

#[db_test]
async fn test_impact(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(IncidentCreateset {
        impact: IncidentImpact::High,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        impact: IncidentImpact::Medium,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        impact: IncidentImpact::Low,
        ..createset.clone()
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

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let incident = response.into_body().into_json::<Incident>().await;
        assert_that!(incident.impact, eq(set.impact));
    }
}

#[db_test]
async fn test_urgency(context: &DbTestContext) {
    let createset = create_basic_createset();
    let mut sets = Vec::new();
    sets.push(IncidentCreateset {
        urgency: IncidentUrgency::High,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        urgency: IncidentUrgency::Medium,
        ..createset.clone()
    });
    sets.push(IncidentCreateset {
        urgency: IncidentUrgency::Low,
        ..createset.clone()
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

        assert_that!(response.status(), eq(StatusCode::CREATED));
        let incident = response.into_body().into_json::<Incident>().await;
        assert_that!(incident.urgency, eq(set.urgency));
    }
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/api/incidents").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incidents: Vec<Incident> = response.into_body().into_json::<Vec<Incident>>().await;
    assert_that!(incidents, len(eq(1)));
    assert_that!(incidents.first().unwrap(), eq(&incident));
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
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/incidents/{}", &incident.id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incident_read: Incident = response.into_body().into_json::<Incident>().await;
    assert_that!(incident_read, eq(&incident));
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(IncidentUpdateset {
        title: Some(Some(String::from(""))),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        title: Some(Some(String::from(&"x".repeat(256)))),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        owner: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        description: Some(Some(String::from(&"x".repeat(1025)))),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/incidents/{}", incident.id))
            .method(Method::PUT)
            .body(Body::from(payload.to_string()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

        let incident_after = incidents::load(incident.id, &context.db_pool)
            .await
            .unwrap();
        assert_that!(incident_after, eq(&incident));
    }
}

#[db_test]
async fn test_update_invalid_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let mut sets = Vec::new();
    sets.push(IncidentUpdateset {
        title: Some(None),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        status: Some(None),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        created_at: Some(None),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        impact: Some(None),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        urgency: Some(None),
        ..updateset.clone()
    });
    sets.push(IncidentUpdateset {
        description: Some(None),
        ..updateset.clone()
    });

    for set in sets {
        let payload = json!(set);

        let response = context
            .app
            .request(&format!("/api/incidents/{}", incident.id))
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
    let createset = create_basic_createset();
    let payload = json!(createset);

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
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = create_basic_updateset();
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let incident: Incident = response.into_body().into_json::<Incident>().await;
    assert_that!(incident.title, eq(&updateset.title.unwrap().unwrap()));
    assert_that!(incident.status, eq(updateset.status.unwrap().unwrap()));
    assert_that!(
        incident.created_at,
        eq(updateset.created_at.unwrap().unwrap())
    );
    assert_that!(incident.resolved_at, eq(updateset.resolved_at.unwrap()));
    assert_that!(incident.impact, eq(updateset.impact.unwrap().unwrap()));
    assert_that!(incident.urgency, eq(updateset.urgency.unwrap().unwrap()));
    assert_that!(incident.owner, eq(&updateset.owner.unwrap()));
    assert_that!(
        incident.description,
        eq(&updateset.description.unwrap().unwrap())
    );

    let incident_after = incidents::load(incident.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(incident_after, eq(&incident));
}

#[db_test]
async fn test_update_set_nulls(context: &DbTestContext) {
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = IncidentUpdateset {
        resolved_at: Some(None),
        owner: Some(None),
        ..create_basic_updateset()
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let incident_after: Incident = response.into_body().into_json::<Incident>().await;
    assert!(incident_after.resolved_at.is_none());
    assert!(incident_after.owner.is_none());
}

#[db_test]
async fn test_update_nothing(context: &DbTestContext) {
    let createset = create_basic_createset();
    let incident_before = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let updateset = IncidentUpdateset {
        title: None,
        status: None,
        created_at: None,
        resolved_at: None,
        impact: None,
        urgency: None,
        owner: None,
        description: None,
    };
    let payload = json!(updateset);

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident_before.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));
    let incident_after: Incident = response.into_body().into_json::<Incident>().await;
    assert_that!(incident_after, eq(&incident_before));
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
    let createset = create_basic_createset();
    let incident = incidents::create(createset, &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/api/incidents/{}", incident.id))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = incidents::load(incident.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
