use crate::models::{FilteredPriceCategory, FilteredPriceItem, PriceCategory, PriceItem};

pub fn filter_price_record(price: &PriceItem) -> FilteredPriceItem {
	FilteredPriceItem {
		price_item_id: price.price_item_id.to_string(),
		firm_id: price.firm_id.to_string(),
		price_category_id: price.price_category_id.to_string(),
		name: price.name.to_owned(),
		value: price.value.to_owned(),
	}
}

pub fn filter_price_category_record(price: &PriceCategory) -> FilteredPriceCategory {
	FilteredPriceCategory {
		price_category_id: price.price_category_id.to_string(),
		firm_id: price.firm_id.to_string(),
		name: price.name.to_owned(),
		value: price.value.to_owned(),
	}
}
