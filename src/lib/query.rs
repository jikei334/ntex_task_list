use chrono::{NaiveDate, NaiveDateTime};
use diesel::backend::Backend;
use diesel::dsl::Filter;
use diesel::pg::PgConnection;
use diesel::query_builder::BoxedSqlQuery;
use diesel::query_dsl::QueryDsl;
use diesel::query_dsl::methods::FilterDsl;
use diesel::result::Error as DieselError;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, TextExpressionMethods};
use serde::{Deserialize, Serialize};

use crate::schema::{comment, task};
use crate::models::{Task, TaskView, TaskViewList};


#[derive(Deserialize, Serialize)]
pub enum FilterOrderd<V>
{
    LT(V),
    LE(V),
    EQ(V),
    GE(V),
    GT(V),
}

#[derive(Deserialize, Serialize)]
pub enum FilterContainable<V: Sized> {
    CONTAIN(V),
    EQUAL(V),
}

#[derive(Deserialize, Serialize)]
pub enum FilterBoolean {
    FALSE,
    TRUE,
}

#[derive(Deserialize, Serialize)]
pub struct TaskFilter {
    title: Option<FilterContainable<String>>,
    description: Option<FilterContainable<String>>,
    finished: Option<FilterBoolean>,
    deadline: Option<FilterOrderd<NaiveDate>>,
    created: Option<FilterOrderd<NaiveDateTime>>,
}

impl<'a> TaskFilter {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            finished: None,
            deadline: None,
            created: None,
        }
    }

    pub fn set_title(mut self, title: FilterContainable<String>) -> Self {
        self.title = Some(title);
        self
    }

    pub fn set_description(mut self, description: FilterContainable<String>) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_finished(mut self, finished: FilterBoolean) -> Self {
        self.finished = Some(finished);
        self
    }

    pub fn set_deadline(mut self, deadline: FilterOrderd<NaiveDate>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn set_created(mut self, created: FilterOrderd<NaiveDateTime>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn filter(&self, query: task::BoxedQuery<'a, diesel::pg::Pg>) -> task::BoxedQuery<'a, diesel::pg::Pg>
    {
        let query = match &self.title {
            Some(filter_containable) => match filter_containable {
                FilterContainable::CONTAIN(v) => QueryDsl::filter(query, task::title.like(format!("%{}%", v).clone())),
                FilterContainable::EQUAL(v) => QueryDsl::filter(query, task::title.eq(v.clone())),
            },
            None => query,
        };

        let query = match &self.description {
            Some(filter_containable) => match filter_containable {
                FilterContainable::CONTAIN(v) => QueryDsl::filter(query, task::description.like(format!("%{}%", v).clone())),
                FilterContainable::EQUAL(v) => QueryDsl::filter(query, task::description.eq(v.clone())),
            },
            None => query,
        };

        let query = match &self.finished {
            Some(filter_boolean) => match filter_boolean {
                FilterBoolean::TRUE => QueryDsl::filter(query, task::finished.eq(true)),
                FilterBoolean::FALSE => QueryDsl::filter(query, task::finished.eq(false)),
            },
            None => query,
        };

        let query = match &self.deadline {
            Some(filter_ordered) => match filter_ordered {
                FilterOrderd::LT(v) => QueryDsl::filter(query, task::deadline.lt(v.clone())),
                FilterOrderd::LE(v) => QueryDsl::filter(query, task::deadline.le(v.clone())),
                FilterOrderd::EQ(v) => QueryDsl::filter(query, task::deadline.eq(v.clone())),
                FilterOrderd::GE(v) => QueryDsl::filter(query, task::deadline.ge(v.clone())),
                FilterOrderd::GT(v) => QueryDsl::filter(query, task::deadline.gt(v.clone())),
            },
            None => query,
        };

        let query = match &self.created {
            Some(filter_ordered) => match filter_ordered {
                FilterOrderd::LT(v) => QueryDsl::filter(query, task::created.lt(v.clone())),
                FilterOrderd::LE(v) => QueryDsl::filter(query, task::created.le(v.clone())),
                FilterOrderd::EQ(v) => QueryDsl::filter(query, task::created.eq(v.clone())),
                FilterOrderd::GE(v) => QueryDsl::filter(query, task::created.ge(v.clone())),
                FilterOrderd::GT(v) => QueryDsl::filter(query, task::created.gt(v.clone())),
            },
            None => query,
        };

        query
    }
}

#[derive(Deserialize, Serialize)]
pub struct CommentFilter {
    content: Option<FilterContainable<String>>,
    created: Option<FilterOrderd<NaiveDateTime>>,
}

impl<'a> CommentFilter {
    pub fn new() -> Self {
        Self {
            content: None,
            created: None,
        }
    }

    pub fn set_content(mut self, content: FilterContainable<String>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn set_created(mut self, created: FilterOrderd<NaiveDateTime>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn filter(&self, query: comment::BoxedQuery<'a, diesel::pg::Pg>) -> comment::BoxedQuery<'a, diesel::pg::Pg>
    {
        let query = match &self.content {
            Some(filter_containable) => match filter_containable {
                FilterContainable::CONTAIN(v) => QueryDsl::filter(query, comment::content.like(format!("%{}%", v).clone())),
                FilterContainable::EQUAL(v) => QueryDsl::filter(query, comment::content.eq(v.clone())),
            },
            None => query,
        };

        let query = match &self.created {
            Some(filter_ordered) => match filter_ordered {
                FilterOrderd::LT(v) => QueryDsl::filter(query, comment::created.lt(v.clone())),
                FilterOrderd::LE(v) => QueryDsl::filter(query, comment::created.le(v.clone())),
                FilterOrderd::EQ(v) => QueryDsl::filter(query, comment::created.eq(v.clone())),
                FilterOrderd::GE(v) => QueryDsl::filter(query, comment::created.ge(v.clone())),
                FilterOrderd::GT(v) => QueryDsl::filter(query, comment::created.gt(v.clone())),
            },
            None => query,
        };

        query
    }
}

#[derive(Deserialize, Serialize)]
pub enum SortOrder {
    ASC,
    DESC,
}

#[derive(Deserialize, Serialize)]
pub enum TaskOrder {
    Deadline(SortOrder),
    Created(SortOrder),
}

impl TaskOrder {
    pub fn order<'a>(&self, query: task::BoxedQuery<'a, diesel::pg::Pg>) -> task::BoxedQuery<'a, diesel::pg::Pg> {
        match &self {
            TaskOrder::Deadline(sort_order) => match sort_order {
                SortOrder::ASC => query.order(task::deadline.asc()),
                SortOrder::DESC => query.order(task::deadline.desc()),
            },
            TaskOrder::Created(sort_order) => match sort_order {
                SortOrder::ASC => query.order(task::created.asc()),
                SortOrder::DESC => query.order(task::created.desc()),
            },
        }
    }
}

#[derive(Serialize)]
pub struct PagenatedTaskList {
    task_view_list: TaskViewList,
    num_total_tasks: i64,
    page_no: i64,
}

impl PagenatedTaskList {
    pub fn task_view_list(&self) -> &TaskViewList {
        &(self.task_view_list)
    }

    pub fn num_total_tasks(&self) -> i64 {
        self.num_total_tasks
    }

    pub fn page_no(&self) -> i64 {
        self.page_no
    }
}

#[derive(Deserialize, Serialize)]
pub struct TaskQuery {
    pub filter: TaskFilter,
    pub order: TaskOrder,
    page_no: i64,
    per_page: i64,
}

impl TaskQuery {
    pub fn new(page_no: i64, per_page: i64) -> Self {
        Self {
            filter: TaskFilter::new(),
            order: TaskOrder::Deadline(SortOrder::ASC),
            page_no,
            per_page,
        }
    }

    fn filtered<'a>(&self, query: task::BoxedQuery<'a, diesel::pg::Pg>) -> task::BoxedQuery<'a, diesel::pg::Pg> {
        self.filter.filter(query)
    }

    fn ordered<'a>(&self, query: task::BoxedQuery<'a, diesel::pg::Pg>) -> task::BoxedQuery<'a, diesel::pg::Pg> {
        self.order.order(query)
    }

    fn get_query(&self) -> task::BoxedQuery<diesel::pg::Pg> {
        let query = task::table.into_boxed();

        query
    }

    pub fn get_pagenated_tasks(&self, conn: &mut PgConnection) -> Result<PagenatedTaskList, DieselError> {
        let query = self.filtered(self.get_query());
        let num_total = query.count().get_result(conn)?;
        let query = self.ordered(self.filtered(self.get_query()));
        let tasks = query
            .limit(self.per_page)
            .offset(self.per_page * (self.page_no - 1))
            .load::<Task>(conn)?;
        let task_view_list = TaskViewList::new(tasks, conn)?;

        Ok(PagenatedTaskList {
            task_view_list,
            num_total_tasks: num_total,
            page_no: self.page_no,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub enum CommentOrder {
    Created(SortOrder),
}

impl CommentOrder {
    pub fn order<'a>(&self, query: comment::BoxedQuery<'a, diesel::pg::Pg>) -> comment::BoxedQuery<'a, diesel::pg::Pg> {
        match &self {
            CommentOrder::Created(sort_order) => match sort_order {
                SortOrder::ASC => query.order(comment::created.asc()),
                SortOrder::DESC => query.order(comment::created.desc()),
            },
        }
    }
}
