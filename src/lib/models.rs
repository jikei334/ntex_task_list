use chrono::{NaiveDateTime, NaiveDate, Utc};
use diesel::associations::GroupedBy;
use diesel::ExpressionMethods;
use diesel::expression::SelectableHelper;
use diesel::prelude::{
    AsChangeset, Associations, Identifiable, Insertable, Selectable, PgConnection, Queryable, QueryResult, RunQueryDsl
};
use diesel::query_dsl::BelongingToDsl;
use diesel::query_dsl::methods::{FilterDsl, FindDsl, SelectDsl};
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};

use crate::schema;


#[derive(Clone)]
#[derive(Deserialize, Identifiable, Selectable, Serialize, Queryable)]
#[diesel(table_name = schema::task)]
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
#[diesel(table_name = schema::task)]
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
#[diesel(table_name = schema::task)]
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
pub struct TaskView {
    id: i32,
    title: String,
    description: String,
    finished: bool,
    comments: Vec<CommentView>,
    deadline: NaiveDate,
    created: NaiveDateTime,
}

impl TaskView {
    pub fn new(task: &Task, conn: &mut PgConnection) -> QueryResult<Self> {
        let comments = Comment::belonging_to(&task)
            .select(Comment::as_select())
            .load(conn)?
            .into_iter()
            .map(|comment| CommentView::new(&comment))
            .collect();

        Self::new_with_comments(task, comments)
    }

    fn new_with_comments(task: &Task, comments: Vec<CommentView>) -> QueryResult<Self> {
        Ok(Self {
            id: task.id,
            title: task.title.to_owned(),
            description: task.description.to_owned(),
            finished: task.finished,
            comments,
            deadline: task.deadline,
            created: task.created,
        })
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &String {
        &(self.title)
    }

    pub fn description(&self) -> &String {
        &(self.description)
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn comments(&self) -> &Vec<CommentView> {
        &(self.comments)
    }

    pub fn deadline(&self) -> NaiveDate {
        self.deadline
    }

    pub fn created(&self) -> NaiveDateTime {
        self.created
    }
}

#[derive(Serialize)]
pub struct TaskViewList {
    tasks: Vec<TaskView>,
}

impl TaskViewList {
    pub fn new(tasks: Vec<Task>, conn: &mut PgConnection) -> QueryResult<Self> {
        let tasks = Comment::belonging_to(&tasks)
            .select(Comment::as_select())
            .load(conn)?
            .grouped_by(&tasks)
            .into_iter()
            .zip(tasks)
            .map(|(comments, task)| {
                let comment_view_list = comments.iter().map(|comment| CommentView::new(comment)).collect();
                TaskView::new_with_comments(&task, comment_view_list)
            })
            .collect::<QueryResult<Vec<TaskView>>>()?;
        Ok(Self {
            tasks,
        })
    }
}


#[derive(Associations, Deserialize, Identifiable, Selectable, Serialize, Queryable)]
#[diesel(table_name = schema::comment)]
#[diesel(belongs_to(Task))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comment {
    pub id: i32,
    pub content: String,
    pub task_id: i32,
    pub created: NaiveDateTime,
}

impl Comment {
    pub fn get(id: i32, conn: &mut PgConnection) -> Result<Comment, DieselError> {
        crate::schema::comment::dsl::comment.find(id).first::<Comment>(conn)
    }
}


#[derive(Serialize)]
pub struct CommentView {
    id: i32,
    content: String,
    task_id: i32,
    created: NaiveDateTime,
}

impl CommentView {
    pub fn new(comment: &Comment) -> Self {
        Self {
            id: comment.id,
            content: comment.content.clone(),
            task_id: comment.task_id,
            created: comment.created,
        }
    }
}


#[derive(Deserialize, Insertable, Serialize)]
#[diesel(table_name = schema::comment)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewComment {
    content: String,
    task_id: i32,
}

impl NewComment {
    pub fn new(content: String, task: &Task) -> Self {
        Self {
            content,
            task_id: task.id,
        }
    }

    pub fn insert(self, conn: &mut PgConnection) -> QueryResult<Comment> {
        // assert task_id is valid
        let _ = Task::get(self.task_id, conn)?;

        diesel::insert_into(crate::schema::comment::dsl::comment)
            .values(&self)
            .returning(Comment::as_returning())
            .get_result(conn)
    }
}


#[derive(Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = schema::tag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: i32,
    pub name: String,
}


#[derive(Associations, Deserialize, Selectable, Serialize, Queryable)]
#[diesel(table_name = schema::task_tag)]
#[diesel(belongs_to(Tag))]
#[diesel(belongs_to(Task))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskTag {
    pub tag_id: i32,
    pub task_id: i32,
}
