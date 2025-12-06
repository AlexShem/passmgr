//! Trie data structure for efficient prefix-based autocomplete.
//!
//! This module provides a trie implementation optimized for command and
//! credential key autocompletion in the shell.

use std::collections::HashMap;

/// A node in the trie structure.
#[derive(Debug, Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end_of_word: bool,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_end_of_word: false,
        }
    }
}

/// A trie (prefix tree) for efficient string completion.
///
/// # Example
///
/// ```
/// use passmgr::trie::Trie;
///
/// let mut trie = Trie::new();
/// trie.insert("add");
/// trie.insert("admin");
/// trie.insert("get");
///
/// assert_eq!(trie.completions("ad"), vec!["add", "admin"]);
/// assert_eq!(trie.completions("g"), vec!["get"]);
/// assert!(trie.contains("add"));
/// assert!(!trie.contains("unknown"));
/// ```
#[derive(Debug, Default)]
pub struct Trie {
    root: TrieNode,
    count: usize,
}

impl Trie {
    /// Creates a new empty trie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
            count: 0,
        }
    }

    /// Inserts a word into the trie.
    ///
    /// If the word already exists, this is a no-op.
    pub fn insert(&mut self, word: &str) {
        if word.is_empty() {
            return;
        }

        let mut current = &mut self.root;
        for ch in word.chars() {
            current = current.children.entry(ch).or_insert_with(TrieNode::new);
        }

        if !current.is_end_of_word {
            current.is_end_of_word = true;
            self.count += 1;
        }
    }

    /// Removes a word from the trie.
    ///
    /// Returns `true` if the word was found and removed, `false` otherwise.
    pub fn remove(&mut self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        // First check if the word exists
        if !self.contains(word) {
            return false;
        }

        // Navigate to the end node and unmark it
        let mut current = &mut self.root;
        for ch in word.chars() {
            current = match current.children.get_mut(&ch) {
                Some(node) => node,
                None => return false,
            };
        }

        if current.is_end_of_word {
            current.is_end_of_word = false;
            self.count -= 1;

            // Note: We don't prune empty branches for simplicity.
            // This could be optimized if memory is a concern.
            true
        } else {
            false
        }
    }

    /// Checks if a word exists in the trie.
    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        let mut current = &self.root;
        for ch in word.chars() {
            match current.children.get(&ch) {
                Some(node) => current = node,
                None => return false,
            }
        }
        current.is_end_of_word
    }

    /// Returns all words that start with the given prefix.
    ///
    /// The results are sorted alphabetically.
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();

        // Navigate to the prefix node
        let mut current = &self.root;
        for ch in prefix.chars() {
            match current.children.get(&ch) {
                Some(node) => current = node,
                None => return results, // Prefix not found
            }
        }

        // Collect all words from this node
        self.collect_words(current, &mut prefix.to_string(), &mut results);

        // Sort results alphabetically
        results.sort();
        results
    }

    /// Returns all words in the trie.
    #[allow(unused)]
    pub fn all_words(&self) -> Vec<String> {
        self.completions("")
    }

    /// Returns the number of words in the trie.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the trie is empty.
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Clears all words from the trie.
    pub fn clear(&mut self) {
        self.root = TrieNode::new();
        self.count = 0;
    }

    /// Helper function to collect all words from a given node.
    fn collect_words(&self, node: &TrieNode, prefix: &mut String, results: &mut Vec<String>) {
        if node.is_end_of_word {
            results.push(prefix.clone());
        }

        for (ch, child) in &node.children {
            prefix.push(*ch);
            self.collect_words(child, prefix, results);
            prefix.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut trie = Trie::new();

        trie.insert("hello");
        trie.insert("world");
        trie.insert("help");

        assert!(trie.contains("hello"));
        assert!(trie.contains("world"));
        assert!(trie.contains("help"));
        assert!(!trie.contains("hel")); // Not a complete word
        assert!(!trie.contains("unknown"));
    }

    #[test]
    fn test_empty_string() {
        let mut trie = Trie::new();

        trie.insert("");
        assert!(!trie.contains(""));
        assert_eq!(trie.len(), 0);
    }

    #[test]
    fn test_remove() {
        let mut trie = Trie::new();

        trie.insert("hello");
        trie.insert("help");
        assert_eq!(trie.len(), 2);

        assert!(trie.remove("hello"));
        assert!(!trie.contains("hello"));
        assert!(trie.contains("help")); // Should still exist
        assert_eq!(trie.len(), 1);

        assert!(!trie.remove("hello")); // Already removed
        assert!(!trie.remove("unknown")); // Never existed
    }

    #[test]
    fn test_completions() {
        let mut trie = Trie::new();

        trie.insert("add");
        trie.insert("admin");
        trie.insert("administrator");
        trie.insert("get");
        trie.insert("help");

        let completions = trie.completions("ad");
        assert_eq!(completions, vec!["add", "admin", "administrator"]);

        let completions = trie.completions("adm");
        assert_eq!(completions, vec!["admin", "administrator"]);

        let completions = trie.completions("g");
        assert_eq!(completions, vec!["get"]);

        let completions = trie.completions("xyz");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_completions_empty_prefix() {
        let mut trie = Trie::new();

        trie.insert("cat");
        trie.insert("add");
        trie.insert("banana");

        let all = trie.completions("");
        assert_eq!(all, vec!["add", "banana", "cat"]); // Sorted
    }

    #[test]
    fn test_all_words() {
        let mut trie = Trie::new();

        trie.insert("zebra");
        trie.insert("apple");
        trie.insert("mango");

        let all = trie.all_words();
        assert_eq!(all, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut trie = Trie::new();

        trie.insert("test");
        trie.insert("test");
        trie.insert("test");

        assert_eq!(trie.len(), 1);
        assert!(trie.contains("test"));
    }

    #[test]
    fn test_special_characters() {
        let mut trie = Trie::new();

        trie.insert("user@email.com");
        trie.insert("api-key-123");
        trie.insert("my_password");

        assert!(trie.contains("user@email.com"));
        assert!(trie.contains("api-key-123"));
        assert!(trie.contains("my_password"));

        let completions = trie.completions("api");
        assert_eq!(completions, vec!["api-key-123"]);
    }

    #[test]
    fn test_unicode() {
        let mut trie = Trie::new();

        trie.insert("café");
        trie.insert("naïve");
        trie.insert("日本語");

        assert!(trie.contains("café"));
        assert!(trie.contains("naïve"));
        assert!(trie.contains("日本語"));

        let completions = trie.completions("caf");
        assert_eq!(completions, vec!["café"]);
    }

    #[test]
    fn test_clear() {
        let mut trie = Trie::new();

        trie.insert("one");
        trie.insert("two");
        trie.insert("three");
        assert_eq!(trie.len(), 3);

        trie.clear();
        assert!(trie.is_empty());
        assert_eq!(trie.len(), 0);
        assert!(!trie.contains("one"));
    }

    #[test]
    fn test_prefix_is_also_word() {
        let mut trie = Trie::new();

        trie.insert("help");
        trie.insert("helper");
        trie.insert("helping");

        assert!(trie.contains("help"));
        assert!(trie.contains("helper"));
        assert!(trie.contains("helping"));

        let completions = trie.completions("help");
        assert_eq!(completions, vec!["help", "helper", "helping"]);

        // Remove the prefix word, children should remain
        trie.remove("help");
        assert!(!trie.contains("help"));
        assert!(trie.contains("helper"));
        assert!(trie.contains("helping"));

        let completions = trie.completions("help");
        assert_eq!(completions, vec!["helper", "helping"]);
    }
}
