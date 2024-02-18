use crate::settings::SETTINGS;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{collector::TopDocs, query::*, schema::*, Index};

struct ParsedQuery {
    #[allow(dead_code)]
    original_query: String,
    query: String,
    tags: Vec<String>,
}

impl ParsedQuery {
    fn from_query(query: &str) -> Self {
        let mut tags = Vec::new();
        let mut query_parts = Vec::new();

        for part in query.split_whitespace() {
            if part.starts_with('#') {
                tags.push(part.trim_start_matches('#').to_string());
            } else {
                query_parts.push(part.to_string());
            }
        }

        ParsedQuery {
            original_query: query.to_string(),
            query: query_parts.join(" "),
            tags,
        }
    }
}

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

pub fn search_index(query: &str) -> tantivy::Result<()> {
    // Open the index
    let index_path = &SETTINGS.get_cache_path();
    let index = Index::open_in_dir(index_path)?;
    let ngram_tokenizer = NgramTokenizer::new(2, 7, false).unwrap();
    index.tokenizers().register("ngram", ngram_tokenizer);

    let parsed_query = ParsedQuery::from_query(query);

    let schema = index.schema();
    let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    if !parsed_query.query.is_empty() {
        let text_query = TermQuery::new(
            Term::from_field_text(
                schema.get_field("typeahead_title").unwrap(),
                &parsed_query.query.to_lowercase(),
            ),
            IndexRecordOption::Basic,
        );
        queries.push((Occur::Must, Box::new(text_query)));
    }

    for tag in parsed_query.tags {
        let facet = Facet::from(&format!("/tag/{}", tag));
        let facet_term = Term::from_facet(schema.get_field("tag").unwrap(), &facet);
        let facet_query = TermQuery::new(facet_term, IndexRecordOption::Basic);
        let facet_tuple = (Occur::Must, Box::new(facet_query) as Box<dyn Query>);
        queries.push(facet_tuple);
    }

    let combined_query = BooleanQuery::new(queries);

    // Create a searcher
    let searcher = index.reader()?.searcher();

    // Search the index
    let top_docs = searcher.search(&combined_query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        if let Some((title, path)) = extract_stored_fields(&retrieved_doc, &schema) {
            println!("{}\t{}", title, path)
        }
    }

    Ok(())
}
