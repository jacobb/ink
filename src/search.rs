use crate::markdown::{frontmatter, get_markdown_str};
use std::path::PathBuf;
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::*, Index, IndexWriter, TantivyError,
};

use crate::utils::ensure_directory_exists;
use crate::walk::{has_extension, walk_files};

struct MarkdownDocument {
    title: String,
    body: String,
    path: String,
}

// Indexing documents

fn add_document(
    doc: &MarkdownDocument,
    index_writer: &IndexWriter,
    schema: &Schema,
) -> Result<(), TantivyError> {
    let path_field: Field = schema.get_field("path").unwrap();

    // Create a term to identify the document to delete
    let term = Term::from_field_text(path_field, &doc.path);

    index_writer.delete_term(term);
    // Delete any existing document with the same path

    let _ = index_writer.add_document(doc.to_tantivy_document(schema));
    Ok(())
}

fn index_file(markdown_path: &str, schema: &Schema, index_writer: &IndexWriter) {
    let raw_markdown = get_markdown_str(markdown_path);
    if let Some(front_matter) = frontmatter(&raw_markdown) {
        if let (Some(title), body) = (front_matter.title, front_matter.content) {
            let doc = MarkdownDocument {
                title,
                body,
                path: markdown_path.to_string(),
            };
            // Handle the case where adding a document fails.
            if let Err(e) = add_document(&doc, index_writer, schema) {
                println!("Failed to add document: {}", e);
            }
        }
    }
}

impl MarkdownDocument {
    fn to_tantivy_document(&self, schema: &Schema) -> Document {
        let mut doc = Document::new();
        doc.add_text(schema.get_field("title").unwrap(), &self.title);
        doc.add_text(schema.get_field("path").unwrap(), &self.path);
        doc.add_text(schema.get_field("body").unwrap(), &self.body);
        doc
    }
}

// Handling Index
fn open_or_create_index(index_path: &PathBuf, schema: &Schema) -> Result<Index, TantivyError> {
    ensure_directory_exists(index_path)?;
    match Index::open_in_dir(index_path) {
        Ok(index) => Ok(index), // If successful, return the existing index
        Err(_) => {
            println!("Creating index in {}", &index_path.to_str().unwrap());
            Index::create_in_dir(index_path, schema.clone())
        }
    }
}

pub fn create_index_and_add_documents(
    index_path: &PathBuf,
    notes_path: &PathBuf,
) -> tantivy::Result<()> {
    // Create a schema builder
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_text_field("path", STRING | STORED);
    // Build the schema
    let schema = schema_builder.build();

    // Create or open the index
    let index = match open_or_create_index(index_path, &schema) {
        Ok(index) => index,
        Err(e) => {
            println!("Something went wrong generating the index {}", e);
            return Err(e);
        }
    };
    // Create an index writer
    let mut index_writer = index.writer(50_000_000)?;

    walk_files(notes_path, true, has_extension, |path| {
        index_file(path, &schema, &index_writer)
    });

    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();
    println!("Indexed {} documents", searcher.num_docs());
    Ok(())
}

// Searching
fn extract_stored_fields(document: &Document, schema: &Schema) -> Option<(String, String)> {
    // Get the field references from the schema
    let title_field = schema
        .get_field("title")
        .expect("Title field does not exist.");
    let path_field = schema
        .get_field("path")
        .expect("Path field does not exist.");

    // Extract the values from the document
    let title_value = document.get_first(title_field)?.as_text()?;
    let path_value = document.get_first(path_field)?.as_text()?;

    Some((title_value.to_string(), path_value.to_string()))
}

pub fn search_index(index_path: &PathBuf, query_str: &str) -> tantivy::Result<()> {
    // Open the index
    let index = Index::open_in_dir(index_path)?;

    // Get the schema and create a query parser
    let schema = index.schema();
    let query_parser = QueryParser::for_index(
        &index,
        vec![
            schema.get_field("title").unwrap(),
            schema.get_field("body").unwrap(),
            schema.get_field("path").unwrap(),
        ],
    );

    // Parse the query
    let query = query_parser.parse_query(query_str)?;

    // Create a searcher
    let searcher = index.reader()?.searcher();

    // Search the index
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        if let Some((title, path)) = extract_stored_fields(&retrieved_doc, &schema) {
            println!("{}\t{}", title, path)
        }
    }

    Ok(())
}
