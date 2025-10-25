pub mod models;
pub mod message;
pub mod query;
pub mod schema;

use std::sync::Arc;

// use diesel::query_dsl::methods::FindDsl;
use diesel::r2d2;
use diesel::prelude::{Connection, PgConnection, RunQueryDsl};
// use diesel::QueryDsl;
use diesel::result::Error as DieselError;
use diesel::result::QueryResult;
use ntex::web;
use serde::{Deserialize, Serialize};

use models::{Comment, CommentView, NewTask, NewComment, ModifiedTask, Task, TaskView, TaskViewList};
use message::{CommentMessage, PagenatedTaskListMessage, SimpleMessage, TaskMessage};
use query::{TaskFilter, TaskOrder, TaskQuery};

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;


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
            match TaskView::new(&task, &mut conn) {
                Ok(task_view) => {
                    let message = TaskMessage::info(
                        "got task".to_string(),
                        task_view,
                    );
                    Ok(web::HttpResponse::Ok().json(&message))
                },
                Err(err) => {
                    let message = TaskMessage::error("Failed to get task".to_string());
                    Ok(web::HttpResponse::InternalServerError().json(&message))
                }
            }
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
    let pool_ref = pool.get_ref().clone();
    let task = task.0;

    let result = web::block(move || {
        let mut conn = pool_ref.get().expect("couldn't get db connection from pool");
        task.insert(&mut conn)
    }).await;

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    match result {
        Ok(task) => {
            match TaskView::new(&task, &mut conn) {
                Ok(task_view) => {
                    let message = TaskMessage::info(
                        "task registered!".to_string(),
                        task_view,
                    );
                    Ok(web::HttpResponse::Ok().json(&message))
                },
                Err(err) => {
                    let message = TaskMessage::error("Failed to get task".to_string());
                    Ok(web::HttpResponse::InternalServerError().json(&message))
                }
            }
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
            match TaskView::new(&task, &mut conn) {
                Ok(task_view) => {
                    let message = TaskMessage::info(
                        "task modified".to_string(),
                        task_view,
                    );
                    Ok(web::HttpResponse::Ok().json(&message))
                },
                Err(err) => {
                    let message = TaskMessage::error("Failed to get task".to_string());
                    Ok(web::HttpResponse::InternalServerError().json(&message))
                }
            }
        },
        Err(err) => {
            let message = TaskMessage::error(format!("{:?}", err).to_string());
            Ok(web::HttpResponse::InternalServerError().json(&message))
        }
    }
}

async fn delete_task(
    pool: web::types::State<Arc<DbPool>>,
    path: web::types::Path<i32>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let task_id = path.into_inner();

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let result = Task::delete(task_id, &mut conn);

    match result {
        Ok(task_id) => Ok(
            web::HttpResponse::Ok().json(&SimpleMessage::info(format!("task {} was deleted.", task_id).to_string()))),
        Err(_) => Ok(
            web::HttpResponse::InternalServerError().json(&SimpleMessage::error("Error occurred".to_string()))),
    }
}

async fn get_comment(
    pool: web::types::State<Arc<DbPool>>,
    path: web::types::Path<i32>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let comment_id = path.into_inner();

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    match Comment::get(comment_id, &mut conn) {
        Ok(comment) => {
            let comment_view = CommentView::new(&comment);
            let message = CommentMessage::info(
                "got comment".to_string(),
                comment_view,
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(error) => {
            match error {
                DieselError::NotFound => {
                    let message = CommentMessage::error("Not Found".to_string());
                    Ok(web::HttpResponse::NotFound().json(&message))
                },
                _ => {
                    let message = CommentMessage::error("Internal Error".to_string());
                    Ok(web::HttpResponse::InternalServerError().json(&message))
                },
            }
        }
    }
}

async fn add_comment(
    pool: web::types::State<Arc<DbPool>>,
    comment: web::types::Json<NewComment>
) -> Result<web::HttpResponse, web::Error> {
    let pool = pool.get_ref().clone();
    let comment = comment.0;

    let result = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        comment.insert(&mut conn)
    }).await;

    match result {
        Ok(comment) => {
            let message = CommentMessage::info(
                "comment added!".to_string(),
                CommentView::new(&comment),
            );
            Ok(web::HttpResponse::Ok().json(&message))
        },
        Err(err) => {
            let message = CommentMessage::error(format!("{:?}", err).to_string());
            Ok(web::HttpResponse::InternalServerError().json(&message))
        }
    }
}

pub fn ntex_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/task")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            // .route(web::get().to(get_tasks))
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
            .route(web::delete().to(delete_task))
    ).service(
        web::resource("/comment")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::post().to(add_comment))
    ).service(
        web::resource("/comment/{comment_id}")
            .wrap(web::middleware::DefaultHeaders::new().header("Access-Control-Allow-Origin", "*"))
            .route(web::get().to(get_comment))
    );
}
