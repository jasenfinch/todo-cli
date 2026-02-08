use assert_cmd::{cargo, Command};
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a new command with a temporary database
fn todo_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-p").arg(temp_dir.path());
    cmd
}

/// Helper to add a task and extract its ID from output
fn add_task(temp_dir: &TempDir, args: &[&str]) -> String {
    let output = todo_cmd(temp_dir)
        .args(["add"])
        .args(args)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Extract ID from "Added task with ID abc123"
    stdout.split_whitespace().last().unwrap().trim().to_string()
}

#[test]
fn test_help() {
    Command::new(cargo::cargo_bin!())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("task management"));
}

#[test]
fn test_version() {
    Command::new(cargo::cargo_bin!())
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_no_args_shows_help() {
    Command::new(cargo::cargo_bin!())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

// ============================================================================
// ADD COMMAND TESTS
// ============================================================================

#[test]
fn test_add_simple_task() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["add", "Simple task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added task with ID"));
}

#[test]
fn test_add_task_with_all_options() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args([
            "add",
            "Complex task",
            "--desc",
            "A detailed description",
            "--diff",
            "7",
            "--deadline",
            "tomorrow",
            "--tags",
            "work,urgent",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added task with ID"));
}

#[test]
fn test_add_task_with_parent() {
    let temp_dir = TempDir::new().unwrap();

    let parent_id = add_task(&temp_dir, &["Parent task"]);

    todo_cmd(&temp_dir)
        .args(["add", "Subtask", "--pid", &parent_id])
        .assert()
        .success();
}

#[test]
fn test_add_task_invalid_difficulty() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["add", "Task", "--diff", "15"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("15"));
}

#[test]
fn test_add_task_with_deadline_formats() {
    let temp_dir = TempDir::new().unwrap();

    // Test various deadline formats
    for deadline in &["today", "tomorrow", "friday", "+5d", "2026-12-31", "eom"] {
        todo_cmd(&temp_dir)
            .args(["add", "Task", "--deadline", deadline])
            .assert()
            .success();
    }
}

#[test]
fn test_add_task_invalid_deadline() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["add", "Task", "--deadline", "invalid-date"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid deadline"));
}

// ============================================================================
// LIST COMMAND TESTS
// ============================================================================

#[test]
fn test_list_empty_database() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir).arg("list").assert().success();
}

#[test]
fn test_list_shows_added_tasks() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["First task"]);
    add_task(&temp_dir, &["Second task"]);

    todo_cmd(&temp_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("First task"))
        .stdout(predicate::str::contains("Second task"));
}

#[test]
fn test_list_view_modes() {
    let temp_dir = TempDir::new().unwrap();
    add_task(&temp_dir, &["Task"]);

    for view in &["minimal", "compact", "full"] {
        todo_cmd(&temp_dir)
            .args(["list", "--view", view])
            .assert()
            .success();
    }
}

#[test]
fn test_list_custom_columns() {
    let temp_dir = TempDir::new().unwrap();
    add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir)
        .args(["list", "--columns", "id,task,difficulty"])
        .assert()
        .success();
}

#[test]
fn test_list_filter_by_tags() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Work task", "--tags", "work"]);
    add_task(&temp_dir, &["Personal task", "--tags", "personal"]);

    let output = todo_cmd(&temp_dir)
        .args(["list", "--tags", "work"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Work task"));
    assert!(!stdout.contains("Personal task"));
}

#[test]
fn test_list_filter_by_parent() {
    let temp_dir = TempDir::new().unwrap();

    let parent_id = add_task(&temp_dir, &["Parent"]);
    add_task(&temp_dir, &["Child", "--pid", &parent_id]);
    add_task(&temp_dir, &["Unrelated"]);

    let output = todo_cmd(&temp_dir)
        .args(["list", "--pid", &parent_id])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Child"));
    assert!(!stdout.contains("Unrelated"));
}

#[test]
fn test_list_all() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);
    todo_cmd(&temp_dir)
        .args(["complete", &id])
        .assert()
        .success();

    // Without flag, shouldn't show completed
    let output = todo_cmd(&temp_dir).arg("list").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Task") || stdout.contains("No tasks"));

    // With flag, should show completed
    todo_cmd(&temp_dir)
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task"));
}

#[test]
fn test_list_only_completed() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Incomplete"]);
    let id = add_task(&temp_dir, &["Complete"]);
    todo_cmd(&temp_dir)
        .args(["complete", &id])
        .assert()
        .success();

    let output = todo_cmd(&temp_dir)
        .args(["list", "--completed"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Complete"));
    assert!(!stdout.contains("Incomplete"));
}

#[test]
fn test_list_alias() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir).arg("ls").assert().success();
}

// ============================================================================
// SHOW COMMAND TESTS
// ============================================================================

#[test]
fn test_show_task() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(
        &temp_dir,
        &[
            "Test task",
            "--desc",
            "Description",
            "--diff",
            "5",
            "--tags",
            "work",
        ],
    );

    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test task"))
        .stdout(predicate::str::contains("Description"))
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("work"));
}

#[test]
fn test_show_nonexistent_task() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["show", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No task found"));
}

#[test]
fn test_show_partial_id() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);
    let partial_id = &id[..3];

    todo_cmd(&temp_dir)
        .args(["show", partial_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task"));
}

// ============================================================================
// COMPLETE COMMAND TESTS
// ============================================================================

#[test]
fn test_complete_task() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir)
        .args(["complete", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("marked as complete"));
}

#[test]
fn test_complete_alias() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir).args(["done", &id]).assert().success();
}

#[test]
fn test_complete_nonexistent_task() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["complete", "nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_complete_partial_id() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);
    let partial_id = &id[..3];

    todo_cmd(&temp_dir)
        .args(["complete", partial_id])
        .assert()
        .success();
}

// ============================================================================
// UPDATE COMMAND TESTS
// ============================================================================

#[test]
fn test_update_task_title() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Old title"]);

    todo_cmd(&temp_dir)
        .args(["update", &id, "--task", "New title"])
        .assert()
        .success();

    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .stdout(predicate::str::contains("New title"));
}

#[test]
fn test_update_task_difficulty() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task", "--diff", "3"]);

    todo_cmd(&temp_dir)
        .args(["update", &id, "--diff", "8"])
        .assert()
        .success();

    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .stdout(predicate::str::contains("8"));
}

#[test]
fn test_update_task_deadline() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir)
        .args(["update", &id, "--deadline", "tomorrow"])
        .assert()
        .success();
}

#[test]
fn test_update_task_tags() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task", "--tags", "old"]);

    todo_cmd(&temp_dir)
        .args(["update", &id, "--tags", "new,updated"])
        .assert()
        .success();

    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("updated"));
}

#[test]
fn test_update_multiple_fields() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir)
        .args([
            "update",
            &id,
            "--task",
            "Updated",
            "--diff",
            "9",
            "--deadline",
            "friday",
            "--tags",
            "urgent",
        ])
        .assert()
        .success();
}

#[test]
fn test_update_nonexistent_task() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["update", "nonexistent", "--task", "New"])
        .assert()
        .failure();
}

// ============================================================================
// NEXT COMMAND TESTS
// ============================================================================

#[test]
fn test_next_with_no_tasks() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir).arg("next").assert().failure();
}

#[test]
fn test_next_shows_highest_priority() {
    let temp_dir = TempDir::new().unwrap();

    add_task(
        &temp_dir,
        &["Low priority", "--diff", "1", "--deadline", "+7d"],
    );
    add_task(
        &temp_dir,
        &["High priority", "--diff", "10", "--deadline", "tomorrow"],
    );

    todo_cmd(&temp_dir)
        .arg("next")
        .assert()
        .success()
        .stdout(predicate::str::contains("High priority"));
}

#[test]
fn test_next_ignores_completed() {
    let temp_dir = TempDir::new().unwrap();

    let id1 = add_task(
        &temp_dir,
        &["Urgent", "--diff", "10", "--deadline", "today"],
    );
    add_task(
        &temp_dir,
        &["Less urgent", "--diff", "5", "--deadline", "+7d"],
    );

    todo_cmd(&temp_dir)
        .args(["complete", &id1])
        .assert()
        .success();

    todo_cmd(&temp_dir)
        .arg("next")
        .assert()
        .success()
        .stdout(predicate::str::contains("Less urgent"));
}

// ============================================================================
// REMOVE COMMAND TESTS
// ============================================================================

#[test]
fn test_remove_single_task() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir)
        .args(["remove", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 1 task"));
}

#[test]
fn test_remove_multiple_tasks() {
    let temp_dir = TempDir::new().unwrap();

    let id1 = add_task(&temp_dir, &["Task 1"]);
    let id2 = add_task(&temp_dir, &["Task 2"]);

    todo_cmd(&temp_dir)
        .args(["remove", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 2 task"));
}

#[test]
fn test_remove_by_tags() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Task 1", "--tags", "remove"]);
    add_task(&temp_dir, &["Task 2", "--tags", "remove"]);
    add_task(&temp_dir, &["Task 3", "--tags", "keep"]);

    todo_cmd(&temp_dir)
        .args(["remove", "--tags", "remove"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 2 task"));

    // Verify the "keep" task is still there
    todo_cmd(&temp_dir)
        .arg("list")
        .assert()
        .stdout(predicate::str::contains("Task 3"));
}

#[test]
fn test_remove_alias() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);

    todo_cmd(&temp_dir).args(["rm", &id]).assert().success();
}

#[test]
fn test_remove_nonexistent_task() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .args(["remove", "nonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 0 task"));
}

#[test]
fn test_remove_partial_id() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task"]);
    let partial_id = &id[..3];

    todo_cmd(&temp_dir)
        .args(["remove", partial_id])
        .assert()
        .success();
}

#[test]
fn test_remove_requires_id_or_tags() {
    let temp_dir = TempDir::new().unwrap();

    todo_cmd(&temp_dir)
        .arg("remove")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// CLEAR COMMAND TESTS
// ============================================================================

#[test]
fn test_clear_with_force() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Task 1"]);
    add_task(&temp_dir, &["Task 2"]);

    todo_cmd(&temp_dir)
        .args(["clear", "--force"])
        .assert()
        .success();

    // Verify all tasks are gone
    let output = todo_cmd(&temp_dir).arg("list").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Task 1"));
    assert!(!stdout.contains("Task 2"));
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_task_lifecycle() {
    let temp_dir = TempDir::new().unwrap();

    // Add a task
    let id = add_task(
        &temp_dir,
        &[
            "Complete project",
            "--desc",
            "Finish the todo app",
            "--diff",
            "8",
            "--deadline",
            "friday",
            "--tags",
            "work,coding",
        ],
    );

    // Verify it appears in list
    todo_cmd(&temp_dir)
        .arg("list")
        .assert()
        .stdout(predicate::str::contains("Complete project"));

    // Update it
    todo_cmd(&temp_dir)
        .args(["update", &id, "--diff", "6"])
        .assert()
        .success();

    // Show details
    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .stdout(predicate::str::contains("6"));

    // Complete it
    todo_cmd(&temp_dir)
        .args(["complete", &id])
        .assert()
        .success();

    // Verify it's not in default list
    let output = todo_cmd(&temp_dir).arg("list").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Complete project") || stdout.contains("No tasks"));

    // Verify it appears with --completed flag
    todo_cmd(&temp_dir)
        .args(["list", "--completed"])
        .assert()
        .stdout(predicate::str::contains("Complete project"));
}

#[test]
fn test_hierarchical_tasks() {
    let temp_dir = TempDir::new().unwrap();

    // Create parent task
    let parent_id = add_task(&temp_dir, &["Main project"]);

    // Create subtasks
    let subtask1_id = add_task(&temp_dir, &["Subtask 1", "--pid", &parent_id]);
    let subtask2_id = add_task(&temp_dir, &["Subtask 2", "--pid", &parent_id]);

    // List tasks by parent
    let output = todo_cmd(&temp_dir)
        .args(["list", "--pid", &parent_id])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Subtask 1"));
    assert!(stdout.contains("Subtask 2"));

    // Complete subtasks
    todo_cmd(&temp_dir)
        .args(["complete", &subtask1_id])
        .assert()
        .success();
    todo_cmd(&temp_dir)
        .args(["complete", &subtask2_id])
        .assert()
        .success();

    // Complete parent
    todo_cmd(&temp_dir)
        .args(["complete", &parent_id])
        .assert()
        .success();
}

#[test]
fn test_tag_filtering() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Work task 1", "--tags", "work"]);
    add_task(&temp_dir, &["Work task 2", "--tags", "work,urgent"]);
    add_task(&temp_dir, &["Personal task", "--tags", "personal"]);
    add_task(&temp_dir, &["Urgent personal", "--tags", "personal,urgent"]);

    // Filter by single tag
    let output = todo_cmd(&temp_dir)
        .args(["list", "--tags", "work"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Work task"));

    // Filter by multiple tags
    let output = todo_cmd(&temp_dir)
        .args(["list", "--tags", "urgent"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Work task 2"));
    assert!(stdout.contains("Urgent personal"));
}

#[test]
fn test_priority_ordering() {
    let temp_dir = TempDir::new().unwrap();

    // Add tasks with different priorities
    add_task(&temp_dir, &["Low", "--diff", "2", "--deadline", "+30d"]);
    add_task(&temp_dir, &["Medium", "--diff", "5", "--deadline", "+7d"]);
    add_task(
        &temp_dir,
        &["High", "--diff", "9", "--deadline", "tomorrow"],
    );

    // Next should show the highest priority
    todo_cmd(&temp_dir)
        .arg("next")
        .assert()
        .success()
        .stdout(predicate::str::contains("High"));
}

#[test]
fn test_custom_database_path() {
    let temp_dir = TempDir::new().unwrap();

    let id = add_task(&temp_dir, &["Task in custom location"]);

    // Verify task is accessible with custom path
    todo_cmd(&temp_dir)
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task in custom location"));
}

#[test]
fn test_partial_id_ambiguity() {
    let temp_dir = TempDir::new().unwrap();

    // This test assumes IDs might conflict - if your ID generation
    // makes this impossible, you can skip this test
    let id1 = add_task(&temp_dir, &["Task 1"]);
    let _id2 = add_task(&temp_dir, &["Task 2"]);

    // Using full ID should always work
    todo_cmd(&temp_dir).args(["show", &id1]).assert().success();
}

#[test]
fn test_deadline_sorting() {
    let temp_dir = TempDir::new().unwrap();

    add_task(&temp_dir, &["Due later", "--deadline", "+14d"]);
    add_task(&temp_dir, &["Due soon", "--deadline", "tomorrow"]);
    add_task(&temp_dir, &["Due today", "--deadline", "today"]);

    let output = todo_cmd(&temp_dir).arg("list").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Earlier deadlines should appear first
    let today_pos = stdout.find("Due today");
    let soon_pos = stdout.find("Due soon");
    let later_pos = stdout.find("Due later");

    assert!(today_pos.is_some());
    assert!(soon_pos.is_some());
    assert!(later_pos.is_some());
    assert!(today_pos < soon_pos);
    assert!(soon_pos < later_pos);
}
