use crate::models::{FilteredQuote, Quote};

pub fn filter_quote_record(quote: &Quote) -> FilteredQuote {
	FilteredQuote {
		id: quote.id.to_string(),
		text: quote.text.to_owned(),
		author: quote.author.to_owned(),
		createdAt: quote.created_at.unwrap(),
		updatedAt: quote.updated_at.unwrap(),
	}
}
