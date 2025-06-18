use crate::cli::SortChoice;
use crate::note::Note;
use crate::prompt::ParsedQuery;
use crate::search::index_updater::{index_needs_update, spawn_index_update};
use crate::settings::SETTINGS;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::DateTime as tantivy_DateTime;
use tantivy::{
    collector::TopDocs,
    index::Order,
    query::{BooleanQuery, BoostQuery, Occur, Query, QueryParser, TermQuery},
    schema::{Facet, IndexRecordOption, TantivyDocument, Term},
    DocAddress, Index, Searcher,
};

pub fn search_index(
    query: &str,
    is_json: bool,
    sort: Option<SortChoice>,
    limit: usize,
    include_ignored: bool,
) -> tantivy::Result<()> {
    if index_needs_update()? {
        spawn_index_update();
    }
    // Open the index
    let index_path = &SETTINGS.get_cache_path();
    let index = Index::open_in_dir(index_path)?;
    let ngram_tokenizer = NgramTokenizer::new(2, 7, false).unwrap();
    index.tokenizers().register("ngram", ngram_tokenizer);

    let parsed_query = ParsedQuery::from_query(query);

    let schema = index.schema();
    let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

    let title_weight = 2.0;
    let lookahead_weight = 1.5;

    if !parsed_query.query.is_empty() {
        let query_str = &parsed_query.query.to_lowercase();

        // typeahead
        let typeahead_query = TermQuery::new(
            Term::from_field_text(schema.get_field("typeahead_title").unwrap(), query_str),
            IndexRecordOption::Basic,
        );
        let boosted_typeahead_query = BoostQuery::new(Box::new(typeahead_query), lookahead_weight);
        queries.push((Occur::Should, Box::new(boosted_typeahead_query)));

        // title
        let title_query = TermQuery::new(
            Term::from_field_text(schema.get_field("title").unwrap(), query_str),
            IndexRecordOption::Basic,
        );
        let boosted_title_query = BoostQuery::new(Box::new(title_query), title_weight);
        queries.push((Occur::Should, Box::new(boosted_title_query)));

        // body
        let body_query = QueryParser::for_index(&index, vec![schema.get_field("body").unwrap()])
            .parse_query(query_str)?;
        queries.push((Occur::Should, Box::new(body_query)));
    }

    if !include_ignored {
        let is_hidden_term = Term::from_field_bool(schema.get_field("is_hidden").unwrap(), false);
        let is_hidden_query = TermQuery::new(is_hidden_term, IndexRecordOption::Basic);
        queries.push((Occur::Must, Box::new(is_hidden_query)));
    }

    for tag in parsed_query.tags {
        let facet = Facet::from(&format!("/tag/{tag}"));
        let facet_term = Term::from_facet(schema.get_field("tag").unwrap(), &facet);
        let facet_query = TermQuery::new(facet_term, IndexRecordOption::Basic);
        queries.push((Occur::Must, Box::new(facet_query)));
    }

    let combined_query = BooleanQuery::new(queries);

    // Create a searcher
    //
    let searcher = index.reader()?.searcher();
    // Define minimum score threshold
    let min_score_threshold = 0.5;

    let top_docs = match sort {
        Some(SortChoice::DescLastModified) => get_datetime_top_docs(
            &searcher,
            &combined_query,
            "sort_modified",
            Order::Desc,
            limit,
        ),
        Some(SortChoice::AscLastModified) => get_datetime_top_docs(
            &searcher,
            &combined_query,
            "sort_modified",
            Order::Asc,
            limit,
        ),
        _ => searcher
            .search(&combined_query, &TopDocs::with_limit(limit))?
            .into_iter()
            .filter(|(score, _doc_address)| *score >= min_score_threshold)
            .map(|(_score, doc_address)| doc_address)
            .collect(),
    };

    let top_notes: Vec<Note> = top_docs
        .into_iter()
        .map(|doc_address| {
            let doc: TantivyDocument = searcher.doc(doc_address).unwrap();
            Note::from_tantivy_document(&doc, &schema)
        })
        .collect();
    if is_json {
        println!("{}", serde_json::to_string(&top_notes).unwrap());
    } else {
        for note in &top_notes {
            println!("{}\t{}", note.title, note.get_file_path().to_str().unwrap());
        }
    }

    Ok(())
}

fn get_datetime_top_docs(
    searcher: &Searcher,
    combined_query: &BooleanQuery,
    field_name: &str,
    order: Order,
    limit: usize,
) -> Vec<DocAddress> {
    let doc_search: Vec<(tantivy_DateTime, DocAddress)> = searcher
        .search(
            combined_query,
            &TopDocs::with_limit(limit).order_by_fast_field(field_name, order),
        )
        .unwrap();
    doc_search
        .into_iter()
        .map(|(_score, doc_address)| doc_address)
        .collect()
}
