use crate::prompt::ParsedQuery;
use crate::settings::SETTINGS;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::{collector::TopDocs, query::*, schema::*, Index};
//Okokok

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

    // Create a collector that uses a weighted score
    let title_weight = 2.0;
    let lookahead_weight = 1.5;

    if !parsed_query.query.is_empty() {
        let text_query = TermQuery::new(
            Term::from_field_text(
                schema.get_field("typeahead_title").unwrap(),
                &parsed_query.query.to_lowercase(),
            ),
            IndexRecordOption::Basic,
        );
        let boosted_text_query = BoostQuery::new(Box::new(text_query), lookahead_weight);
        queries.push((Occur::Should, Box::new(boosted_text_query)));

        let title_query = TermQuery::new(
            Term::from_field_text(schema.get_field("title").unwrap(), &parsed_query.query),
            IndexRecordOption::Basic,
        );
        let boosted_title_query = BoostQuery::new(Box::new(title_query), title_weight);
        queries.push((Occur::Should, Box::new(boosted_title_query)));

        let body_query = QueryParser::for_index(&index, vec![schema.get_field("body").unwrap()])
            .parse_query(&parsed_query.query)?;
        queries.push((Occur::Should, Box::new(body_query)));
    }

    for tag in parsed_query.tags {
        let facet = Facet::from(&format!("/tag/{}", tag));
        let facet_term = Term::from_facet(schema.get_field("tag").unwrap(), &facet);
        let facet_query = TermQuery::new(facet_term, IndexRecordOption::Basic);
        queries.push((Occur::Must, Box::new(facet_query)));
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
