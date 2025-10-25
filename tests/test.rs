mod models;

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

use ntex_task_list::{DbPool, ntex_config};

use models::{
    TestTaskView,
    TestCommentView,
    TestFilterBoolean, TestFilterContainable, TestFilterOrdered,
    TestSortOrder, TestTaskOrder, TestTaskQuery,
    TestValidNewTask, TestInvalidNewTask, TestValidModifiedTask,
    TestValidNewComment, TestInvalidNewComment,
    TestSimpleMessage,
    TestTaskMessage, TestPagenatedTaskListMessage,
    TestCommentMessage,
};


fn get_state() -> Arc<DbPool> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL").expect("DATABASE_URL is not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Arc::new(r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool"))
}

async fn create_task<S>(new_task: TestValidNewTask, app: &Pipeline<S>) -> TestTaskView
where S: Service<Request, Response = WebResponse>
{
    let request = web::test::TestRequest::post()
        .uri("/api/task")
        .set_json(&new_task)
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(app, request).await;

    match message {
        TestTaskMessage::Info(info_message) => {
            assert_eq!(info_message.message(), "task registered!");
            assert_eq!(info_message.task().title().to_owned(), new_task.title().to_owned());
            assert_eq!(info_message.task().description().to_owned(), new_task.description().to_owned());
            assert_eq!(info_message.task().deadline(), new_task.deadline());
            assert_eq!(info_message.task().finished(), false);
            info_message.task().clone()
        },
        TestTaskMessage::Error(error) => {
            panic!("Error: {:?}", error.message());
        }
    }
}

async fn get_task<S>(task_id: i32, app: &Pipeline<S>) -> TestTaskView
where S: Service<Request, Response = WebResponse>
{
    let request = web::test::TestRequest::get()
        .uri(&format!("/api/task/{}", task_id))
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestTaskMessage::Info(task_info_message) => {
            task_info_message.task().to_owned()
        },
        TestTaskMessage::Error(error) => {
            panic!("{}", error.message());
        },
    }
}

#[ntex::test]
async fn test_register_task_ok() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let task = TestValidNewTask::new(
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
    let task = TestInvalidNewTask::new("TestTask".to_string());
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
    let new_task = TestValidNewTask::new(
        "NewTask".to_string(),
        "description".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let task = create_task(new_task, &app).await;
    let request = web::test::TestRequest::get()
        .uri(&format!("/api/task/{}", task.id()))
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestTaskMessage::Info(task_info_message) => {
            let got_task = task_info_message.task();
            assert_eq!(got_task.title(), task.title());
            assert_eq!(got_task.description(), task.description());
            assert_eq!(got_task.deadline(), task.deadline());
        },
        TestTaskMessage::Error(error) => {
            panic!("{}", error.message());
        },
    }
}

#[ntex::test]
async fn test_get_task_not_found() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let task = create_task(TestValidNewTask::new(
            "NewTask".to_string(),
            "description".to_string(),
            NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    ), &app).await;
    // FIXME: latest id + 100 may not be exist.
    let request = web::test::TestRequest::get()
        .uri(&format!("/api/task/{}", task.id() + 100))
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestTaskMessage::Info(_) => {
            panic!("This request must not return task");
        },
        TestTaskMessage::Error(error) => {
            assert_eq!(error.message(), "Not Found");
        },
    }
}

#[ntex::test]
async fn test_modify_task_ok() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

    let task = TestValidNewTask::new(
        "ModifyTaskTest".to_string(),
        "Before modify".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let registered_task = create_task(task, &app).await;
    let modifying_task = TestValidModifiedTask::new(
        registered_task.title().clone(),
        "After modify".to_string(),
        false,
        registered_task.deadline()
    );
    let request = web::test::TestRequest::put()
        .uri(&format!("/api/task/{}", registered_task.id()))
        .set_json(&modifying_task)
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;

    match message {
        TestTaskMessage::Info(task_info_message) => {
            assert_eq!(task_info_message.message(), "task modified");
            assert_eq!(task_info_message.task().id(), registered_task.id());
            assert_eq!(task_info_message.task().title().to_owned(), "ModifyTaskTest".to_string());
            assert_eq!(task_info_message.task().description().to_owned(), "After modify".to_string());
            assert_eq!(task_info_message.task().deadline(), registered_task.deadline());
        },
        TestTaskMessage::Error(_) => {
            panic!();
        },
    }
}

#[ntex::test]
async fn test_search_filter() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

    let current_time = format!("{:?}", Local::now());
    let task1 = TestValidNewTask::new(
        format!("SearchTask1[{}]", current_time).to_string(),
        "description1 Lose".to_string(),
        NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
    );
    let registered_task1 = create_task(task1, &app).await;
    let task2 = TestValidNewTask::new(
        format!("SearchTask2[{}]", current_time).to_string(),
        "description2 Lose".to_string(),
        NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
    );
    let registered_task2 = create_task(task2, &app).await;
    let task3 = TestValidNewTask::new(
        format!("SearchTask3[{}]", current_time).to_string(),
        "description3 Win".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let registered_task3 = create_task(task3, &app).await;

    // containe title
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()));
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(task_set.contains(&registered_task1.id()), "[containe title]registered_task1 is not exist.");
            assert!(task_set.contains(&registered_task2.id()), "[containe title]registered_task2 is not exist.");
            assert!(task_set.contains(&registered_task3.id()), "[containe title]registered_task3 is not exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[containe title]Error message: {}", error.message());
        },
    }

    // equal description
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()))
        .set_description(TestFilterContainable::EQUAL("description2 Lose".to_string()));
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(!task_set.contains(&registered_task1.id()), "[equal description]registered_task1 is exist.");
            assert!(task_set.contains(&registered_task2.id()), "[equal description]registered_task2 is not exist.");
            assert!(!task_set.contains(&registered_task3.id()), "[equal description]registered_task3 is exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[equal description]Error message: {}", error.message());
        },
    }

    // finished true
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()))
        .set_finished(TestFilterBoolean::TRUE);
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(!task_set.contains(&registered_task1.id()), "[finished true]registered_task1 is exist.");
            assert!(!task_set.contains(&registered_task2.id()), "[finished true]registered_task2 is exist.");
            assert!(!task_set.contains(&registered_task3.id()), "[finished true]registered_task3 is exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[finished true]Error message: {}", error.message());
        },
    }

    // finished false
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()))
        .set_finished(TestFilterBoolean::FALSE);
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(task_set.contains(&registered_task1.id()), "[finished false]registered_task1 is not exist.");
            assert!(task_set.contains(&registered_task2.id()), "[finished false]registered_task2 is not exist.");
            assert!(task_set.contains(&registered_task3.id()), "[finished false]registered_task3 is not exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[finished false]Error message: {}", error.message());
        },
    }

    // created less
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()))
        .set_created(TestFilterOrdered::LT(registered_task2.created()));
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(task_set.contains(&registered_task1.id()), "[created less]registered_task1 is not exist.");
            assert!(!task_set.contains(&registered_task2.id()), "[created less]registered_task2 is exist.");
            assert!(!task_set.contains(&registered_task3.id()), "[created less]registered_task3 is exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[created less]Error message: {}", error.message());
        },
    }

    // deadline greater or equal
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()))
        .set_deadline(TestFilterOrdered::GE(registered_task2.deadline()));
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_set = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks().iter()
                .map(|task| task.id()).collect::<std::collections::HashSet<i32>>();
            eprintln!("{}, {}, {} in {:?}({})", registered_task1.id(), registered_task2.id(), registered_task3.id(),
            task_set, pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            assert!(task_set.contains(&registered_task1.id()), "[deadline greater or equal]registered_task1 is not exist.");
            assert!(task_set.contains(&registered_task2.id()), "[deadline greater or equal]registered_task2 is not exist.");
            assert!(!task_set.contains(&registered_task3.id()), "[deadline greater or equal]registered_task3 is exist.");
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[deadline greater or equal]Error message: {}", error.message());
        },
    }
}

#[ntex::test]
async fn test_search_order() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

    let current_time = format!("{:?}", Local::now());
    let task1 = TestValidNewTask::new(
        format!("SearchTask1[{}]", current_time).to_string(),
        "description1 Lose".to_string(),
        NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
    );
    let registered_task1 = create_task(task1, &app).await;
    let task2 = TestValidNewTask::new(
        format!("SearchTask2[{}]", current_time).to_string(),
        "description2 Lose".to_string(),
        NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
    );
    let registered_task2 = create_task(task2, &app).await;
    let task3 = TestValidNewTask::new(
        format!("SearchTask3[{}]", current_time).to_string(),
        "description3 Win".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let registered_task3 = create_task(task3, &app).await;

    // created asc
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()));
    task_query.order = TestTaskOrder::Created(TestSortOrder::ASC);
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks();
            for i in 1..task_list.len() {
                let prev_task = task_list[i-1].clone();
                let current_task = task_list[i].clone();
                assert!(prev_task.created() <= current_task.created(), "[created asc]{}: {:?} > {:?}",
                    i, prev_task.created(), current_task.created());
            }
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[created asc]Error message: {}", error.message());
        },
    }

    // deadline desc
    let mut task_query = TestTaskQuery::new(1, 100);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(current_time.clone()));
    task_query.order = TestTaskOrder::Deadline(TestSortOrder::DESC);
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks();
            for i in 1..task_list.len() {
                let prev_task = task_list[i-1].clone();
                let current_task = task_list[i].clone();
                assert!(prev_task.deadline() >= current_task.deadline(), "[deadline desc]{}: {:?} < {:?}",
                    i, prev_task.deadline(), current_task.deadline());
            }
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[deadline desc]Error message: {}", error.message());
        },
    }
}

#[ntex::test]
async fn test_pagenated_task() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;

    let key = format!("{:?}_pagenated_task", Local::now());
    let task1 = TestValidNewTask::new(
        format!("SearchTask1[{}]", key).to_string(),
        "description1 Lose".to_string(),
        NaiveDate::from_ymd_opt(2023, 11, 5).unwrap()
    );
    let registered_task1 = create_task(task1, &app).await;
    let task2 = TestValidNewTask::new(
        format!("SearchTask2[{}]", key).to_string(),
        "description2 Lose".to_string(),
        NaiveDate::from_ymd_opt(2014, 10, 25).unwrap()
    );
    let registered_task2 = create_task(task2, &app).await;
    let task3 = TestValidNewTask::new(
        format!("SearchTask3[{}]", key).to_string(),
        "description3 Win".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let registered_task3 = create_task(task3, &app).await;

    let mut task_query = TestTaskQuery::new(2, 2);
    task_query.filter = task_query.filter
        .set_title(TestFilterContainable::CONTAIN(key.clone()));
    let request = web::test::TestRequest::post()
        .uri("/api/task/search")
        .set_json(&task_query)
        .to_request();
    let message: TestPagenatedTaskListMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestPagenatedTaskListMessage::Info(pagenated_task_list_info_list_message) => {
            assert!(pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks() == 3,
            "num task is not 3: {}", pagenated_task_list_info_list_message.pagenated_task_list().num_total_tasks());
            let task_list = pagenated_task_list_info_list_message.pagenated_task_list().task_view_list().tasks();
            assert!(task_list.len() == 1, "task list must contains 1 item, but contains {} items.", task_list.len());
            assert!(task_list[0].id() == registered_task1.id());
        },
        TestPagenatedTaskListMessage::Error(error) => {
            panic!("[pagenated task]Error message: {}", error.message());
        },
    }
}

#[ntex::test]
async fn test_delete_task_ok() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let new_task = TestValidNewTask::new(
        "NewTask".to_string(),
        "description".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let task = create_task(new_task, &app).await;
    let request = web::test::TestRequest::get()
        .uri(&format!("/api/task/{}", task.id()))
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestTaskMessage::Info(task_info_message) => {
            let got_task = task_info_message.task();
            assert_eq!(got_task.title(), task.title());
            assert_eq!(got_task.description(), task.description());
            assert_eq!(got_task.deadline(), task.deadline());
        },
        TestTaskMessage::Error(error) => {
            panic!("{}", error.message());
        },
    }

    let request = web::test::TestRequest::delete()
        .uri(&format!("/api/task/{}", task.id()))
        .to_request();
    let message: TestSimpleMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestSimpleMessage::Info(_) => (),
        TestSimpleMessage::Error(error) => panic!("{:?}", error),
    }

    let request = web::test::TestRequest::get()
        .uri(&format!("/api/task/{}", task.id()))
        .to_request();
    let message: TestTaskMessage = web::test::read_response_json(&app, request).await;
    match message {
        TestTaskMessage::Info(_) => {
            panic!("task is not deleted");
        },
        TestTaskMessage::Error(_) => (),
    }
}

async fn create_comment<S>(new_comment: TestValidNewComment, app: &Pipeline<S>) -> TestCommentView
where S: Service<Request, Response = WebResponse>
{
    let request = web::test::TestRequest::post()
        .uri("/api/comment")
        .set_json(&new_comment)
        .to_request();
    let message: TestCommentMessage = web::test::read_response_json(app, request).await;

    match message {
        TestCommentMessage::Info(info_message) => {
            assert_eq!(info_message.message(), "comment added!");
            assert_eq!(info_message.comment().content().to_owned(), new_comment.content().to_owned());
            assert_eq!(info_message.comment().task_id(), new_comment.task_id());
            info_message.comment().clone()
        },
        TestCommentMessage::Error(error) => {
            panic!("Error: {:?}", error.message());
        },
    }
}

#[ntex::test]
async fn test_register_comment_ok() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let task = TestValidNewTask::new(
        "TestTask for register comment".to_string(),
        "".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let task = create_task(task, &app).await;

    let comment1 = TestValidNewComment::new(
        "1st register comment".to_string(),
        task.id(),
    );
    let comment1_view = create_comment(comment1, &app).await;

    let comment2 = TestValidNewComment::new(
        "2nd register comment".to_string(),
        task.id(),
    );
    let comment2_view = create_comment(comment2, &app).await;

    let task_view = get_task(task.id(), &app).await;

    assert_eq!(task_view.comments().len(), 2);

    let mut comment_appeared = vec![false; 2];

    for comment in task_view.comments() {
        if comment.id() == comment1_view.id() {
            comment_appeared[0] = true;
            assert_eq!(comment.content().to_owned(), comment1_view.content().to_owned());
        } else {
            comment_appeared[1] = true;
            assert_eq!(comment.content().to_owned(), comment2_view.content().to_owned());
        }
        assert_eq!(comment.task_id(), task_view.id());
    }
    assert!(comment_appeared[0]);
    assert!(comment_appeared[1]);
}

#[ntex::test]
async fn test_register_comment_ng_with_invalid_comment() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let task = TestValidNewTask::new(
        "TestTask for register comment with invalid format".to_string(),
        "".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let task = create_task(task, &app).await;

    let invalid_comment = TestInvalidNewComment::new(
        "register comment".to_string(),
        task.id(),
    );

    let request = web::test::TestRequest::post()
        .uri("/api/comment")
        .set_json(&invalid_comment)
        .to_request();
    let response = web::test::call_service(&app, request).await;

    assert!(!response.status().is_success());
}

#[ntex::test]
async fn test_register_comment_ng_with_invalid_task_id() {
    let state = get_state();
    let app = web::test::init_service(web::App::new().state(state).service(web::scope("/api").configure(ntex_config))).await;
    let task = TestValidNewTask::new(
        "TestTask for register comment".to_string(),
        "".to_string(),
        NaiveDate::from_ymd_opt(2005, 10, 26).unwrap()
    );
    let task = create_task(task, &app).await;

    let comment = TestValidNewComment::new(
        "comment with invalid task".to_string(),
        // the newest task's id plus 10000 may not be exist
        task.id() + 10000,
    );
    let request = web::test::TestRequest::post()
        .uri("/api/comment")
        .set_json(&comment)
        .to_request();
    let message: TestCommentMessage = web::test::read_response_json(&app, request).await;

    match message {
        TestCommentMessage::Info(_) => {
            panic!("This comment's task id is invalid");
        },
        TestCommentMessage::Error(_) => {
        },
    }
}
