use crate::note::Note;
use crate::search::index_updater::update_index_metadata;
use crate::settings::SETTINGS;
use crate::utils::ensure_directory_exists;
use crate::walk::{has_extension, walk_files};
use std::path::PathBuf;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{schema::*, Index, IndexWriter, TantivyError};

fn add_document(
    note: &Note,
    index_writer: &IndexWriter,
    schema: &Schema,
) -> Result<(), TantivyError> {
    let path_field: Field = schema.get_field("path").unwrap();
    let path = note.get_file_path();
    let path_str = path
        .to_str()
        .expect("Valid path required to index document");

    // Create a term to identify the document to delete
    let term = Term::from_field_text(path_field, path_str);

    index_writer.delete_term(term);
    // Delete any existing document with the same path

    let _ = index_writer.add_document(note.to_tantivy_document(schema));
    Ok(())
}

fn index_file(markdown_path: &str, schema: &Schema, index_writer: &IndexWriter) {
    let note = match Note::from_markdown_file(markdown_path) {
        Ok(note) => note,
        Err(e) => {
            println!("{}: Error parsing", e.msg);
            return;
        }
    };
    if let Err(e) = add_document(&note, index_writer, schema) {
        println!("Failed to add note: {}", e);
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
    let text_options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("en_stem")
            .set_index_option(IndexRecordOption::Basic),
    );
    let stored_text_options = text_options.clone().set_stored();
    schema_builder.add_date_field("sort_created", FAST);
    schema_builder.add_date_field("sort_modified", FAST);
    schema_builder.add_date_field("created", INDEXED | STORED);
    schema_builder.add_date_field("modified", INDEXED | STORED);

    schema_builder.add_text_field("typeahead_title", typeahead_options);
    schema_builder.add_text_field("sort_title", FAST);

    schema_builder.add_bool_field("is_hidden", INDEXED | FAST | STORED);

    schema_builder.add_text_field("title", stored_text_options);
    schema_builder.add_text_field("body", text_options);
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
    update_index_metadata().expect("Error writing index metadata file");
    Ok(())
}
