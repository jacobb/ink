use crate::markdown::{frontmatter, get_markdown_str};
use crate::settings::SETTINGS;
use crate::utils::ensure_directory_exists;
use crate::walk::{has_extension, walk_files};
use std::path::PathBuf;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{schema::*, Index, IndexWriter, TantivyError};

struct MarkdownDocument {
    title: String,
    body: String,
    path: String,
    tags: Vec<String>,
}

impl MarkdownDocument {
    fn to_tantivy_document(&self, schema: &Schema) -> Document {
        let mut doc = Document::new();
        doc.add_text(schema.get_field("title").unwrap(), &self.title);
        doc.add_text(
            schema.get_field("typeahead_title").unwrap(),
            &self.title.to_lowercase(),
        );
        doc.add_text(schema.get_field("path").unwrap(), &self.path);
        doc.add_text(schema.get_field("body").unwrap(), &self.body);
        for tag in &self.tags {
            let facet = Facet::from(&format!("/tag/{}", tag));
            doc.add_facet(schema.get_field("tag").unwrap(), facet);
        }
        doc
    }
}

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
        if let (Some(title), tags, body) = (
            front_matter.title,
            front_matter.tags.unwrap_or(Vec::new()),
            front_matter.content,
        ) {
            let doc = MarkdownDocument {
                title,
                body,
                tags,
                path: markdown_path.to_string(),
            };
            // Handle the case where adding a document fails.
            if let Err(e) = add_document(&doc, index_writer, schema) {
                println!("Failed to add document: {}", e);
            }
        }
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

fn get_index(schema: &Schema) -> Result<Index, TantivyError> {
    // Create or open the index
    let index = open_or_create_index(&SETTINGS.get_cache_path(), schema)?;
    let ngram_tokenizer = NgramTokenizer::new(2, 7, false).unwrap();
    index.tokenizers().register("ngram", ngram_tokenizer);
    Ok(index)
}

fn get_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    let typeahead_options = TextOptions::default().set_stored().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("ngram")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );
    schema_builder.add_text_field("typeahead_title", typeahead_options);
    schema_builder.add_text_field("title", STRING | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_text_field("path", STRING | STORED);
    schema_builder.add_facet_field("tag", INDEXED | STORED);
    // Build the schema
    schema_builder.build()
}

pub fn create_index_and_add_documents() -> tantivy::Result<()> {
    let schema = get_schema();
    let index = get_index(&schema)?;

    // Create an index writer
    let mut index_writer = index.writer(50_000_000)?;

    walk_files(&SETTINGS.get_notes_path(), true, has_extension, |path| {
        index_file(path, &schema, &index_writer)
    });

    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();
    println!("Indexed {} documents", searcher.num_docs());
    Ok(())
}
