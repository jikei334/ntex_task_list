// @generated automatically by Diesel CLI.

diesel::table! {
    comment (id) {
        id -> Int4,
        content -> Text,
        task_id -> Int4,
        created -> Timestamp,
    }
}

diesel::table! {
    tag (id) {
        id -> Int4,
        name -> Varchar,
        created -> Timestamp,
    }
}

diesel::table! {
    task (id) {
        id -> Int4,
        title -> Varchar,
        description -> Text,
        finished -> Bool,
        deadline -> Date,
        created -> Timestamp,
    }
}

diesel::table! {
    task_tag (task_id, tag_id) {
        task_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::joinable!(comment -> task (task_id));
diesel::joinable!(task_tag -> tag (tag_id));
diesel::joinable!(task_tag -> task (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    comment,
    tag,
    task,
    task_tag,
);
