use chrono::{DateTime, Local};
use colored::Colorize;
use std::{
    fs,
    io::{Read, Write},
    path::{self},
};

use crate::utils::format_timestamp_ago;

const STORE_FILE: &str = "progress.store";

enum ParseState {
    WritingMetadata(u8),
    WritingTask(u8),
}

pub struct Task {
    pub id: u32,
    pub done: bool,
    pub label: String,
    pub date_created: i64,
    pub date_checked: Option<i64>,
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
pub struct Metadata {
    pub last_task_id: u32,
}

impl Metadata {
    fn dump(&self, buffer: &mut String) {
        assert!(buffer.is_empty());
        buffer.push_str(":metadata\n");
        buffer.push_str(format!("{}\n", self.last_task_id).as_str());
        buffer.push_str(":end\n");
    }
}

pub struct Store {
    root: String,
    pub metadata: Metadata,
    pub tasks: Vec<Task>,
}

impl Store {
    pub fn save(&self) {
        let file_path = path::Path::new(&self.root).join(STORE_FILE);
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

    pub fn open(root: &str) -> Result<Self, String> {
        let file_path = path::Path::new(root).join(STORE_FILE);

        if !file_path.exists() {
            return Ok(Store {
                root: root.to_string(),
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

        return Ok(Store {
            root: root.to_string(),
            metadata,
            tasks,
        });
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.metadata.last_task_id += 1;
        self.save();
    }

    pub fn show_task_information(&self, id: u32) {
        for task in &self.tasks {
            if task.id == id {
                println!(
                    "{} · TSK-{}",
                    if task.done {
                        "[x]".green()
                    } else {
                        "[-]".red()
                    },
                    task.id
                );
                println!("----------------------");
                println!("{}", task.label);
                println!("Created ({})", format_timestamp_ago(task.date_created));

                if let Some(date_checked) = task.date_checked {
                    println!("Finished ({})", format_timestamp_ago(date_checked))
                }
                return;
            }
        }

        let error_message = format!("Task {} does not exist", id);
        println!("{}", error_message.red());
    }

    pub fn remove_task(&mut self, id: u32) -> Result<(), &str> {
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

    pub fn relabel_task(&mut self, id: u32, label: &str) -> Result<(), &str> {
        for task in &mut self.tasks {
            if task.id != id {
                continue;
            }

            if task.done {
                println!("{}", "Cannot rename finished task".red());
                return Ok(()); // bad design here, but who cares
            }

            task.label = label.to_owned();
            return Ok(());
        }
        println!("No task with te specified id");
        return Ok(());
    }

    pub fn toggle_check_task(&mut self, id: u32, check: bool) -> Result<(), &str> {
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
                let now = Local::now().timestamp();
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

    pub fn show_info(&self) {
        let now = Local::now();
        let today = now.date_naive();

        let tasks_today: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| {
                let task_date = DateTime::from_timestamp(task.date_created, 0)
                    .map(|dt| dt.date_naive())
                    .unwrap_or(today);
                task_date == today
            })
            .collect();

        let unchecked_tasks_before_today: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| {
                let task_date = DateTime::from_timestamp(task.date_created, 0)
                    .map(|dt| dt.date_naive())
                    .unwrap_or(today);
                task_date < today && task.date_checked.is_none()
            })
            .collect();

        if tasks_today.is_empty() {
            // there's no task created today
            if unchecked_tasks_before_today.is_empty() {
                // there's o tsk created in the past that needs to be do today
                println!("{}", "No tasks for today".green().bold());
            }
        } else {
            println!("{}", "Tasks for Today:".green().bold());
            for task in &tasks_today {
                println!(
                    "{} {}",
                    if task.done {
                        format!("TSK-{} - [x]", task.id,).green()
                    } else {
                        format!("TSK-{} - [-]", task.id,).red()
                    },
                    if task.done {
                        task.label.strikethrough()
                    } else {
                        task.label.normal()
                    }
                );
            }
            println!();
        }

        if !unchecked_tasks_before_today.is_empty() {
            println!("{}", "Carry-over tasks:".yellow().bold());
            for task in &unchecked_tasks_before_today {
                println!(
                    "TSK-{} ({}) - [{}] {}",
                    task.id,
                    format_timestamp_ago(task.date_created),
                    if task.done { "x" } else { " " },
                    task.label
                );
            }
        }

        let total_tasks = self.tasks.len();
        let completed_tasks = self.tasks.iter().filter(|task| task.done).count();
        let incomplete_tasks = total_tasks - completed_tasks;
        let done_today = self
            .tasks
            .iter()
            .filter(|task| {
                if let Some(checked_time) = task.date_checked {
                    let checked_date = DateTime::from_timestamp(checked_time, 0)
                        .map(|dt| dt.date_naive())
                        .unwrap_or(today);
                    return checked_date == today;
                }
                false
            })
            .count();
        let done_before_today = completed_tasks - done_today;

        let earliest_date = self
            .tasks
            .iter()
            .filter_map(|task| {
                DateTime::from_timestamp(task.date_created, 0).map(|dt| dt.date_naive())
            })
            .min();
        let latest_date = self
            .tasks
            .iter()
            .filter_map(|task| {
                DateTime::from_timestamp(task.date_created, 0).map(|dt| dt.date_naive())
            })
            .max();

        println!("{}", "\nStatistics:".green().bold());
        println!("- Total tasks: {}", total_tasks);
        println!("- Completed tasks: {}", completed_tasks);
        println!("- Incomplete tasks: {}", incomplete_tasks);
        println!("- Tasks created today: {}", tasks_today.len());
        println!("- Tasks marked as done today: {}", done_today);
        println!("- Tasks marked as done before today: {}", done_before_today);
        println!(
            "- Unchecked tasks from before today: {}",
            unchecked_tasks_before_today.len()
        );

        if let Some(earliest) = earliest_date {
            println!("- Earliest task creation date: {}", earliest);
        }
        if let Some(latest) = latest_date {
            println!("- Latest task creation date: {}", latest);
        }

        println!("Use --help to see more.")
    }

    pub fn show_info_basic(&self) {
        let today = Local::now().date_naive();

        // pending tasks (including unchecked tasks from previous days)
        let pending_tasks_today: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| {
                let task_date = DateTime::from_timestamp(task.date_created, 0)
                    .map(|dt| dt.date_naive())
                    .unwrap_or(today);

                // task is either from today or unchecked (not marked as done)
                (task_date == today || task.date_checked.is_none()) && !task.done
            })
            .collect();

        // unchecked tasks from previous days
        let pending_tasks_previous_days: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| {
                let task_date = DateTime::from_timestamp(task.date_created, 0)
                    .map(|dt| dt.date_naive())
                    .unwrap_or(today);

                // task is from a previous day and unchecked
                task_date < today && task.date_checked.is_none() && !task.done
            })
            .collect();

        let total_pending = pending_tasks_today.len();
        let from_previous_days = pending_tasks_previous_days.len();

        if total_pending + from_previous_days == 0 {
            println!(
                "📅 {} You gotta lockin! create a task! see --help",
                format!("[{}]", Local::now().format("%H:%M")).green(),
            );
        } else {
            println!(
                "📅 {} You have {} pending task(s) for today, {} from previous days",
                format!("[{}]", Local::now().format("%H:%M")).green(),
                total_pending.to_string().yellow().bold(),
                from_previous_days.to_string().red().bold()
            )
        };
    }
}
