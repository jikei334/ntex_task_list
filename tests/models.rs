use chrono::{NaiveDateTime, NaiveDate};

use serde::{Deserialize, Serialize};


#[derive(Clone, Deserialize)]
pub struct TestTaskView {
    id: i32,
    title: String,
    description: String,
    finished: bool,
    comments: Vec<TestCommentView>,
    deadline: NaiveDate,
    created: NaiveDateTime,
}

impl TestTaskView {
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

    pub fn comments(&self) -> &Vec<TestCommentView> {
        &(self.comments)
    }

    pub fn deadline(&self) -> NaiveDate {
        self.deadline
    }

    pub fn created(&self) -> NaiveDateTime {
        self.created
    }
}

#[derive(Clone, Deserialize)]
pub struct TestTaskViewList {
    tasks: Vec<TestTaskView>,
}

impl TestTaskViewList {
    pub fn tasks(&self) -> &Vec<TestTaskView> {
        &(self.tasks)
    }
}

#[derive(Deserialize)]
pub struct TestPagenatedTaskList {
    task_view_list: TestTaskViewList,
    num_total_tasks: i64,
    page_no: i64,
}

impl TestPagenatedTaskList {
    pub fn task_view_list(&self) -> &TestTaskViewList {
        &(self.task_view_list)
    }

    pub fn num_total_tasks(&self) -> i64 {
        self.num_total_tasks
    }

    pub fn page_no(&self) -> i64 {
        self.page_no
    }
}

#[derive(Serialize)]
pub enum TestFilterOrdered<V>
{
    LT(V),
    LE(V),
    EQ(V),
    GE(V),
    GT(V),
}

#[derive(Serialize)]
pub enum TestFilterContainable<V> {
    CONTAIN(V),
    EQUAL(V),
}

#[derive(Serialize)]
pub enum TestFilterBoolean {
    FALSE,
    TRUE,
}

#[derive(Serialize)]
pub struct TestTaskFilter {
    title: Option<TestFilterContainable<String>>,
    description: Option<TestFilterContainable<String>>,
    finished: Option<TestFilterBoolean>,
    deadline: Option<TestFilterOrdered<NaiveDate>>,
    created: Option<TestFilterOrdered<NaiveDateTime>>,
}

impl TestTaskFilter {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            finished: None,
            deadline: None,
            created: None,
        }
    }

    pub fn set_title(mut self, title: TestFilterContainable<String>) -> Self {
        self.title = Some(title);
        self
    }

    pub fn set_description(mut self, description: TestFilterContainable<String>) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_finished(mut self, finished: TestFilterBoolean) -> Self {
        self.finished = Some(finished);
        self
    }

    pub fn set_deadline(mut self, deadline: TestFilterOrdered<NaiveDate>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn set_created(mut self, created: TestFilterOrdered<NaiveDateTime>) -> Self {
        self.created = Some(created);
        self
    }
}

#[derive(Serialize)]
pub enum TestSortOrder {
    ASC,
    DESC,
}

#[derive(Serialize)]
pub enum TestTaskOrder {
    Deadline(TestSortOrder),
    Created(TestSortOrder),
}

#[derive(Serialize)]
pub struct TestTaskQuery {
    pub filter: TestTaskFilter,
    pub order: TestTaskOrder,
    page_no: i64,
    per_page: i64,
}

impl TestTaskQuery {
    pub fn new(page_no: i64, per_page: i64) -> Self {
        Self {
            filter: TestTaskFilter::new(),
            order: TestTaskOrder::Deadline(TestSortOrder::ASC),
            page_no,
            per_page,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct TestCommentView {
    id: i32,
    content: String,
    task_id: i32,
    created: NaiveDateTime,
}

impl TestCommentView {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn content(&self) -> &String {
        &(self.content)
    }

    pub fn task_id(&self) -> i32 {
        self.task_id
    }
}

#[derive(Serialize)]
pub struct TestValidNewTask {
    title: String,
    description: String,
    deadline: NaiveDate,
}

impl TestValidNewTask {
    pub fn new(title: String, description: String, deadline: NaiveDate) -> Self {
        Self {
            title,
            description,
            deadline,
        }
    }

    pub fn title(&self) -> &String {
        &(self.title)
    }

    pub fn description(&self) -> &String {
        &(self.description)
    }

    pub fn deadline(&self) -> NaiveDate {
        self.deadline
    }
}

#[derive(Serialize)]
pub struct TestInvalidNewTask {
    pub fake_title: String,
}

impl TestInvalidNewTask {
    pub fn new(fake_title: String) -> Self {
        Self {
            fake_title
        }
    }
}

#[derive(Serialize)]
pub struct TestValidModifiedTask {
    pub title: String,
    pub description: String,
    pub finished: bool,
    pub deadline: NaiveDate,
}

impl TestValidModifiedTask {
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
}

#[derive(Serialize)]
pub struct TestValidNewComment {
    content: String,
    task_id: i32,
}

impl TestValidNewComment {
    pub fn new(content: String, task_id: i32) -> Self {
        Self {
            content,
            task_id,
        }
    }

    pub fn content(&self) -> &String {
        &(self.content)
    }

    pub fn task_id(&self) -> i32 {
        self.task_id
    }
}

#[derive(Serialize)]
pub struct TestInvalidNewComment {
    cotnent: String,
    task_id: i32,
}

impl TestInvalidNewComment {
    pub fn new(cotnent: String, task_id: i32) -> Self {
        Self {
            cotnent,
            task_id,
        }
    }
}

#[derive(Deserialize)]
pub enum TestSimpleMessage {
    Info(String),
    Error(String),
}

#[derive(Deserialize)]
pub struct TestTaskInfoMessage {
    message: String,
    task: TestTaskView,
}

impl TestTaskInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn task(&self) -> &TestTaskView {
        &(self.task)
    }
}

#[derive(Deserialize)]
pub struct TestTaskErrorMessage {
    message: String,
}

impl TestTaskErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Deserialize)]
pub enum TestTaskMessage {
    Info(TestTaskInfoMessage),
    Error(TestTaskErrorMessage),
}

#[derive(Deserialize)]
pub struct TestPagenatedTaksListInfoMessage {
    message: String,
    pagenated_task_list: TestPagenatedTaskList,
}

impl TestPagenatedTaksListInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn pagenated_task_list(&self) -> &TestPagenatedTaskList {
        &(self.pagenated_task_list)
    }
}

#[derive(Deserialize)]
pub struct TestPagenatedTaksListErrorMessage {
    message: String,
}

impl TestPagenatedTaksListErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Deserialize)]
pub enum TestPagenatedTaskListMessage {
    Info(TestPagenatedTaksListInfoMessage),
    Error(TestPagenatedTaksListErrorMessage),
}

#[derive(Deserialize)]
pub struct TestCommentInfoMessage {
    message: String,
    comment: TestCommentView,
}

impl TestCommentInfoMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }

    pub fn comment(&self) -> &TestCommentView {
        &(self.comment)
    }
}

#[derive(Deserialize)]
pub struct TestCommentErrorMessage {
    message: String,
}

impl TestCommentErrorMessage {
    pub fn message(&self) -> &String {
        &(self.message)
    }
}

#[derive(Deserialize)]
pub enum TestCommentMessage {
    Info(TestCommentInfoMessage),
    Error(TestCommentErrorMessage),
}
