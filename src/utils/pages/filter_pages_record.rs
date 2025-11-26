use crate::models::{FilteredPage, Page};

pub fn filter_page_record(page: &Page) -> FilteredPage {
	FilteredPage {
		page_id: page.page_id.to_string(),
		firm_id: match page.firm_id {
			Some(x) => x.to_string(),
			None => "".to_string(),
		},
		page_category_id: match page.page_category_id {
			Some(x) => x.to_string(),
			None => "".to_string(),
		},
		user_id: match page.user_id {
			Some(x) => x.to_string(),
			None => "".to_string(),
		},
		url: page.url.to_owned(),
		page_photo: page.page_photo.to_owned(),
		oai_value: page.oai_value.to_owned(),
		created_ts: page.created_ts.to_owned(),
	}
}
