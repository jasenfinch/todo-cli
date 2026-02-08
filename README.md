# todo

A powerful, intuitive command-line task management tool built with Rust.

## Features

- **Smart Deadline Parsing** - Use natural language like `tomorrow`, `friday`, `+5d`, or exact dates
- **Task Hierarchy** - Create subtasks with parent-child relationships
- **Tag-Based Organization** - Categorize tasks with multiple tags
- **Difficulty Ratings** - Rate tasks from 0 (trivial) to 10 (near-impossible)
- **Priority Ranking** - Automatically prioritizes tasks by deadline and difficulty
- **Flexible Filtering** - Filter by tags, parent tasks, or completion status
- **Interactive Mode** - Add tasks with a guided prompt interface
- **Multiple View Modes** - Minimal, compact, or full task displays
- **Custom Columns** - Choose exactly which information to display
- **SQLite Backend** - Reliable, file-based storage

## Installation

### From Source

```bash
git clone https://github.com/jasenfinch/todo-cli.git
cd todo
cargo install --path .
```

## Quick Start

```bash
# Add a task
todo add "Write documentation" --diff 3 --deadline friday --tags work

# List all tasks
todo list

# Mark a task as complete
todo complete abc123

# Show the next highest-priority task
todo next

# Remove a task
todo remove abc123
```

## Usage

### Adding Tasks

**Command-line mode:**
```bash
todo add "Task name" [OPTIONS]
```

**Options:**
- `-d, --desc <TEXT>` - Task description
- `--diff <0-10>` - Difficulty rating (0=trivial, 10=near-impossible)
- `-l, --deadline <DATE>` - Due date (see Deadline Formats below)
- `-t, --tags <TAGS>` - Comma-separated tags
- `-p, --pid <PARENT_ID>` - Parent task ID for subtasks (must be 7 characters)

**Examples:**
```bash
# Simple task
todo add "Buy groceries"

# Task with all options
todo add "Deploy to production" \
  --desc "Deploy version 2.0 to production servers" \
  --diff 8 \
  --deadline tomorrow \
  --tags work,urgent,devops

# Create a subtask
todo add "Run migration scripts" --pid abc1234
```

**Interactive mode:**
```bash
todo add
# Follow the prompts to enter task details
```

### Deadline Formats

Use flexible date formats that make sense to you:

**Keywords:**
- `today`, `tomorrow`, `tmr`
- `monday`, `tuesday`, `wed`, etc. (next occurrence)

**Relative:**
- `+5d`, `5d` - 5 days from now
- `+2w`, `2weeks` - 2 weeks from now
- `+1m`, `1month` - 1 month from now

**Special:**
- `eow`, `endofweek` - End of week (Sunday)
- `eom`, `endofmonth` - End of month
- `eoy`, `endofyear` - End of year

**Exact dates:**
- `2026-02-10` - ISO format (YYYY-MM-DD)
- `10/02/2026` - UK format (DD/MM/YYYY)
- `10-02-2026` - US format (MM-DD-YYYY)

### Listing Tasks

```bash
# Default compact view
todo list

# Minimal view (ID, title, completion only)
todo list --view minimal

# Full view (all fields)
todo list --view full

# Custom columns
todo list --columns id,title,difficulty,deadline

# Filter by tags
todo list --tags work,urgent

# Show subtasks of a parent
todo list --pid abc1234

# Show all tasks including completed
todo list --all

# Show only completed tasks
todo list --completed
```

**Aliases:**
```bash
todo ls  # Same as todo list
```

**View Modes:**
- `minimal` - ID, title, completion status
- `compact` - ID, title, difficulty, deadline, tags, completion (default)
- `full` - All available fields

**Available Columns:**
`id`, `title`, `description`, `difficulty`, `deadline`, `tags`, `parent`, `complete`

### Managing Tags

```bash
# List all tags in use
todo tags
```

This shows all tags currently assigned to tasks, useful for discovering what tags you've used and for filtering.

### Updating Tasks

```bash
todo update <ID> [OPTIONS]

# Update title
todo update abc1234 --task "New task name"

# Update deadline
todo update abc1234 --deadline +7d

# Update multiple fields
todo update abc1234 \
  --diff 9 \
  --deadline eom \
  --tags critical,backend

# Remove a field by updating it to empty
todo update abc1234 --desc ""
```

Only specified fields are changed; others remain unchanged.

### Completing Tasks

```bash
# Mark as complete
todo complete abc1234

# Alias
todo done abc1234
```

### Viewing Task Details

```bash
todo show abc1234
```

Displays full task information including description, tags, parent task, and creation date.

### Finding Priority Tasks

```bash
# Show the highest-priority task
todo next
```

Priority is calculated based on:
1. Deadline urgency (sooner deadlines rank higher)
2. Task difficulty (harder tasks rank higher)

Tasks without deadlines are ranked lower than tasks with deadlines.

### Removing Tasks

```bash
# Remove by ID (supports partial IDs)
todo remove abc1234

# Remove multiple tasks
todo remove abc1234 def5678 ghi9012

# Remove all tasks with a tag
todo remove --tags deprecated

# Alias
todo rm abc1234
```

**Partial ID matching:** You only need to type enough characters to uniquely identify a task (e.g., `abc` instead of `abc1234`).

### Clearing All Tasks

```bash
# With confirmation prompt
todo clear

# Skip confirmation (use with caution!)
todo clear --force
```

## Database Location

By default, the task database is stored at `~/.local/share/todo/tasks.db`.

Specify a custom location:
```bash
todo -p /path/to/database list
```

## Examples

### Plan a project

```bash
# Create parent task
todo add "Website redesign" --diff 8 --deadline eom --tags project

# Add subtasks (note: use the full 7-character ID from the parent)
todo add "Design mockups" --pid abc1234 --diff 5 --deadline +3d --tags design
todo add "Implement frontend" --pid abc1234 --diff 7 --deadline +2w --tags dev
todo add "User testing" --pid abc1234 --diff 4 --deadline +3w --tags qa

# View project and subtasks
todo list --pid abc1234
```

### Daily workflow

```bash
# See what to work on next
todo next

# Add a quick task
todo add "Fix login bug" --diff 6 --deadline today --tags urgent,backend

# Check all urgent tasks
todo list --tags urgent

# Complete a task
todo done abc

# See what's left for today
todo list --columns id,title,deadline
```

### Weekly review

```bash
# See all tasks including completed
todo list --all

# Review completed tasks
todo list --completed

# Remove old completed tasks
todo remove --tags old-project

# See what tags you're using
todo tags
```

### Filter and organize

```bash
# See all work tasks
todo list --tags work

# See all tasks due this week
todo list --columns id,title,deadline,difficulty

# Find tasks by multiple tags
todo list --tags urgent,backend

# Custom view for planning
todo list --columns title,difficulty,deadline,tags
```

## Tips & Best Practices

- **Chain tags**: Use tags like `work,urgent` to quickly categorize tasks
- **Natural deadlines**: Use `friday` instead of calculating the exact date
- **Interactive mode**: When adding complex tasks, run `todo add` without arguments for a guided experience
- **Custom views**: Create your own column combinations for different workflows
- **Consistent tagging**: Use `todo tags` to see what tags you've used and stay consistent
- **Hierarchical tasks**: Break down large projects into parent tasks with subtasks
- **Priority system**: Let `todo next` guide your work based on deadline urgency and difficulty
- **Regular cleanup**: Use `todo list --completed` to review and `todo remove --tags <tag>` to bulk-remove old tasks

## Command Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `add` | - | Add a new task |
| `list` | `ls` | List tasks with filtering and view options |
| `show` | - | Show detailed information about a task |
| `update` | - | Update task fields |
| `complete` | `done` | Mark a task as complete |
| `next` | - | Show the highest-priority task |
| `tags` | - | List all tags in use |
| `remove` | `rm` | Remove tasks by ID or tag |
| `clear` | - | Remove all tasks (with confirmation) |

## Global Options

- `-p, --path <PATH>` - Specify a custom database location
- `-h, --help` - Show help information
- `-V, --version` - Show version information

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [chrono](https://github.com/chronotope/chrono) - Date and time handling
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive prompts
- [tabled](https://github.com/zhiburt/tabled) - Table formatting
- [colored](https://github.com/mackwic/colored) - Terminal colors
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/jasenfinch/todo-cli/issues) on GitHub.
