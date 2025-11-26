use crate::models::{FilteredAddReview, FilteredOAIReview, FilteredReview, OAIReview, Review};

pub fn filter_review_record(review: &Review) -> FilteredReview {
	FilteredReview {
		review_id: review.review_id.to_string(),
		two_gis_firm_id: review.two_gis_firm_id.to_owned(),
		firm_id: review.firm_id.to_string(),
		author: review.author.to_owned(),
		date: review.date.to_owned(),
		text: review.text.to_owned(),
		rating: review.rating.to_owned(),
	}
}

pub fn filter_add_review_record(review: &Review) -> FilteredAddReview {
	FilteredAddReview {
		review_id: review.review_id.to_string(),
		firm_id: review.firm_id.to_string(),
		author: review.author.to_owned(),
		date: review.date.to_owned(),
		text: review.text.to_owned(),
	}
}

pub fn filter_oai_review_record(review: &OAIReview) -> FilteredOAIReview {
	FilteredOAIReview {
		oai_review_id: review.oai_review_id.to_string(),
		firm_id: review.firm_id.to_string(),
		text: review.text.to_owned(),
	}
}
