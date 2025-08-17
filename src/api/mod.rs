mod models;
mod message;
mod query;

use std::sync::Arc;

use diesel::query_dsl::methods::FindDsl;
use diesel::r2d2;
use diesel::prelude::{Connection, PgConnection, RunQueryDsl};
use diesel::QueryDsl;
use diesel::result::Error as DieselError;
use ntex::web;
use serde::{Deserialize, Serialize};

use models::{NewTask, ModifiedTask, Task, TaskList};
use message::{PagenatedTaskListMessage, TaskMessage};
use query::{TaskFilter, TaskOrder, TaskQuery};

use crate::schema::task;
use crate::schema::task::dsl;


type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;


async fn get_tasks(
    pool: web::types::State<Arc<DbPool>>,
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

async fn search_task (
    pool: web::types::State<Arc<DbPool>>,
    task_query: web::types::Json<TaskQuery>,
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let task_query = task_query.0;

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let result = task_query.get_pagenated_tasks(&mut conn);

    match result {
        Ok(paginated_task_list) => {
            let message = PagenatedTaskListMessage::info(
                "search success".to_string(),
                paginated_task_list
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(error) => {
            let message = PagenatedTaskListMessage::error(format!("{}", error).to_string());
            Ok(web::HttpResponse::InternalServerError().json(&message))
        },
    }
}

async fn get_task(
    pool: web::types::State<Arc<DbPool>>,
    path: web::types::Path<i32>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let task_id = path.into_inner();

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    match Task::get(task_id, &mut conn) {
        Ok(task) => {
            let message = TaskMessage::info(
                "got task".to_string(),
                task,
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(error) => {
            match error {
                DieselError::NotFound => {
                    let message = TaskMessage::error("Not Found".to_string());
                    Ok(web::HttpResponse::NotFound().json(&message))
                },
                _ => {
                    let message = TaskMessage::error("Internal Error".to_string());
                    Ok(web::HttpResponse::InternalServerError().json(&message))
                },
            }
        }
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
        web::resource("/task/search")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::post().to(search_task))
    ).service(
        web::resource("/task/{task_id}")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::get().to(get_task))
            .route(web::put().to(modify_task))
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Local, NaiveDate};
    use diesel::prelude::PgConnection;
    use diesel::r2d2;
    use diesel::r2d2::ConnectionManager;
    use ntex::Pipeline;
    use ntex::Service;
    use ntex::web;
    use ntex::web::{types, WebResponse};
    use ntex::http::{Request, Response, StatusCode};
    use serde::{Deserialize, Serialize};

    use crate::api::{DbPool, get_tasks, modify_task, register_task, ntex_config};
    use crate::api::query::{FilterBoolean, FilterContainable, FilterOrderd, SortOrder, TaskOrder, TaskQuery};
    use super::message::{PagenatedTaskListMessage, TaskMessage};
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

    async fn create_task<S>(new_task: NewTask, app: &Pipeline<S>) -> Task
        where S: Service<Request, Response = WebResponse>
    {
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&new_task)
            .to_request();
        let message: TaskMessage = web::test::read_response_json(app, request).await;

        match message {
            TaskMessage::Info(info_message) => {
                assert_eq!(info_message.message(), "task registered!");
                assert_eq!(info_message.task().title, new_task.title);
                assert_eq!(info_message.task().description, new_task.description);
                assert_eq!(info_message.task().deadline, new_task.deadline);
                assert_eq!(info_message.task().finished, false);
                info_message.task().clone()
            },
            TaskMessage::Error(error) => {
                panic!("Error: {:?}", error.message());
            }
        }
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
        let _ = create_task(task, &app).await;
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
    async fn test_get_task_ok() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let new_task = NewTask::new(
            "NewTask".to_string(),
            "description".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let task = create_task(new_task, &app).await;
        let request = web::test::TestRequest::get()
            .uri(&format!("/api/task/{}", task.id))
            .to_request();
        let message: TaskMessage = web::test::read_response_json(&app, request).await;
        match message {
            TaskMessage::Info(task_info_message) => {
                let got_task = task_info_message.task();
                assert_eq!(got_task.title, task.title);
                assert_eq!(got_task.description, task.description);
                assert_eq!(got_task.deadline, task.deadline);
            },
            TaskMessage::Error(error) => {
                panic!("{}", error.message());
            },
        }
    }

    #[ntex::test]
    async fn test_get_task_not_found() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
        let task = create_task(NewTask::new(
                "NewTask".to_string(),
                "description".to_string(),
                NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        ), &app).await;
        // FIXME: latest id + 100 may not be exist.
        let request = web::test::TestRequest::get()
            .uri(&format!("/api/task/{}", task.id + 100))
            .to_request();
        let message: TaskMessage = web::test::read_response_json(&app, request).await;
        match message {
            TaskMessage::Info(_) => {
                panic!("This request must not return task");
            },
            TaskMessage::Error(error) => {
                assert_eq!(error.message(), "Not Found");
            },
        }
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
        let registered_task = create_task(task, &app).await;
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
    }

    #[ntex::test]
    async fn test_search_filter() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

        let current_time = format!("{:?}", Local::now());
        let task1 = NewTask::new(
            format!("SearchTask1[{}]", current_time).to_string(),
            "description1 Lose".to_string(),
            NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
        );
        let registered_task1 = create_task(task1, &app).await;
        let task2 = NewTask::new(
            format!("SearchTask2[{}]", current_time).to_string(),
            "description2 Lose".to_string(),
            NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
        );
        let registered_task2 = create_task(task2, &app).await;
        let task3 = NewTask::new(
            format!("SearchTask3[{}]", current_time).to_string(),
            "description3 Win".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let registered_task3 = create_task(task3, &app).await;

        // containe title
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()));
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(task_set.contains(&registered_task1.id), "[containe title]registered_task1 is not exist.");
                assert!(task_set.contains(&registered_task2.id), "[containe title]registered_task2 is not exist.");
                assert!(task_set.contains(&registered_task3.id), "[containe title]registered_task3 is not exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[containe title]Error message: {}", error.message());
            },
        }

        // equal description
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()))
            .set_description(FilterContainable::EQUAL("description2 Lose".to_string()));
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(!task_set.contains(&registered_task1.id), "[equal description]registered_task1 is exist.");
                assert!(task_set.contains(&registered_task2.id), "[equal description]registered_task2 is not exist.");
                assert!(!task_set.contains(&registered_task3.id), "[equal description]registered_task3 is exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[equal description]Error message: {}", error.message());
            },
        }

        // finished true
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()))
            .set_finished(FilterBoolean::TRUE);
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(!task_set.contains(&registered_task1.id), "[finished true]registered_task1 is exist.");
                assert!(!task_set.contains(&registered_task2.id), "[finished true]registered_task2 is exist.");
                assert!(!task_set.contains(&registered_task3.id), "[finished true]registered_task3 is exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[finished true]Error message: {}", error.message());
            },
        }

        // finished false
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()))
            .set_finished(FilterBoolean::FALSE);
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(task_set.contains(&registered_task1.id), "[finished false]registered_task1 is not exist.");
                assert!(task_set.contains(&registered_task2.id), "[finished false]registered_task2 is not exist.");
                assert!(task_set.contains(&registered_task3.id), "[finished false]registered_task3 is not exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[finished false]Error message: {}", error.message());
            },
        }

        // created less
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()))
            .set_created(FilterOrderd::LT(registered_task2.created));
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(task_set.contains(&registered_task1.id), "[created less]registered_task1 is not exist.");
                assert!(!task_set.contains(&registered_task2.id), "[created less]registered_task2 is exist.");
                assert!(!task_set.contains(&registered_task3.id), "[created less]registered_task3 is exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[created less]Error message: {}", error.message());
            },
        }

        // deadline greater or equal
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()))
            .set_deadline(FilterOrderd::GE(registered_task2.deadline));
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_list().iter()
                    .map(|task| task.id).collect::<std::collections::HashSet<i32>>();
                eprintln!("{}, {}, {} in {:?}({})", registered_task1.id, registered_task2.id, registered_task3.id,
                    task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                assert!(task_set.contains(&registered_task1.id), "[deadline greater or equal]registered_task1 is not exist.");
                assert!(task_set.contains(&registered_task2.id), "[deadline greater or equal]registered_task2 is not exist.");
                assert!(!task_set.contains(&registered_task3.id), "[deadline greater or equal]registered_task3 is exist.");
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[deadline greater or equal]Error message: {}", error.message());
            },
        }
    }

    #[ntex::test]
    async fn test_search_order() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

        let current_time = format!("{:?}", Local::now());
        let task1 = NewTask::new(
            format!("SearchTask1[{}]", current_time).to_string(),
            "description1 Lose".to_string(),
            NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
        );
        let registered_task1 = create_task(task1, &app).await;
        let task2 = NewTask::new(
            format!("SearchTask2[{}]", current_time).to_string(),
            "description2 Lose".to_string(),
            NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
        );
        let registered_task2 = create_task(task2, &app).await;
        let task3 = NewTask::new(
            format!("SearchTask3[{}]", current_time).to_string(),
            "description3 Win".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let registered_task3 = create_task(task3, &app).await;

        // created asc
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()));
        task_query.order = TaskOrder::Created(SortOrder::ASC);
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_list();
                for i in 1..task_list.len() {
                    let prev_task = task_list[i-1].clone();
                    let current_task = task_list[i].clone();
                    assert!(prev_task.created <= current_task.created, "[created asc]{}: {:?} > {:?}",
                        i, prev_task.created, current_task.created);
                }
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[created asc]Error message: {}", error.message());
            },
        }

        // deadline desc
        let mut task_query = TaskQuery::new(1, 100);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(current_time.clone()));
        task_query.order = TaskOrder::Deadline(SortOrder::DESC);
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_list();
                for i in 1..task_list.len() {
                    let prev_task = task_list[i-1].clone();
                    let current_task = task_list[i].clone();
                    assert!(prev_task.deadline >= current_task.deadline, "[deadline desc]{}: {:?} < {:?}",
                        i, prev_task.deadline, current_task.deadline);
                }
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[deadline desc]Error message: {}", error.message());
            },
        }
    }

    #[ntex::test]
    async fn test_pagenated_task() {
        let state = get_state();
        let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

        let key = format!("{:?}_pagenated_task", Local::now());
        let task1 = NewTask::new(
            format!("SearchTask1[{}]", key).to_string(),
            "description1 Lose".to_string(),
            NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
        );
        let registered_task1 = create_task(task1, &app).await;
        let task2 = NewTask::new(
            format!("SearchTask2[{}]", key).to_string(),
            "description2 Lose".to_string(),
            NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
        );
        let registered_task2 = create_task(task2, &app).await;
        let task3 = NewTask::new(
            format!("SearchTask3[{}]", key).to_string(),
            "description3 Win".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
        );
        let registered_task3 = create_task(task3, &app).await;

        let mut task_query = TaskQuery::new(2, 2);
        task_query.filter = task_query.filter
            .set_title(FilterContainable::CONTAIN(key.clone()));
        let request = web::test::TestRequest::post()
            .uri("/api/task/search")
            .set_json(&task_query)
            .to_request();
        let message: PagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
        match message {
            PagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
                assert!(pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks() == 3,
                    "num task is not 3: {}", pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
                let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_list();
                assert!(task_list.len() == 1, "task list must contains 1 item, but contains {} items.", task_list.len());
                assert!(task_list[0].id == registered_task1.id);
            },
            PagenatedTaskListMessage::Error(error) => {
                panic!("[pagenated task]Error message: {}", error.message());
            },
        }
    }
}
