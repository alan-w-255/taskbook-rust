use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use clap::{Arg, App};
use std::io::BufWriter;

const CONFIGURATION_FILE:&str = "taskbook-rust.json";

fn main() {
    let mut taskbook = TaskBook::new();
    taskbook.load_data();

    let matches = App::new("taskbook-rust")
        .version("1.0.0")
        .author("alan wong heywym@qq.com")
        .about("manage tasks in termial")
        .arg(Arg::with_name("new task")
            .short("n")
            .long("new")
            .value_name("task")
            .help("create new task")
            .takes_value(true))
        .arg(Arg::with_name("check")
            .short("c")
            .long("check")
            .value_name("task_ids")
            .help("mark task as checked")
            .takes_value(true)
            .multiple(true))
        .arg(Arg::with_name("uncheck")
            .short("u")
            .long("uncheck")
            .help("set task as uncheck")
            .value_name("task_ids")
            .takes_value(true)
            .multiple(true))
        .before_help("show tasks")
        .get_matches();

    if let Some(new_task) = matches.value_of("new task") {
        taskbook.new_task(new_task.to_string());
        taskbook.save_data();
        return;
    }
    if let Some(task_ids) = matches.values_of("check") {
        let ids: Vec<u32> = task_ids.map( |x| x.parse().unwrap()).collect();
        taskbook.set_task_state(ids, TaskState::DONE);
        taskbook.save_data();
        return;
    }
    if let Some(task_ids) = matches.values_of("uncheck") {
        let ids: Vec<u32> = task_ids.map( |x| x.parse().unwrap()).collect();
        taskbook.set_task_state(ids, TaskState::DOING);
        taskbook.save_data();
        return;
    }
    for task in taskbook.tasks {
        println!("{}", serde_json::to_string_pretty(&task.1).unwrap());
    }
}

#[derive(Serialize, Deserialize)]
struct TaskBook {
    tasks: HashMap<u32, Task>,
    next_id: u32,
}

impl TaskBook {
    fn new() -> TaskBook {
        TaskBook {
            tasks: HashMap::new(),
            next_id: 0,
        }
    }

    fn new_task(&mut self, content: String) {
        let t = Task::new(self.next_id, content);
        self.tasks.insert(self.next_id, t);
        self.next_id += 1;
    }

    fn set_task_state(&mut self, ids: Vec<u32>, state:TaskState) {
        for id in ids {
            self.tasks.entry(id).and_modify(|x| x.set_state(state));
        }
    }

    fn load_data(&mut self) {
        let path = Path::new(CONFIGURATION_FILE);
        if path.is_file() {
            let file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", path.display(), why.description()),
                Ok(file) => file,
            };
            let taskbook: TaskBook = match serde_json::from_reader(file) {
                Err(why) => panic!("can't read data from file: {}", why.description()),
                Ok(tb) => tb,
            };
            self.tasks = taskbook.tasks;
            self.next_id = taskbook.next_id;
            return;
        }
        let file = match File::create(path) {
            Err(why) => panic!("couldn't create {}: {}", path.display(), why.description()),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        match serde_json::to_writer(writer, self) {
            Err(why) => panic!("fail to save json data {}", why.description()),
            _ => (),
        }
    }

    fn save_data(&self) {
        let path = Path::new(CONFIGURATION_FILE);
        let file = match File::create(path) {
            Err(why) => panic!("couldn't save file {}: {}", path.display(), why.description()),
            Ok(file) => file,
        };
        let writer = BufWriter::new(file);
        match serde_json::to_writer(writer, self) {
            Err(why) => panic!("fail to save json data {}", why.description()),
            _ => (),
        }
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