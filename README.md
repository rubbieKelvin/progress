# Progress CLI

A simple command-line application for managing tasks. You can add tasks, mark them as done, uncheck them, and remove them. The application also supports displaying essential task information with minimal output or in-depth details.

---

## Table of Contents

- [Progress CLI](#progress-cli)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
  - [Usage](#usage)
    - [General Commands](#general-commands)
      - [`--help`](#--help)
      - [`--minimal`](#--minimal)
      - [`task-add <label>`](#task-add-label)
      - [`task <task-id> <command> [options]`](#task-task-id-command-options)
    - [Task Commands](#task-commands)
      - [`--remove`](#--remove)
      - [`--check`](#--check)
      - [`--uncheck`](#--uncheck)
  - [Examples](#examples)
  - [License](#license)

---

## Installation

To run this project, you'll need to have Rust installed on your machine. If you haven't installed Rust yet, follow the instructions on [Rust's official website](https://www.rust-lang.org/tools/install).

1. **Clone the repository**:

```bash
   git clone https://github.com/yourusername/progress-cli.git
   cd progress-cli
```

2. **Build the project** using Cargo (Rust package manager):

   ```bash
   cargo build --release
   ```

3. **Run the application**:

   To run the project directly from Cargo:

   ```bash
   cargo run
   ```

   Or use the compiled executable:

   ```bash
   ./target/release/progress
   ```

---

## Usage

The `progress` CLI allows users to manage tasks with a few simple commands. Below is a list of available commands and how to use them.

### General Commands

#### `--help`

Displays the help message with all available commands.

```bash
progress --help
```

#### `--minimal`

Shows basic and essential information about tasks for today, including pending tasks.

```bash
progress --minimal
```

#### `task-add <label>`

Adds a new task with the specified label. You need to provide a label for the task when running this command.

```bash
progress task-add "Buy groceries"
```

#### `task <task-id> <command> [options]`

Manage an existing task using the task ID and specific subcommands. Below are the subcommands available for managing tasks.

---

### Task Commands

#### `--remove`

Removes the task with the given ID.

```bash
progress task TSK-1 --remove
```

#### `--check`

Marks the task with the given ID as done.

```bash
progress task TSK-1 --check
```

#### `--uncheck`

Marks the task with the given ID as undone.

```bash
progress task TSK-1 --uncheck
```

---

## Examples

Here are some example commands to demonstrate how to use the `progress` CLI:

```bash
# Show the help message
progress --help

# Show minimal task information
progress --minimal

# Add a new task
progress task-add "Buy groceries"

# Mark a task as done
progress task TSK-1 --check

# Mark a task as undone
progress task TSK-1 --uncheck

# Remove a task
progress task TSK-1 --remove
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
