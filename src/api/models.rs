use chrono::{NaiveDateTime, NaiveDate, Utc};
use diesel::prelude::{Associations, Insertable, Selectable, PgConnection, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = crate::schema::task)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub finished: bool,
    pub deadline: NaiveDate,
    pub created: NaiveDateTime,
}

#[derive(Deserialize, Insertable, Serialize)]
#[diesel(table_name = crate::schema::task)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTask {
    pub title: String,
    pub description: String,
    pub deadline: NaiveDate,
}

impl NewTask {
    pub fn new(
        title: String,
        description: String,
        deadline: NaiveDate
    ) -> Self {
        NewTask {
            title,
            description,
            deadline,
        }
    }

    pub fn insert(self, conn: &mut PgConnection) -> diesel::prelude::QueryResult<usize> {
        diesel::insert_into(crate::schema::task::dsl::task)
            .values(&self)
            .execute(conn)
    }
}

#[derive(Serialize)]
pub struct TaskList {
    tasks: Vec<Task>,
}

impl TaskList {
    pub fn new(tasks: Vec<Task>) -> Self {
        TaskList {
            tasks
        }
    }
}


#[derive(Associations, Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = crate::schema::comment)]
#[diesel(belongs_to(Task))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub task_id: i32,
    pub created: NaiveDateTime,
}


#[derive(Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = crate::schema::tag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: i32,
    pub name: String,
}


#[derive(Associations, Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = crate::schema::task_tag)]
#[diesel(belongs_to(Tag))]
#[diesel(belongs_to(Task))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskTag {
    pub tag_id: i32,
    pub task_id: i32,
}
