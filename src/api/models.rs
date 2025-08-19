use chrono::{NaiveDateTime, NaiveDate, Utc};
use diesel::ExpressionMethods;
use diesel::expression::SelectableHelper;
use diesel::prelude::{
    AsChangeset, Associations, Identifiable, Insertable, Selectable, PgConnection, Queryable, QueryResult, RunQueryDsl
};
use diesel::query_dsl::methods::{FilterDsl, FindDsl};
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};


#[derive(Clone)]
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

impl Task {
    pub fn get(id: i32, conn: &mut PgConnection) -> Result<Task, DieselError> {
        crate::schema::task::dsl::task.find(id).first::<Task>(conn)
    }

    pub fn delete(id: i32, conn: &mut PgConnection) -> Result<usize, DieselError> {
        diesel::delete(crate::schema::task::dsl::task.filter(
                crate::schema::task::id.eq(id)
        )).execute(conn)
    }
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

    pub fn insert(self, conn: &mut PgConnection) -> QueryResult<Task> {
        diesel::insert_into(crate::schema::task::dsl::task)
            .values(&self)
            .returning(Task::as_returning())
            .get_result(conn)
    }
}

#[derive(AsChangeset, Deserialize, Insertable, Serialize)]
#[diesel(table_name = crate::schema::task)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ModifiedTask {
    pub title: String,
    pub description: String,
    pub finished: bool,
    pub deadline: NaiveDate,
}

impl ModifiedTask {
    pub fn new(
        title: String,
        description: String,
        finished: bool,
        deadline: NaiveDate,
    ) -> Self {
        Self {
            title,
            description,
            finished,
            deadline,
        }
    }

    pub fn modify(self, id: i32, conn: &mut PgConnection) -> QueryResult<Task> {
        diesel::update(crate::schema::task::table.find(id))
            .set(&self)
            .returning(Task::as_returning())
            .get_result(conn)
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
