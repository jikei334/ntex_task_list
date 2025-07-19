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

async function fetchTaskList() {
    try {
        const response = await fetch(apiUrl + "/task")
        if (!response.ok) {
            showNotification("Failed to get tasks");
            throw new Error("データの取得に失敗しました");
        }

        console.log(response);

        const data = await response.json();
        const taskListElement = document.getElementById("task-list");

        console.log(data.tasks);

        data.tasks.forEach(item => {
            taskListElement.appendChild(generateTaskArticle(item));
        });
    } catch (error) {
        showNotification("Error occured: " + error, "error");
    }
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
