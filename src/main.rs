use clap::{App, Arg};
use rusqlite;
use rusqlite::params;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::string::String;

const DB_PATH: &str = "taskbook.db";

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
    for task in taskbook.tasks {}
}

// #[derive(Serialize, Deserialize)]
struct TaskBook {
    tasks: HashMap<u32, Task>,
    next_id: u32,
    db_conn: Option<rusqlite::Connection>,
}

impl TaskBook {
    fn new() -> TaskBook {
        TaskBook {
            tasks: HashMap::new(),
            next_id: 0,
            db_conn: None,
        }
    }

    fn new_task(&mut self, content: String) -> u32 {
        let t = Task::new(self.next_id, content);
        match &self.db_conn {
            Some(conn) => {
                let r = conn.execute(
                    "insert into task (id, content, state)
                        values (?1, ?2, ?3)",
                    params![t.id, &t.content, &t.state.to_string()],
                );
                match r {
                    Ok(_) => (),
                    Err(e) => panic!("failed to insert task into database: {}", e.description()),
                }
            }
            None => panic!("The database is not initialized"),
        }
        self.next_id += 1;
        t.id
    }

    fn set_task_state(&mut self, ids: Vec<u32>, state: TaskState) {
        let mut ids_str: String = String::new();
        for id in ids {
            ids_str.push_str(&format!(" '{}'", id.to_string()));
        }
        let sqlstr = format!(
            "
            update task set state={} where id in ({})
        ",
            state.to_string(),
            ids_str
        );
        if let Some(conn) = &self.db_conn {
            conn.execute(&sqlstr, rusqlite::NO_PARAMS).unwrap();
        } else {
            panic!("database is not connected!");
        }
    }

    fn load_data(&mut self) {}

    fn find_task(&self, id: u32) -> Option<Task> {
        match &self.db_conn {
            Some(conn) => {
                let mut stmt = conn.prepare("select * from task where id=?").unwrap();
                let mut task_iter = stmt
                    .query_map(params![id], |row| {
                        Ok(Task {
                            id: row.get(0).unwrap(),
                            content: row.get(1).unwrap(),
                            state: row.get(2).unwrap(),
                        })
                    })
                    .unwrap();
                if let Some(Ok(t)) = task_iter.next() {
                    return Some(t);
                }
            }
            None => panic!("fail to connect to database."),
        };
        None
    }

    fn save_data(&self) {}

    fn init_db_conn(&mut self) {
        match rusqlite::Connection::open(DB_PATH) {
            Ok(conn) => {
                let r = conn.execute(
                    "create table if not exists task (
                    id integer primary key autoincrement,
                    content varchar(200),
                    state varchar(8))",
                    rusqlite::NO_PARAMS,
                );
                match r {
                    Ok(_) => self.db_conn = Some(conn),
                    Err(e) => panic!("fail to create db table: {}", e.description()),
                }
            }
            Err(e) => panic!("fail to open database: {}", e.description()),
        }
    }

    fn del_task(&self, id: u32) -> Result<u32, String> {
        if let Some(conn) = &self.db_conn {
            let r = conn.execute("delete from task where id=?", params![id]);
            match r {
                Ok(_) => return Ok(id),
                Err(e) => return Err(format!("fail to delete record: {}", e.description())),
            }
        }
        Err(String::from("fail to connect to database"))
    }
}

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

#[derive(Debug, PartialEq)]
enum TaskState {
    DONE,
    DOING,
    DEAD,
}

#[derive(Debug, Clone)]
struct UnknownTaskStateError;

impl fmt::Display for UnknownTaskStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown task state")
    }
}

impl std::error::Error for UnknownTaskStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl FromSql for TaskState {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(b"DEAD") => Ok(TaskState::DEAD),
            ValueRef::Text(b"DOING") => Ok(TaskState::DOING),
            ValueRef::Text(b"DONE") => Ok(TaskState::DONE),
            ValueRef::Text(_state) => Err(FromSqlError::Other(Box::new(UnknownTaskStateError))),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskState::DONE => write!(f, "DONE"),
            TaskState::DOING => write!(f, "DOING"),
            TaskState::DEAD => write!(f, "DEAD"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sqlite_create_table() {
        let mut tb = TaskBook::new();
        tb.init_db_conn();
    }

    #[test]
    fn sqlite_insert_task() {
        let mut tb = TaskBook::new();
        tb.init_db_conn();
        tb.new_task(String::from("hello world test test test"));

        let conn = tb.db_conn.unwrap();

        if let Ok(count) = conn.execute(
            "select * from task where content='hello world test test test'",
            rusqlite::NO_PARAMS,
        ) {
            assert!(true, count > 0);
        }

        conn.execute(
            "delete from task where content='hello world test test test'",
            rusqlite::NO_PARAMS,
        )
        .unwrap();
    }

    #[test]
    fn update_task_state() {
        let mut tb = TaskBook::new();
        tb.init_db_conn();
        let task_id01 = tb.new_task(String::from("test task zero"));
        let task01 = tb.find_task(task_id01).unwrap();
        assert_eq!(task01.state, TaskState::DOING);
        tb.set_task_state(vec![task_id01], TaskState::DONE);
        let task01 = tb.find_task(task_id01).unwrap();
        assert_eq!(task01.state, TaskState::DONE);
        let r = tb.del_task(task01.id);
        assert_eq!(r, Ok(task01.id));
    }

}
