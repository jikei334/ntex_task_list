mod models;
mod message;

use std::sync::Arc;

use diesel::query_dsl::methods::FindDsl;
use diesel::r2d2;
use diesel::prelude::{Connection, PgConnection, RunQueryDsl};
use ntex::web;

use models::{NewTask, ModifiedTask, Task, TaskList};
use message::TaskMessage;
use super::schema::task::dsl;


type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

async fn get_tasks(
    pool: web::types::State<Arc<DbPool>>
) -> Result<web::HttpResponse, web::Error> {

    let pool = pool.get_ref().clone();

    let results = web::block(move || {
        let mut conn = pool.get().expect("Failed to get DB connection");
        dsl::task
            .load::<Task>(&mut conn)
    }).await;

    match results {
        Ok(task_list) => {
            let task_list = TaskList::new(task_list);
            Ok(web::HttpResponse::Ok()
                .json(&task_list))
        },
        Err(err) => {
            eprintln!("Database query error: {:?}", err);
            Ok(web::HttpResponse::InternalServerError().body("Error retrieving tasks"))
        },
    }
}

async fn register_task(
    pool: web::types::State<Arc<DbPool>>,
    task: web::types::Json<NewTask>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let task = task.0;

    let result = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        task.insert(&mut conn)
    }).await;

    match result {
        Ok(task) => {
            let message = TaskMessage::info(
                "task registered!".to_string(),
                task
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(err) => {
            let message = TaskMessage::error(format!("{:?}", err).to_string());
            Ok(web::HttpResponse::InternalServerError().json(&message))
        },
    }
}

async fn modify_task(
    pool: web::types::State<Arc<DbPool>>,
    path: web::types::Path<i32>,
    task: web::types::Json<ModifiedTask>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let task_id = path.into_inner();
    let task = task.0;

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let result = task.modify(task_id, &mut conn);

    match result {
        Ok(task) => {
            let message = TaskMessage::info(
                "task modified".to_string(),
                task
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(err) => {
            let message = TaskMessage::error(format!("{:?}", err).to_string());
            Ok(web::HttpResponse::InternalServerError().json(&message))
        }
    }
}

pub fn ntex_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/task")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::get().to(get_tasks))
            .route(web::post().to(register_task))
    ).service(
        web::resource("/task/{task_id}")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::put().to(modify_task))
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;
    use diesel::prelude::PgConnection;
    use diesel::r2d2;
    use diesel::r2d2::ConnectionManager;
    use ntex::web;
    use ntex::web::types;
    use ntex::http::StatusCode;
    use serde::{Deserialize, Serialize};

    use super::{DbPool, get_tasks, modify_task, register_task, ntex_config};
    use super::message;
    use super::message::TaskMessage;
    use super::models::{NewTask, ModifiedTask, Task};


    #[derive(Deserialize, Serialize)]
    pub struct InvalidTask {
        pub fake_title: String,
    }

    impl InvalidTask {
        pub fn new(fake_title: String) -> Self {
            InvalidTask {
                fake_title
            }
        }
    }

    fn get_state() -> Arc<DbPool> {
        dotenv::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL").expect("DATABASE_URL is not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Arc::new(r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool"))
    }

    #[ntex::test]
    async fn test_get_tasks_ok() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let request = web::test::TestRequest::get().uri("/api/task").to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(response.status().is_success());
    }

    #[ntex::test]
    async fn test_register_task_ok() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let task = NewTask::new(
            "TestTask".to_string(),
            "".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&task)
            .to_request();
        let message: TaskMessage = web::test::read_response_json(&app, request).await;

        match message {
            TaskMessage::Info(info_message) => {
                assert_eq!(info_message.message(), "task registered!");
                assert_eq!(info_message.task().title, "TestTask");
                assert_eq!(info_message.task().description, "");
                assert_eq!(info_message.task().deadline, NaiveDate::from_ymd_opt(2005, 10, 26).unwrap());
                assert_eq!(info_message.task().finished, false);
            },
            TaskMessage::Error(_) => {
                panic!("Task registration error");
            },
        }
    }

    #[ntex::test]
    async fn test_register_task_ng_with_no_task() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(!response.status().is_success());
    }

    #[ntex::test]
    async fn test_register_task_ng_with_invalid_task() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let task = InvalidTask::new("TestTask".to_string());
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&task)
            .to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(!response.status().is_success());
    }

    #[ntex::test]
    async fn test_modify_task_ok() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

        let task = NewTask::new(
            "ModifyTaskTest".to_string(),
            "Before modify".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&task)
            .to_request();
        let message: TaskMessage = web::test::read_response_json(&app, request).await;
        match message {
            TaskMessage::Info(task_info_message) => {
                let registered_task = task_info_message.task();
                assert_eq!(registered_task.description, "Before modify".to_string());
                let modifying_task = ModifiedTask::new(
                    registered_task.title.clone(),
                    "After modify".to_string(),
                    false,
                    registered_task.deadline
                );
                let request = web::test::TestRequest::put()
                    .uri(&format!("/api/task/{}", registered_task.id))
                    .set_json(&modifying_task)
                    .to_request();
                let message: TaskMessage = web::test::read_response_json(&app, request).await;

                match message {
                    TaskMessage::Info(task_info_message) => {
                        assert_eq!(task_info_message.message(), "task modified");
                        assert_eq!(task_info_message.task().id, registered_task.id);
                        assert_eq!(task_info_message.task().title, "ModifyTaskTest".to_string());
                        assert_eq!(task_info_message.task().description, "After modify".to_string());
                        assert_eq!(task_info_message.task().deadline, registered_task.deadline);
                    },
                    TaskMessage::Error(_) => {
                        panic!();
                    },
                }
            },
            TaskMessage::Error(_) => {
                panic!();
            },
        }
    }
}
