const apiUrl = "http://localhost:8080/api";

function checkbox(value, name, classNames=[]) {
    const element = document.createElement("div");

    const display = document.createElement("input");
    display.type = "checkbox";
    display.checked = value;
    for (const className of classNames) {
        display.classList.add(className);
    }
    element.appendChild(display);

    const form = document.createElement("input");
    form.type = "hidden";
    form.name = name;
    form.value = value;
    element.appendChild(form);

    display.addEventListener('change', () => {
        if (display.checked) {
            form.value = true;
        } else {
            form.value = false;
        }
    })

    return element;
}

function updateJSON(element, name, value) {
    const selectRadio = element.querySelector(`input[name="${name}"]:checked`);
    let jsonData = {};

    if (selectRadio) {
        const key = selectRadio.value;
        jsonData[key] = value;
    }

    return JSON.stringify(jsonData);
}

function TextFilter(name, label) {
    const element = document.createElement("div");

    const inputTextId = name + "-search-filter-text";
    const inputTextName = name + "-search-filter-text";
    const inputTextLabel = document.createElement("label");
    inputTextLabel.for = inputTextId;
    inputTextLabel.textContent = label;
    element.appendChild(inputTextLabel);

    const form = document.createElement("input");
    form.type = "hidden";
    form.name = name;
    form.value = JSON.stringify({});
    element.appendChild(form);

    const inputRadioName = name + "-search-filter-radio";

    for (const filterType of ["CONTAIN", "EQUAL"]) {
        const filterTypeId = name + "-search-filter-radio-" + filterType;
        const filterTypeInput = document.createElement("input");
        filterTypeInput.id = filterTypeId;
        filterTypeInput.type = "radio";
        filterTypeInput.name = inputRadioName;
        filterTypeInput.value = filterType;
        element.appendChild(filterTypeInput);
        const filterTypeLabel = document.createElement("label");
        filterTypeLabel.for = filterTypeId;
        filterTypeLabel.textContent = filterType;
        element.appendChild(filterTypeLabel);
    }

    const inputText = document.createElement("input");
    inputText.id = inputTextId;
    inputText.type = "text";
    inputText.name = inputTextName;
    element.appendChild(inputText);

    var lastSelected = null;
    element.addEventListener('click', function(event) {
        if (event.target.checked) {
            if (event.target == lastSelected) {
                event.target.checked = false;
                lastSelected = null;
            } else {
                lastSelected = event.target;
            }
        }
        form.value = updateJSON(element, inputRadioName, inputText.value);
    });

    inputText.addEventListener('change', function(event) {
        form.value = updateJSON(element, inputRadioName, inputText.value);
    })

    return element;
}

function DateFilter(name, label) {
    const element = document.createElement("div");

    const inputDateId = name + "-search-filter-date";
    const inputDateName = name + "-search-filter-date";
    const inputDateLabel = document.createElement("label");
    inputDateLabel.for = inputDateId;
    inputDateLabel.textContent = label;
    element.appendChild(inputDateLabel);

    const form = document.createElement("input");
    form.type = "hidden";
    form.name = name;
    form.value = JSON.stringify({});
    element.appendChild(form);

    const inputRadioName = name + "-search-filter-radio";

    for (const filterType of ["LT", "LE", "EQ", "GE", "GT"]) {
        const filterTypeId = name + "-search-filter-radio-" + filterType;
        const filterTypeInput = document.createElement("input");
        filterTypeInput.id = filterTypeId;
        filterTypeInput.type = "radio";
        filterTypeInput.name = inputRadioName;
        filterTypeInput.value = filterType;
        element.appendChild(filterTypeInput);
        const filterTypeLabel = document.createElement("label");
        filterTypeLabel.for = filterTypeId;
        filterTypeLabel.textContent = filterType;
        element.appendChild(filterTypeLabel);
    }

    const inputDate = document.createElement("input");
    inputDate.id = inputDateId;
    inputDate.type = "date";
    inputDate.name = inputDateName;
    element.appendChild(inputDate);

    var lastSelected = null;
    element.addEventListener('click', function(event) {
        if (event.target.checked) {
            if (event.target == lastSelected) {
                event.target.checked = false;
                lastSelected = null;
            } else {
                lastSelected = event.target;
            }
        }
        form.value = updateJSON(element, inputRadioName, inputDate.value);
    });

    inputDate.addEventListener('change', function(event) {
        form.value = updateJSON(element, inputRadioName, inputDate.value);
    })

    return element;
}

function BooleanFilter(name, label) {
    const element = document.createElement("div");

    const inputBooleanId = name + "-search-filter-boolean";
    const inputBooleanIdName = name + "-search-filter-boolean";
    const inputBooleanLabel = document.createElement("label");
    inputBooleanLabel.for = inputBooleanId;
    inputBooleanLabel.textContent = label;
    element.appendChild(inputBooleanLabel);

    const inputRadioName = name + "-search-filter-radio";

    for (const filterType of ["TRUE", "FALSE"]) {
        const filterTypeId = name + "-search-filter-radio-" + filterType;
        const filterTypeInput = document.createElement("input");
        filterTypeInput.id = filterTypeId;
        filterTypeInput.type = "radio";
        filterTypeInput.name = name;
        filterTypeInput.value = filterType;
        element.appendChild(filterTypeInput);
        const filterTypeLabel = document.createElement("label");
        filterTypeLabel.for = filterTypeId;
        filterTypeLabel.textContent = filterType;
        element.appendChild(filterTypeLabel);
    }

    var lastSelected = null;
    element.addEventListener('click', function(event) {
        if (event.target.checked) {
            if (event.target == lastSelected) {
                event.target.checked = false;
                lastSelected = null;
            } else {
                lastSelected = event.target;
            }
        }
    });

    return element;
}

function GetParsedModifiedTaskFormData(form) {
    const formData = new FormData(form);
    const parsedData = {};

    for (const [key, value] of formData.entries()) {
        const input = form.querySelector(`[name="${key}"]`);
        if (!input) continue;

        switch (key) {
            case "finished":
                parsedData[key] = value == "true";
                break;
            default:
                parsedData[key] = value;
                break;
        }
    }

    return parsedData;
}

function GetParsedNewCommentFormData(form) {
    const formData = new FormData(form);
    const parsedData = {};

    for (const [key, value] of formData.entries()) {
        const input = form.querySelector(`[name="${key}"]`);
        if (!input) continue;

        switch (key) {
            case "task_id":
                parsedData[key] = parseInt(value, 10);
                break;
            default:
                parsedData[key] = value;
                break;
        }
    }

    return parsedData;
}

function GetParsedTaskQueryFormData(form) {
    const formData = new FormData(form);
    const parsedData = {};
    const filter = {};
    var order = {
        "Deadline": "ASC"
    };

    for (const [key, value] of formData.entries()) {
        const input = form.querySelector(`[name="${key}"]`);
        if (!input) continue;

        switch (key) {
            case "page_no":
                parsedData[key] = parseInt(value, 10);
                break;
            case "per_page":
                parsedData[key] = parseInt(value, 10);
                break;
            case "title":
            case "description":
            case "deadline":
                const jsonValue = JSON.parse(value);
                if (Object.keys(jsonValue).length !== 0) {
                    filter[key] = jsonValue;
                }
                break;
            case "finished":
                filter[key] = value;
                break;
            case "order":
                order = JSON.parse(value);
                break;
            default:
                break;
        }
    }

    parsedData["filter"] = filter;
    parsedData["order"] = order;
    return parsedData;
}

const notificationArea = document.getElementById("notification-area");

function showNotification(message, messageType) {
    const notification = document.createElement("div");
    notification.classList.add("notification");
    switch (messageType) {
        case "info":
            notification.classList.add("notification-info");
            break;
        case "error":
            notification.classList.add("notification-error");
            break;
        default:
            console.log("message type error");
            break;
    }
    notification.textContent = message;

    notificationArea.appendChild(notification);

    setTimeout(() => {
        notification.classList.add("notification-show");
    }, 10);

    setTimeout(() => {
        notification.classList.add("notification-fade-out");
        setTimeout(() => {
            notification.remove();
        }, 300);
    }, 3000);
}

function generateCommentArticle(comment) {
    const commentItem = document.createElement("article");
    commentItem.className = "comment";

    const description = document.createElement("div");
    description.textContent = comment.content;
    commentItem.appendChild(description);

    return commentItem;
}

function generateCommentList(task) {
    const commentList = document.createElement("div");

    const comments = document.createElement("div");
    comments.className = "comments";
    task.comments.forEach(comment => {
        comments.appendChild(generateCommentArticle(comment));
    });
    commentList.appendChild(comments);

    const form = document.createElement("form");

    const id = document.createElement("input");
    id.type = "hidden";
    id.value = task.id;
    id.name = "task_id";
    form.appendChild(id);

    const content = document.createElement("input");
    content.type = "text";
    content.name = "content";
    form.appendChild(content);

    const addButton = document.createElement("button");
    addButton.textContent = "Add";
    addButton.type = "submit";

    form.addEventListener("submit", async function(event) {
        event.preventDefault();

        const jsonData = GetParsedNewCommentFormData(event.target);

        console.log("Add New Comment: ", jsonData);

        try {
            const response = await fetch(apiUrl + "/comment", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(jsonData)
            });


            if (!response.ok) {
                showNotification("Error occured(status " + response.status + ")", "error");
                throw new Error(`Error: ${response.status}`);
            }

            const result = await response.json();
            if (result.Info) {
                showNotification(result.Info.message, "info");
                console.log(result.Info);
                comments.appendChild(generateCommentArticle(result.Info.comment));
            } else if (result.Error) {
                showNotification(result.Error.message, "error");
            } else {
                showNotification("Error occured", "error");
            }
        } catch (error) {
            showNotification("Error occured: " + error, "error");
        }
    })

    form.appendChild(addButton);

    commentList.appendChild(form);

    return commentList;
}

function generateTaskArticle(task) {
    const taskItem = document.createElement("article");
    taskItem.className = "task";

    const header = document.createElement("header");
    const finished = checkbox(task.finished, "finished", ["finished"]);
    header.appendChild(finished);
    const title = document.createElement("h2");
    title.textContent = task.title;
    header.appendChild(title);
    taskItem.appendChild(header);

    const content = document.createElement("div");
    content.className = "content";

    const deadline = document.createElement("div");
    deadline.className = "deadline";
    deadline.textContent = task.deadline;
    content.appendChild(deadline);

    const description = document.createElement("div");
    description.className = "description";
    description.textContent = task.description;
    content.appendChild(description);

    content.appendChild(generateCommentList(task));

    header.addEventListener("click", function(event) {
        if (event.target.classList.contains("finished") || event.target.classList.contains("editButton")) {
            return;
        }

        if (content.style.maxHeight && content.style.maxHeight != "0px") {
            content.style.maxHeight = "0px";
        } else {
            content.style.maxHeight = content.scrollHeight + "px";
        }
    });

    const editButton = document.createElement("button");
    editButton.className = "editButton";
    editButton.textContent = "Edit";
    editButton.addEventListener("click", function (event) {
        event.stopPropagation();
        generateTaskEditForm(taskItem, task);
    });
    content.appendChild(editButton);

    const deleteModalWindow = document.createElement("dialog");
    const deleteModalWindowContent = document.createElement("div");
    deleteModalWindowContent.textContent = "Are you sure to delete?"
    deleteModalWindow.appendChild(deleteModalWindowContent);
    const deleteModalWindowDeleteButton = document.createElement("button");
    deleteModalWindowDeleteButton.textContent = "Delete";
    deleteModalWindowDeleteButton.className = "delete";
    deleteModalWindowDeleteButton.addEventListener("click", async function(event) {
        try {
            const response = await fetch(apiUrl + "/task/" + task.id, {
                method: "DELETE",
            });

            if (!response.ok) {
                showNotification("Error occured(status " + response.status + ")", "error");
                throw new Error(`Error: ${response.status}`);
            }

            const result = await response.json();
            if (result.Info) {
                showNotification(result.Info, "info");
                taskItem.remove();
                deleteModalWindow.close();
            } else if (result.Error) {
                showNotification(result.Error, "error");
                deleteModalWindow.close();
            } else {
                showNotification("Error occured", "error");
                deleteModalWindow.close();
            }
        } catch (error) {
            showNotification("Error occured: " + error, "error");
            deleteModalWindow.close();
        }
    })
    deleteModalWindow.appendChild(deleteModalWindowDeleteButton);
    const deleteModalWindowCancelButton = document.createElement("button");
    deleteModalWindowCancelButton.textContent = "Cancel";
    deleteModalWindowCancelButton.className = "cancel";
    deleteModalWindowCancelButton.autofocus = true;
    deleteModalWindowCancelButton.addEventListener("click", function(event) {
        deleteModalWindow.close();
    });
    deleteModalWindow.appendChild(deleteModalWindowCancelButton);
    content.appendChild(deleteModalWindow);

    const deleteButton = document.createElement("button");
    deleteButton.textContent = "Delete";
    deleteButton.addEventListener("click", function(event) {
        event.preventDefault();
        deleteModalWindow.showModal();
    });
    content.appendChild(deleteButton);

    taskItem.appendChild(content);

    return taskItem;
}

function generateTaskEditForm(taskItem, task) {
    taskItem.innerHTML = "";

    const form = document.createElement("form");

    const id = document.createElement("input");
    id.type = "hidden";
    id.value = task.id;
    id.name = "id";
    form.appendChild(id);

    const header = document.createElement("header");
    const finished = checkbox(task.finished, "finished", ["finished"]);
    header.appendChild(finished);
    const titleH2 = document.createElement("h2");
    const title = document.createElement("input");
    title.type = "text";
    title.value = task.title;
    title.name = "title";
    titleH2.appendChild(title);
    header.appendChild(titleH2);
    form.appendChild(header);

    const content = document.createElement("div");
    content.className = "content";

    const deadline = document.createElement("input");
    deadline.className = "deadline";
    deadline.type = "date"
    deadline.value = task.deadline;
    deadline.name = "deadline";
    content.appendChild(deadline);

    const description = document.createElement("input");
    description.className = "description";
    description.type = "text";
    description.value = task.description;
    description.name = "description";
    content.appendChild(description);

    form.appendChild(content);

    const saveButton = document.createElement("button");
    saveButton.textContent = "Save";
    saveButton.type = "submit";

    header.addEventListener("click", function() {
        if (event.target.classList.contains("finished") || event.target.classList.contains("editButton")) {
            return;
        }

        if (content.style.maxHeight && content.style.maxHeight != "0px") {
            content.style.maxHeight = "0px";
        } else {
            content.style.maxHeight = content.scrollHeight + "px";
        }
    });

    form.addEventListener("submit", async function(event) {
        event.preventDefault();

        const formData = new FormData(event.target);
        const pureJsonData = Object.fromEntries(formData.entries());
        const jsonData = GetParsedModifiedTaskFormData(event.target);

        console.log("Modify Response clicked:", jsonData);

        try {
            const response = await fetch(apiUrl + "/task/" + jsonData.id, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(jsonData)
            });

            if (!response.ok) {
                showNotification("Error occured(status " + response.status + ")", "error");
                throw new Error(`Error: ${response.status}`);
            }

            const result = await response.json();
            if (result.Info) {
                showNotification(result.Info.message, "info");
            } else if (result.Error) {
                showNotification(result.Error.message, "error");
            } else {
                showNotification("Error occured", "error");
            }
        } catch (error) {
            showNotification("Error occured: " + error, "error");
        }

        const updatedTask = { ...task, ...jsonData };
        const newTaskItem = generateTaskArticle(updatedTask);
        taskItem.replaceWith(newTaskItem);
    })
    form.appendChild(saveButton);

    taskItem.appendChild(form);

    content.style.maxHeight = content.scrollHeight + "px";
}

function createPagenatedTaskList(pagenatedTaskList) {
    console.log(pagenatedTaskList);
    const taskListElement = document.getElementById("task-list");
    taskListElement.innerHTML = "";

    pagenatedTaskList.task_view_list.tasks.forEach(item => {
        taskListElement.appendChild(generateTaskArticle(item));
    });
}

async function searchTask(jsonData) {
    try {
        const response = await fetch(apiUrl + "/task/search", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(jsonData)
        });

        if (!response.ok) {
            showNotification("Error occured(status " + response.status + ")", "error");
            throw new Error(`Error: ${response.status}`);
        }

        const result = await response.json();
        createPagenatedTaskList(result.Info.pagenated_task_list);
    } catch (error) {
        showNotification("Error occured: " + error, "error");
    }
}

async function createTaskQueryForm() {
    const taskQueryForm = document.getElementById("task-query-form");

    const titleFilter = TextFilter("title", "title");
    taskQueryForm.appendChild(titleFilter);

    const descriptionFilter = TextFilter("description", "description");
    taskQueryForm.appendChild(descriptionFilter);

    const deadlineFilter = DateFilter("deadline", "deadline");
    taskQueryForm.appendChild(deadlineFilter);

    const finishedFilter = BooleanFilter("finished", "finished");
    taskQueryForm.appendChild(finishedFilter);

    const pageNo = document.createElement("input");
    pageNo.type = "text";
    pageNo.inputmode = "decimal";
    pageNo.min = 1;
    pageNo.name = "page_no";
    pageNo.value = 1;
    taskQueryForm.appendChild(pageNo);

    const perPage = document.createElement("input");
    perPage.type = "text";
    perPage.inputmode = "decimal";
    perPage.min = 1;
    perPage.name = "per_page";
    perPage.value = 10;
    taskQueryForm.appendChild(perPage);

    const sortOrder = document.createElement("div");
    const sortOrderName = "order";
    var isSortOrderSelected = false;
    for (const [key, val] of [
        ["Deadline", "ASC"],
        ["Deadline", "DESC"],
        ["Created", "ASC"],
        ["Created", "DESC"]
    ]) {
        const sortOrderId = key + "-" + val + "-sort-order-radio";
        const sortOrderRadioInput = document.createElement("input");
        sortOrderRadioInput.id = sortOrderId;
        sortOrderRadioInput.type = "radio";
        sortOrderRadioInput.name = sortOrderName;
        if (!isSortOrderSelected) {
            sortOrderRadioInput.checked = true;
            isSortOrderSelected = true;
        }
        const value = {}
        value[key] = val
        sortOrderRadioInput.value = JSON.stringify(value);
        sortOrder.appendChild(sortOrderRadioInput);
        const sortOrderRadioLabel = document.createElement("label");
        sortOrderRadioLabel.for = sortOrderId;
        sortOrderRadioLabel.textContent = key + "(" + val + ")";
        sortOrder.appendChild(sortOrderRadioLabel);
    }
    taskQueryForm.appendChild(sortOrder);

    const searchButton = document.createElement("button");
    searchButton.className = "searchName";
    searchButton.textContent = "Search";
    searchButton.type = "submit";
    taskQueryForm.addEventListener("submit", async function (event) {
        event.preventDefault();

        const jsonData = GetParsedTaskQueryFormData(event.target);

        searchTask(jsonData);
    })
    taskQueryForm.appendChild(searchButton);
}

createTaskQueryForm();

async function fetchTaskList() {
    const query = {
        "filter": {
            "finished": "FALSE",
        },
        "order": {
            "Deadline": "ASC",
        },
        "page_no": 1,
        "per_page": 10,
    };
    searchTask(query);
}

fetchTaskList();

document.getElementById("task-register-form").addEventListener("submit", async function (event) {
    event.preventDefault();

    const formData = new FormData(event.target);
    const jsonData = Object.fromEntries(formData.entries());

    try {
        const response = await fetch(apiUrl + "/task", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(jsonData)
        });

        if (!response.ok) {
            showNotification("Error occured(status " + response.status + ")", "error");
            throw new Error(`Error: ${response.status}`);
        }

        const result = await response.json();
        if (result.Info) {
            showNotification(result.Info.message, "info");
        } else if (result.Error) {
            showNotification(result.Error.message, "error");
        } else {
            showNotification("Error occured", "error");
        }
    } catch (error) {
        console.error("Error:", error);
        showNotification("Error occured: " + error, "error");
    }
});
