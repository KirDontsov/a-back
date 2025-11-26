use crate::models::{FilteredType, Type};

pub fn filter_type_record(type_item: &Type) -> FilteredType {
	FilteredType {
		type_id: type_item.type_id.to_string(),
		name: type_item.name.to_owned(),
		abbreviation: type_item.abbreviation.to_owned(),
	}
}
