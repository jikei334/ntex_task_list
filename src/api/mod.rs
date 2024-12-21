mod models;
mod message;

use ntex::web;

use models::{Task, TaskList};
use message::Message;


async fn get_tasks() -> Result<web::HttpResponse, web::Error> {
    let task_list = TaskList::new(
        vec![
            Task::new("TaskA".to_string()),
        ]
    );
    Ok(web::HttpResponse::Ok()
        .json(&task_list))
}

async fn register_task(task: web::types::Json<Task>) -> Result<web::HttpResponse, web::Error> {
    println!("register: {}", task.title);
    let message = Message::info(format!("{} is registered!", task.title).to_string());
    Ok(web::HttpResponse::Ok()
        .json(&message))
}

pub fn ntex_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/task")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::get().to(get_tasks))
            .route(web::post().to(register_task))
    );
}

#[cfg(test)]
mod tests {
    use ntex::web;
    use ntex::web::types;
    use ntex::http::StatusCode;
    use serde::{Deserialize, Serialize};

    use super::{get_tasks, register_task, ntex_config};
    use super::message;
    use super::message::Message;
    use super::models::Task;


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

    #[ntex::test]
    async fn test_get_tasks_ok() {
        match get_tasks().await {
            Ok(response) => assert_eq!(response.status(), StatusCode::OK),
            Err(error) => panic!("{:?}", error),
        }
    }

    #[ntex::test]
    async fn test_register_task() {
        let task = types::Json(Task::new("TestTask".to_string()));
        match register_task(task).await {
            Ok(response) => {
                assert_eq!(response.status(), StatusCode::OK);
            },
            Err(error) => panic!("{:?}", error),
        }
    }

    #[ntex::test]
    async fn test_integ_get_tasks_ok() {
        let app = web::test::init_service(web::App::new().service(web::scope("/api").configure(ntex_config))).await;
        let request = web::test::TestRequest::get().uri("/api/task").to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(response.status().is_success());
    }

    #[ntex::test]
    async fn test_integ_register_task_ok() {
        let app = web::test::init_service(web::App::new().service(web::scope("/api").configure(ntex_config))).await;
        let task = Task::new("TestTask".to_string());
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&task)
            .to_request();
        let message: Message = web::test::read_response_json(&app, request).await;

        assert_eq!(message.text(), "TestTask is registered!".to_string());

        assert!(matches!(message.status(), message::Status::Info));
    }

    #[ntex::test]
    async fn test_integ_register_task_ng_with_no_task() {
        let app = web::test::init_service(web::App::new().service(web::scope("/api").configure(ntex_config))).await;
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(!response.status().is_success());
    }

    #[ntex::test]
    async fn test_integ_register_task_ng_with_invalid_task() {
        let app = web::test::init_service(web::App::new().service(web::scope("/api").configure(ntex_config))).await;
        let task = InvalidTask::new("TestTask".to_string());
        let request = web::test::TestRequest::post()
            .uri("/api/task")
            .set_json(&task)
            .to_request();
        let response = web::test::call_service(&app, request).await;

        assert!(!response.status().is_success());
    }
}
