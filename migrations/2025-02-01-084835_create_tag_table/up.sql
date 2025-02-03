CREATE TABLE tag (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE task_tag (
    task_id INTEGER REFERENCES task(id),
    tag_id INTEGER REFERENCES tag(id),
    PRIMARY KEY(task_id, tag_id)
)
