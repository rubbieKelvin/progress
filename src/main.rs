use std::path::Path;

use chrono::Local;
use colored::Colorize;
use ds::{Store, Task};
use utils::print_help;

mod ds;
mod utils;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let binary_name = args[0].clone();
    let binary_file_path = std::env::current_exe().expect("Could not get binary's directory");
    let mut bin_dir_ansestor = binary_file_path.ancestors();

    bin_dir_ansestor.next(); // first iteration of ansestor
    let binary_directory = bin_dir_ansestor.next().unwrap_or(Path::new("."));

    let mut store =
        Store::open(binary_directory.to_str().unwrap()).expect("Could not create store");

    if args.len() == 1 {
        store.show_info();
        return;
    }

    match args[1].as_str() {
        "--help" => {
            print_help(&binary_name);
        }
        "--minimal" => {
            store.show_info_basic();
        }
        "--add" => {
            if let Some(task_label) = args.get(2) {
                let now = Local::now().timestamp();
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
        "--task" => {
            let id = args.get(2).expect("Expected task id");
            let command = args.get(3);

            let ids = id.split('-').collect::<Vec<&str>>();

            if ids.len() != 2 || ids[0].to_lowercase() != "tsk" {
                println!("Invalid task id");
                panic!()
            }

            let id = ids[1].parse::<u32>().expect("Invalid task id suffix");

            if command.is_none() {
                store.show_task_information(id);
                return;
            }

            let command = command.unwrap();

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
                "--rename" => {
                    if let Some(label) = args.get(4) {
                        if label.trim().len() == 0 {
                            println!("{}", "Need to include label".red());
                            return;
                        }
                        store.relabel_task(id, label).unwrap();
                    } else {
                        println!("{}", "Need to include label".red());
                    }
                }
                _ => {
                    println!("{}", "Invalid task command".red());
                    print_help(&binary_name);
                    panic!();
                }
            }

            store.save();
        }
        _ => {
            print_help(&binary_name);
        }
    }
}

// progress
// progress --help
// progress --add "The one that said fuck"
// progress --task TSK-2 --remove
// progress --task TSK-2 --check
// progress --task TSK-2 --uncheck
