use std::fs;
use tempfile::TempDir;

/// This integration test reproduces the facet parsing warnings that occur during search.
/// The warnings look like: "Warning: Skipping invalid facet 'tag bookmark': Failed to parse the facet string"
///
/// This test creates notes with tags that trigger the issue, indexes them, and then searches
/// to verify that the warnings occur and that the search still functions correctly.
#[test]
fn test_facet_parsing_warnings_during_search() {
    // Create a temporary directory for this test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let notes_dir = temp_dir.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");

    // Create notes with tags that should trigger facet parsing warnings
    let note1_content = r#"---
title: "Note with Bookmark Tag"
tags: ["bookmark", "test"]
---

# Bookmark Note

This note has tags that might cause facet parsing issues.
"#;

    let note2_content = r#"---
title: "Note with Journal Tag"
tags: ["journal", "work"]
---

# Journal Note

Another note with potentially problematic tags.
"#;

    let note3_content = r#"---
title: "Note with Prompt Tag"
tags: ["prompt", "ai"]
---

# Prompt Note

A note with a prompt tag.
"#;

    // Write the test notes
    fs::write(notes_dir.join("bookmark-note.md"), note1_content).expect("Failed to write note1");
    fs::write(notes_dir.join("journal-note.md"), note2_content).expect("Failed to write note2");
    fs::write(notes_dir.join("prompt-note.md"), note3_content).expect("Failed to write note3");

    // Set up cache directory
    let cache_dir = temp_dir.path().join("ink");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    // First, index the notes
    let index_output = std::process::Command::new("./target/debug/ink")
        .args(["index"])
        .env("INK_NOTES_DIR", notes_dir.to_str().unwrap())
        .env("INK_CACHE_DIR", cache_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute index command");

    assert!(
        index_output.status.success(),
        "Index command should succeed: {}",
        String::from_utf8_lossy(&index_output.stderr)
    );

    // Now run a search and capture stderr to check for warnings
    let search_output = std::process::Command::new("./target/debug/ink")
        .args(["search", "note"])
        .env("INK_NOTES_DIR", notes_dir.to_str().unwrap())
        .env("INK_CACHE_DIR", cache_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute search command");

    let stdout = String::from_utf8_lossy(&search_output.stdout);
    let stderr = String::from_utf8_lossy(&search_output.stderr);

    // The search should succeed despite any warnings
    assert!(
        search_output.status.success(),
        "Search should succeed even with facet warnings. stderr: {stderr}",
    );

    // Should find all three notes in the results (using the title from frontmatter)
    assert!(
        stdout.contains("Note with Bookmark Tag"),
        "Should find bookmark note"
    );
    assert!(
        stdout.contains("Note with Journal Tag"),
        "Should find journal note"
    );
    assert!(
        stdout.contains("Note with Prompt Tag"),
        "Should find prompt note"
    );

    // Check if warnings about invalid facets are present in stderr
    let has_facet_warnings = stderr.contains("Skipping invalid facet");

    if has_facet_warnings {
        println!("Current behavior - Found facet parsing warnings (this indicates the bug):");
        for line in stderr.lines() {
            if line.contains("Skipping invalid facet") {
                println!("  {line}");
            }
        }
        // FAIL the test if we find warnings - this is what we want to fix
        panic!("Found facet parsing warnings that should be fixed! stderr: {stderr}");
    } else {
        println!("✓ No facet parsing warnings found - fix is working correctly");
    }

    // Ensure search works regardless
    assert!(!stdout.is_empty(), "Search should return results");
}
