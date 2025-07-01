use std::fs;
use tempfile::TempDir;

/// This integration test verifies that search handles corrupted files gracefully
/// instead of crashing when encountering tags with null bytes.
///
/// Expected behavior: Search should succeed and either:
/// 1. Skip the corrupted file with a warning, or  
/// 2. Sanitize the corrupted data and include the file
///
/// Currently this test FAILS because the search crashes with FacetParseError.
/// The crash occurs in src/note.rs:296 where Facet::from unwraps a facet containing a null byte.
#[test]
fn test_search_handles_null_byte_in_tag_gracefully() {
    // Create a temporary directory for this test
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let notes_dir = temp_dir.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");

    // Create a valid note for comparison
    let valid_note_content = r#"---
title: "Valid Note"
tags: ["valid", "test"]
---

# Valid Note

This is a valid note that should be found.
"#;
    let valid_note_path = notes_dir.join("valid-note.md");
    fs::write(valid_note_path, valid_note_content).expect("Failed to write valid note");

    // Create a markdown file with a tag containing a null byte
    // This simulates corrupted data that could come from external sources
    let corrupted_note_content = r#"---
title: "Corrupted Note"
tags: ["tag\0prompt"]
---

# Corrupted Note

This note has a tag with a null byte that currently causes a crash.
"#;

    let corrupted_note_path = notes_dir.join("corrupted-note.md");
    fs::write(corrupted_note_path, corrupted_note_content).expect("Failed to write corrupted note");

    // Set up environment to use our temporary directory
    std::env::set_var("INK_NOTES_DIR", notes_dir.to_str().unwrap());

    // Search should succeed gracefully, not crash
    let output = std::process::Command::new("./target/debug/ink")
        .args(&["search", "note"])
        .output()
        .expect("Failed to execute command");

    // The search command should succeed (not crash)
    assert!(
        output.status.success(),
        "Search should handle corrupted files gracefully, but crashed with: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // At minimum, the valid note should be found
    assert!(
        stdout.contains("Valid Note"),
        "Search should find at least the valid note, got: {}",
        stdout
    );

    // The corrupted file should either be:
    // 1. Skipped entirely (preferred), or
    // 2. Included with sanitized tags
    // We don't assert on the corrupted note specifically since either behavior is acceptable

    // Optionally check stderr for warnings about corrupted files
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("warning") || stderr.contains("corrupt") {
        println!(
            "Good: Search emitted warning about corrupted file: {}",
            stderr
        );
    }
}

#[test]
fn test_null_byte_in_tag_causes_search_crash_documentation() {
    // This test documents the issue without actually causing a crash
    // The problem occurs when parsing facets that contain null bytes

    // A tag with a null byte - this is the problematic data structure
    let problematic_tag = "tag\0prompt";

    // This demonstrates the data that causes the issue
    assert!(
        problematic_tag.contains('\0'),
        "Tag contains null byte that causes facet parsing to fail"
    );

    // The issue manifests when:
    // 1. This tag gets converted to a facet: Facet::from(&format!("/tag/{tag}"))
    // 2. The facet is stored in a Tantivy document
    // 3. During search, the document is converted back to a Note
    // 4. get_field_facets() tries to parse the facet with Facet::from()
    // 5. Tantivy's facet parser rejects the null byte and panics

    println!(
        "This test documents the null byte issue in tag: {:?}",
        problematic_tag
    );
}
