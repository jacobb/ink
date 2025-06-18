mod index;
mod index_updater;
mod query;

pub use self::index::create_index_and_add_documents;
pub use self::query::search_index;
