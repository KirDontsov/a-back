use crate::models::{Category, FilteredCategory};

pub fn filter_category_record(category: &Category) -> FilteredCategory {
	FilteredCategory {
		category_id: category.category_id.to_string(),
		name: category.name.to_owned(),
		abbreviation: category.abbreviation.to_owned(),
		single_name: category.single_name.to_owned(),
		rod_name: category.rod_name.to_owned(),
		pred_name: category.pred_name.to_owned(),
		vin_name: category.vin_name.to_owned(),
		order_number: category.order_number.to_owned(),
		is_active: category.is_active.to_owned(),
	}
}
