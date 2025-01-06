use chrono::{DateTime, Local};

pub fn format_timestamp_ago(timestamp: i64) -> String {
    let now = Local::now().to_utc();
    let time = DateTime::from_timestamp(timestamp, 0).unwrap();

    let duration = now - time;

    if duration.num_seconds() < 60 {
        return format!("{} seconds ago", duration.num_seconds());
    } else if duration.num_minutes() < 60 {
        return format!("{} minutes ago", duration.num_minutes());
    } else if duration.num_hours() < 24 {
        return format!("{} hours ago", duration.num_hours());
    } else if duration.num_days() < 7 {
        return format!("{} days ago", duration.num_days());
    } else {
        return format!("{} weeks ago", duration.num_weeks());
    }
}

pub fn print_help(name: &String) {
    println!("{} <command> [options]\n", name);
    println!("Commands:");
    println!("  --help            Show this help message.");
    println!("  --minimal         Show minimal task information.");
    println!("  --add <label>  Add a new task with the specified label.");
    println!("  --task <task-id> <command> [options]  Manage an existing task.\n");
    println!("Task Commands:");
    println!("  --remove          Remove the task with the given ID.");
    println!("  --check           Mark the task with the given ID as done.");
    println!("  --uncheck         Mark the task with the given ID as undone.\n");
    println!("Examples:");
    println!(
        "  {} --help                          Show this help message.",
        name
    );
    println!(
        "  {} --minimal                       Show minimal task information.",
        name
    );
    println!("  {} --add \"Buy groceries\"       Add a new task.", name);
    println!(
        "  {} --task TSK-1 --check              Mark task TSK-1 as done.",
        name
    );
    println!(
        "  {} --task TSK-2 --uncheck            Mark task TSK-2 as undone.",
        name
    );
    println!(
        "  {} --task TSK-3 --remove             Remove task TSK-3.",
        name
    );
    println!("\n\nwith ❤️ from rubbie kelvin (dev.rubbie@gmail.com)\n");
}
