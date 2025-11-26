use crate::models::{FilteredOAIDescription, OAIDescription};

pub fn filter_oai_description_record(description: &OAIDescription) -> FilteredOAIDescription {
	FilteredOAIDescription {
		oai_description_id: description.oai_description_id.to_string(),
		firm_id: description.firm_id.to_string(),
		oai_description_value: description.oai_description_value.to_owned(),
	}
}
