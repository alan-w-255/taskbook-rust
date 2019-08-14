use clap::{App, Arg};
use rusqlite::{Row, params, Connection, NO_PARAMS, Result as SQL_Result, MappedRows};
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const CONFIGURATION_FILE: &str = "taskbook-rust.json";

fn main() {
    let mut taskbook = TaskBook::new();
    taskbook.load_data();

    let matches = App::new("taskbook-rust")
        .version("1.0.0")
        .author("alan wong heywym@qq.com")
        .about("manage tasks in termial")
        .arg(
            Arg::with_name("new task")
                .short("n")
                .long("new")
                .value_name("task")
                .help("create new task")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("check")
                .short("c")
                .long("check")
                .value_name("task_ids")
                .help("mark task as checked")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("uncheck")
                .short("u")
                .long("uncheck")
                .help("set task as uncheck")
                .value_name("task_ids")
                .takes_value(true)
                .multiple(true),
        )
        .before_help("show tasks")
        .get_matches();

    if let Some(new_task) = matches.value_of("new task") {
        taskbook.new_task(new_task.to_string());
        taskbook.save_data();
        return;
    }
    if let Some(task_ids) = matches.values_of("check") {
        let ids: Vec<u32> = task_ids.map(|x| x.parse().unwrap()).collect();
        taskbook.set_task_state(ids, TaskState::DONE);
        taskbook.save_data();
        return;
    }
    if let Some(task_ids) = matches.values_of("uncheck") {
        let ids: Vec<u32> = task_ids.map(|x| x.parse().unwrap()).collect();
        taskbook.set_task_state(ids, TaskState::DOING);
        taskbook.save_data();
        return;
    }
    for task in taskbook.tasks {
        println!("{}", serde_json::to_string_pretty(&task.1).unwrap());
    }
}

struct TaskBook {
    db_conn: Option<Connection>,
    next_id: u32,
}

impl TaskBook {
    fn new() -> TaskBook {
        TaskBook {
            db_conn: None,
            next_id: 0,
        }
    }

    fn new_task(&mut self, content: String) {
        let t = Task::new(self.next_id, content);
        self.tasks.insert(self.next_id, t);
        self.next_id += 1;
    }

    fn set_task_state(&mut self, ids: Vec<u32>, state: TaskState) {
        for id in ids {
            self.tasks.entry(id).and_modify(|x| x.set_state(state));
        }
    }

    fn init_db(&mut self) -> SQL_Result<()> {
        let db_path = Path::new(CONFIGURATION_FILE);
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE task (
            id integer primary key,
            content TEXT NOT NULL,
            state TEXT NOT NULL)",
            NO_PARAMS)?;
        self.db_conn = Some(conn);
        Ok(())
    }

    fn find_all_task<F>(&self) ->Result<MappedRows<F>, &str> 
        where F: FnMut(&Row<'_>) -> Result<Task, &'static str>
    {
        if let None = self.db_conn {
            return Err("fail to connect to db");
        }
        let conn = self.db_conn.unwrap();
        let mut stmt = conn.prepare("SELECT id, content, state FROM task").unwrap();
        let task_iter = stmt.query_map(params![], |row| {
                    Ok(Task{ 
                        id: row.get(0).unwrap(),
                        content: row.get(1).unwrap(),
                        state: row.get(2).unwrap(),
                    })
                }).unwrap();
        Ok(task_iter)
    }

    fn save_data(&self) {
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    content: String,
    state: TaskState,
}

impl Task {
    fn new(id: u32, content: String) -> Task {
        Task {
            id,
            content,
            state: TaskState::DOING,
        }
    }
    fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
enum TaskState {
    DONE,
    DOING,
    DEAD,
}

impl FromSql for TaskState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(|v| {
            match &*v {
                "DOING" => TaskState::DOING,
                "DONE" => TaskState::DONE,
                "DEAD" => TaskState::DEAD,
            }
        })
    }
}
