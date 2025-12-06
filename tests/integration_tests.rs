//! Integration tests for passmgr.
//!
//! These tests verify the complete workflow of the password manager.

use passmgr::credentials::Credentials;
use passmgr::manager::Manager;
use passmgr::shell::command::{CommandRegistry, CommandResult, ShellContext};
use passmgr::shell::commands::register_all;
use passmgr::trie::Trie;
use tempfile::TempDir;

/// Creates a test environment with a temporary directory.
fn setup_test_env() -> (Manager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_passwords.db");

    let mut manager = Manager::new();
    manager.set_db_path(db_path);

    (manager, temp_dir)
}

/// Creates a command registry for testing.
fn create_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    register_all(&mut registry);
    registry
}

// ============================================================================
// Manager Tests
// ============================================================================

#[test]
fn test_manager_new_user_setup() {
    let (mut manager, _temp_dir) = setup_test_env();

    assert!(manager.is_new_user(), "Should be a new user initially");

    let result = manager.setup_new_user("test_master_password".to_string());
    assert!(result.is_ok(), "Setup should succeed");

    assert!(!manager.is_new_user(), "Should no longer be a new user");
}

#[test]
fn test_manager_password_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_passwords.db");

    let mut manager = Manager::new();
    manager.set_db_path(db_path.clone());
    manager
        .setup_new_user("correct_password".to_string())
        .expect("Setup failed");

    // Create a new manager instance pointing to the same database
    let mut manager2 = Manager::new();
    manager2.set_db_path(db_path);

    // Validate with correct password
    let valid = manager2
        .validate_master_password("correct_password".to_string())
        .expect("Validation failed");
    assert!(valid, "Correct password should validate");

    // Validate with wrong password (need fresh manager)
    let mut manager3 = Manager::new();
    manager3.set_db_path(temp_dir.path().join("test_passwords.db"));
    let valid = manager3
        .validate_master_password("wrong_password".to_string())
        .expect("Validation should not error");
    assert!(!valid, "Wrong password should not validate");
}

#[test]
fn test_manager_credential_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_passwords.db");

    // Create and populate first manager
    {
        let mut manager = Manager::new();
        manager.set_db_path(db_path.clone());
        manager
            .setup_new_user("test_password".to_string())
            .expect("Setup failed");

        // Add credentials using direct manipulation (since run() requires interactive input)
        manager
            .credentials_mut()
            .add("github".to_string(), "gh_secret".to_string())
            .expect("Add failed");
        manager
            .credentials_mut()
            .add("email".to_string(), "email_secret".to_string())
            .expect("Add failed");

        manager.save_credentials().expect("Save failed");
    }

    // Create second manager and verify persistence
    {
        let mut manager2 = Manager::new();
        manager2.set_db_path(db_path);

        let valid = manager2
            .validate_master_password("test_password".to_string())
            .expect("Validation failed");
        assert!(valid, "Password should be valid");

        assert_eq!(
            manager2.credentials().get("github"),
            Some(&"gh_secret".to_string())
        );
        assert_eq!(
            manager2.credentials().get("email"),
            Some(&"email_secret".to_string())
        );
    }
}

#[test]
fn test_manager_wrong_password() {
    let (mut manager, temp_dir) = setup_test_env();
    let db_path = temp_dir.path().join("test_passwords.db");

    manager
        .setup_new_user("correct_password".to_string())
        .expect("Setup failed");

    // Create a new manager and try wrong password
    let mut manager2 = Manager::new();
    manager2.set_db_path(db_path);

    let valid = manager2
        .validate_master_password("wrong_password".to_string())
        .expect("Validation should not error");
    assert!(!valid, "Wrong password should not validate");
}

// ============================================================================
// Trie Tests
// ============================================================================

#[test]
fn test_trie_basic_operations() {
    let mut trie = Trie::new();

    trie.insert("hello");
    trie.insert("help");
    trie.insert("world");

    assert!(trie.contains("hello"));
    assert!(trie.contains("help"));
    assert!(trie.contains("world"));
    assert!(!trie.contains("hel")); // Prefix, not a word
    assert!(!trie.contains("unknown"));
}

#[test]
fn test_trie_completions() {
    let mut trie = Trie::new();

    trie.insert("add");
    trie.insert("admin");
    trie.insert("administrator");
    trie.insert("get");

    let completions = trie.completions("ad");
    assert_eq!(completions.len(), 3);
    assert!(completions.contains(&"add".to_string()));
    assert!(completions.contains(&"admin".to_string()));
    assert!(completions.contains(&"administrator".to_string()));

    let completions = trie.completions("g");
    assert_eq!(completions.len(), 1);
    assert!(completions.contains(&"get".to_string()));

    let completions = trie.completions("xyz");
    assert!(completions.is_empty());
}

#[test]
fn test_trie_remove() {
    let mut trie = Trie::new();

    trie.insert("test");
    trie.insert("testing");
    assert_eq!(trie.len(), 2);

    assert!(trie.remove("test"));
    assert!(!trie.contains("test"));
    assert!(trie.contains("testing")); // Child should still exist
    assert_eq!(trie.len(), 1);

    assert!(!trie.remove("nonexistent"));
}

#[test]
fn test_trie_special_characters() {
    let mut trie = Trie::new();

    trie.insert("api-key-123");
    trie.insert("user@email.com");
    trie.insert("my_password");

    assert!(trie.contains("api-key-123"));
    assert!(trie.contains("user@email.com"));
    assert!(trie.contains("my_password"));

    let completions = trie.completions("api");
    assert_eq!(completions, vec!["api-key-123"]);
}

// ============================================================================
// Command Tests
// ============================================================================

#[test]
fn test_add_command() {
    let mut credentials = Credentials::new();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let add_cmd = registry.get("add").expect("Add command should exist");
    let result = add_cmd.execute(&["testkey", "testsecret"], &mut ctx);

    assert!(matches!(result, CommandResult::Success(_)));
    assert!(ctx.modified);
    drop(ctx);
    assert_eq!(credentials.get("testkey"), Some(&"testsecret".to_string()));
}

#[test]
fn test_add_command_duplicate() {
    let mut credentials = Credentials::new();
    credentials
        .add("existing".to_string(), "value".to_string())
        .unwrap();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let add_cmd = registry.get("add").expect("Add command should exist");
    let result = add_cmd.execute(&["existing", "new_value"], &mut ctx);

    assert!(matches!(result, CommandResult::Error(_)));
    assert!(!ctx.modified);
}

#[test]
fn test_get_command() {
    let mut credentials = Credentials::new();
    credentials
        .add("mykey".to_string(), "mysecret".to_string())
        .unwrap();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let get_cmd = registry.get("get").expect("Get command should exist");
    let result = get_cmd.execute(&["mykey"], &mut ctx);

    match result {
        CommandResult::Success(Some(secret)) => assert_eq!(secret, "mysecret"),
        _ => panic!("Expected success with secret"),
    }
}

#[test]
fn test_get_command_not_found() {
    let mut credentials = Credentials::new();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let get_cmd = registry.get("get").expect("Get command should exist");
    let result = get_cmd.execute(&["nonexistent"], &mut ctx);

    assert!(matches!(result, CommandResult::Error(_)));
}

#[test]
fn test_remove_command() {
    let mut credentials = Credentials::new();
    credentials
        .add("toremove".to_string(), "value".to_string())
        .unwrap();
    let mut trie = Trie::new();
    trie.insert("toremove");
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let rm_cmd = registry.get("remove").expect("Remove command should exist");
    let result = rm_cmd.execute(&["toremove"], &mut ctx);

    assert!(matches!(result, CommandResult::Success(_)));
    assert!(ctx.modified);
    drop(ctx);
    assert!(credentials.get("toremove").is_none());
}

#[test]
fn test_list_command() {
    let mut credentials = Credentials::new();
    credentials.add("key1".to_string(), "val1".to_string()).unwrap();
    credentials.add("key2".to_string(), "val2".to_string()).unwrap();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let list_cmd = registry.get("list").expect("List command should exist");
    let result = list_cmd.execute(&[], &mut ctx);

    match result {
        CommandResult::Success(Some(output)) => {
            assert!(output.contains("key1"));
            assert!(output.contains("key2"));
        }
        _ => panic!("Expected success with list"),
    }
}

#[test]
fn test_help_command() {
    let mut credentials = Credentials::new();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let help_cmd = registry.get("help").expect("Help command should exist");
    let result = help_cmd.execute(&[], &mut ctx);

    match result {
        CommandResult::Success(Some(output)) => {
            assert!(output.contains("add"));
            assert!(output.contains("get"));
            assert!(output.contains("list"));
            assert!(output.contains("remove"));
            assert!(output.contains("quit"));
        }
        _ => panic!("Expected success with help text"),
    }
}

#[test]
fn test_quit_command() {
    let mut credentials = Credentials::new();
    let mut trie = Trie::new();
    let registry = create_registry();

    let mut ctx = ShellContext::new(&mut credentials, &mut trie).with_registry(&registry);

    let quit_cmd = registry.get("quit").expect("Quit command should exist");
    let result = quit_cmd.execute(&[], &mut ctx);

    assert!(matches!(result, CommandResult::Exit));
}

#[test]
fn test_command_aliases() {
    let registry = create_registry();

    // Test that aliases work
    assert!(registry.get("rm").is_some()); // alias for remove
    assert!(registry.get("ls").is_some()); // alias for list
    assert!(registry.get("q").is_some()); // alias for quit
    assert!(registry.get("exit").is_some()); // alias for quit
    assert!(registry.get("h").is_some()); // alias for help
    assert!(registry.get("g").is_some()); // alias for get
    assert!(registry.get("a").is_some()); // alias for add
}

// ============================================================================
// Credentials Tests
// ============================================================================

#[test]
fn test_credentials_basic_operations() {
    let mut creds = Credentials::new();

    assert!(creds.is_empty());

    creds.add("key1".to_string(), "val1".to_string()).unwrap();
    creds.add("key2".to_string(), "val2".to_string()).unwrap();

    assert!(!creds.is_empty());
    assert_eq!(creds.get("key1"), Some(&"val1".to_string()));
    assert_eq!(creds.get("key2"), Some(&"val2".to_string()));
    assert_eq!(creds.get("nonexistent"), None);

    let list = creds.list();
    assert_eq!(list.len(), 2);

    assert!(creds.remove("key1"));
    assert!(creds.get("key1").is_none());
    assert!(!creds.remove("key1")); // Already removed
}

#[test]
fn test_credentials_duplicate_prevention() {
    let mut creds = Credentials::new();

    creds.add("key".to_string(), "val1".to_string()).unwrap();
    let result = creds.add("key".to_string(), "val2".to_string());

    assert!(result.is_err());
    assert_eq!(creds.get("key"), Some(&"val1".to_string())); // Original value preserved
}

// ============================================================================
// Command Registry Tests
// ============================================================================

#[test]
fn test_registry_completions() {
    let registry = create_registry();

    let completions = registry.completions("he");
    assert!(completions.contains(&"help".to_string()));

    let completions = registry.completions("r");
    assert!(completions.contains(&"remove".to_string()));
    assert!(completions.contains(&"rm".to_string())); // alias

    let completions = registry.completions("");
    assert!(completions.len() >= 6); // At least the main commands
}
