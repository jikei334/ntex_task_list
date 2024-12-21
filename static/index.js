const apiUrl = "http://localhost:8080/api";

async function fetchTaskList() {
    try {
        const response = await fetch(apiUrl + "/task")
        if (!response.ok) throw new Error("データの取得に失敗しました");

        console.log(response);

        const data = await response.json();
        const taskListElement = document.getElementById("task-list");

        console.log(data.tasks);

        data.tasks.forEach(item => {
            const taskItem = document.createElement("li");
            taskItem.textContent = `${item.title}`
            taskListElement.appendChild(taskItem);
        });
    } catch (error) {
        console.log(error);
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
            throw new Error(`Error: ${response.status}`);
        }

        const result = await response.json();
        console.log("Response:", result);
    } catch (error) {
        console.error("Error:", error);
    }
});
