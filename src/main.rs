use std::{
    fs,
    io::{Read, Write},
    path,
};

use chrono::{DateTime, Local, Utc};

const STORE_PATH: &str = "progress.store";

enum ParseState {
    WritingMetadata(u8),
    WritingTask(u8),
}

struct Task {
    id: u32,
    done: bool,
    label: String,
    date_created: i64,
    date_checked: Option<i64>,
}

impl Task {
    fn default() -> Self {
        return Task {
            id: 0,
            done: false,
            label: "".to_string(),
            date_checked: None,
            date_created: 0,
        };
    }

    fn dump(&self, buffer: &mut String) {
        let date_checked = if let Some(n) = &self.date_checked {
            n.to_string()
        } else {
            "-".to_string()
        };

        buffer.push_str(":task\n");
        buffer.push_str(format!("{}\n", self.id.to_string()).as_str());
        buffer.push_str(if self.done { "[x]\n" } else { "[]\n" });
        buffer.push_str(format!("{}\n", &self.label).as_str());
        buffer.push_str(format!("{}\n", &self.date_created).as_str());
        buffer.push_str(&date_checked);
        buffer.push_str("\n:end\n");
    }
}

#[derive(Default)]
struct Metadata {
    last_task_id: u32,
}

impl Metadata {
    fn dump(&self, buffer: &mut String) {
        assert!(buffer.is_empty());
        buffer.push_str(":metadata\n");
        buffer.push_str(format!("{}\n", self.last_task_id).as_str());
        buffer.push_str(":end\n");
    }
}

struct Store {
    metadata: Metadata,
    tasks: Vec<Task>,
}

impl Store {
    fn save(&self) {
        let file_path = path::Path::new(STORE_PATH);

        let mut content_buffer = String::new();

        // dump metadata
        self.metadata.dump(&mut content_buffer);
        self.tasks
            .iter()
            .for_each(|task| task.dump(&mut content_buffer));

        let mut file = fs::File::create(file_path).expect("Could not create store file");
        file.write(content_buffer.as_bytes())
            .expect("Could not write to store");
    }

    fn open() -> Result<Self, String> {
        let file_path = path::Path::new(STORE_PATH);

        if !file_path.exists() {
            return Ok(Store {
                metadata: Metadata::default(),
                tasks: vec![],
            });
        }

        // open file
        let mut file = fs::File::open(file_path).map_err(|e| e.to_string())?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|e| e.to_string())?;

        // parse content
        let lines = buf.split('\n').collect::<Vec<&str>>();
        let mut metadata = Metadata::default();
        let mut state: Option<ParseState> = None;
        let mut tasks: Vec<Task> = vec![];

        for line in lines {
            if line.len() == 0 {
                continue;
            }

            match line {
                ":metadata" => {
                    state = Some(ParseState::WritingMetadata(0));
                }
                ":task" => {
                    state = Some(ParseState::WritingTask(0));
                    let task = Task::default();
                    tasks.push(task);
                }
                ":end" => {
                    // ensure the pointers ended well
                    if let Some(s) = state {
                        match s {
                            ParseState::WritingMetadata(n) => {
                                if n != 1 {
                                    return Err("Premature eol for metadata".to_string());
                                }
                            }
                            ParseState::WritingTask(n) => {
                                if n != 5 {
                                    return Err("Premature eol for task".to_string());
                                }
                            }
                        }
                    }
                    state = None;
                }
                _ => {
                    let status = state.take();
                    if let Some(mut status) = status {
                        match status {
                            ParseState::WritingMetadata(pointer) => match pointer {
                                0 => {
                                    metadata.last_task_id =
                                        line.trim().parse::<u32>().map_err(|e| e.to_string())?;
                                    status = ParseState::WritingMetadata(1);
                                }
                                _ => {
                                    return Err("invalid metadata pointer".to_string());
                                }
                            },
                            ParseState::WritingTask(pointer) => match pointer {
                                0 => {
                                    let task = tasks.pop();
                                    if let Some(mut task) = task {
                                        task.id = line
                                            .trim()
                                            .parse::<u32>()
                                            .map_err(|e| e.to_string())?;
                                        tasks.push(task);
                                    } else {
                                        unreachable!();
                                    }
                                    status = ParseState::WritingTask(1);
                                }
                                1 => {
                                    let task = tasks.pop();
                                    if let Some(mut task) = task {
                                        task.done = match line.trim() {
                                            "[x]" => true,
                                            "[]" => false,
                                            n => {
                                                return Err(format!(
                                                    "Invalid value {n} for task property"
                                                ))
                                            }
                                        };
                                        tasks.push(task);
                                    } else {
                                        unreachable!();
                                    }
                                    status = ParseState::WritingTask(2);
                                }
                                2 => {
                                    let task = tasks.pop();
                                    if let Some(mut task) = task {
                                        task.label = line.trim().to_string();
                                        tasks.push(task);
                                    } else {
                                        unreachable!();
                                    }
                                    status = ParseState::WritingTask(3);
                                }
                                3 => {
                                    let task = tasks.pop();
                                    if let Some(mut task) = task {
                                        task.date_created = line
                                            .trim()
                                            .parse::<i64>()
                                            .map_err(|e| e.to_string())?;
                                        tasks.push(task);
                                    } else {
                                        unreachable!();
                                    }
                                    status = ParseState::WritingTask(4);
                                }
                                4 => {
                                    let task = tasks.pop();
                                    if let Some(mut task) = task {
                                        task.date_checked = match line.trim() {
                                            "-" => None,
                                            n => Some(n.parse::<i64>().map_err(|e| e.to_string())?),
                                        };
                                        tasks.push(task);
                                    } else {
                                        unreachable!();
                                    }
                                    status = ParseState::WritingTask(5);
                                }
                                _ => {
                                    return Err("invalid task pointer".to_string());
                                }
                            },
                        }
                        state = Some(status);
                    }
                }
            }
        }

        return Ok(Store { metadata, tasks });
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.metadata.last_task_id += 1;
        self.save();
    }

    fn remove_task(&mut self, id: u32) -> Result<(), &str> {
        for (index, task) in self.tasks.iter().enumerate() {
            if task.id != id {
                continue;
            }
            // we cant remove a task if it wasnt added today
            let today = Local::now().date_naive();
            let task_created_date = DateTime::from_timestamp(task.date_created, 0).unwrap();

            if task_created_date.date_naive() == today {
                self.tasks.swap_remove(index);
            } else {
                return Err("Cannot remove task that wasn't added today");
            }
            break;
        }
        return Ok(());
    }

    fn toggle_check_task(&mut self, id: u32, check: bool) -> Result<(), &str> {
        for task in self.tasks.iter_mut() {
            if task.id != id {
                continue;
            }

            if task.done == check {
                return Err(if check {
                    "Task already done"
                } else {
                    "Task not completed yet"
                });
            }

            if check {
                // task
                let now = Utc::now().timestamp();
                task.done = true;
                task.date_checked = Some(now);
            } else {
                // we cant uncheck task if not the same day
                let today = Local::now().date_naive();
                let task_created_date = DateTime::from_timestamp(task.date_created, 0).unwrap();

                if task_created_date.date_naive() == today {
                    task.done = false;
                    task.date_checked = None;
                } else {
                    return Err("Cannot uncheck task that wasn't checked today");
                }
            }
            break;
        }
        return Ok(());
    }
}

fn print_help() {
    println!("Progress tracker v0.0.1");
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        print_help();
        return;
    }

    let mut store = Store::open().expect("Could not create store");

    match args[1].as_str() {
        "task-add" => {
            if let Some(task_label) = args.get(2) {
                let now = Utc::now().timestamp();
                let id = store.metadata.last_task_id;
                let task = Task {
                    id,
                    done: false,
                    label: task_label.clone(),
                    date_checked: None,
                    date_created: now,
                };

                store.add_task(task);
                println!("Task (tsk-{}) added to store", id);
            } else {
                println!("No task in entry");
            }
        }
        "task" => {
            let id = args.get(2).expect("Expected task id");
            let command = args.get(3).expect("Expected task command");

            let ids = id.split('-').collect::<Vec<&str>>();
            if ids.len() != 2 || ids[0].to_lowercase() != "tsk" {
                println!("Invalid task id");
                panic!()
            }

            let id = ids[1].parse::<u32>().expect("Invalid task id suffix");

            match command.as_str() {
                "--remove" => {
                    store.remove_task(id).unwrap();
                }
                "--check" => {
                    store.toggle_check_task(id, true).unwrap();
                }
                "--uncheck" => {
                    store.toggle_check_task(id, false).unwrap();
                }
                _ => {
                    println!("Invalid task command");
                    print_help();
                    panic!();
                }
            }

            store.save();
        }
        _ => {
            print_help();
        }
    }
}

// progress
// progress task-add "The one that said fuck"
// progress task TSK-2 --remove
// progress task TSK-2 --check
// progress task TSK-2 --uncheck
