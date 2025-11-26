use crate::AppState;
use actix_web::{web, HttpRequest};

use crate::controllers::auth::*;
use crate::controllers::avito_accounts::*;
use crate::controllers::avito_ads::*;
use crate::controllers::avito_client::*;
use crate::controllers::avito_editor::*;
use crate::controllers::avito_feeds::*;
use crate::controllers::avito_requests::*;
use crate::controllers::categories::*;
use crate::controllers::cities::*;
use crate::controllers::data_crawlers::*;
// use crate::controllers::data_operations::*;
// use crate::controllers::data_processing::*;
use crate::controllers::firms::*;
use crate::controllers::images::*;
use crate::controllers::oai_descriptions::*;
use crate::controllers::oai_reviews::*;
use crate::controllers::pages::*;
use crate::controllers::prices::*;
use crate::controllers::quotes::*;
use crate::controllers::reviews::*;
use crate::controllers::types::*;
use crate::controllers::user::*;
use crate::controllers::websocket::*;

pub fn config(conf: &mut web::ServiceConfig) {
	let scope = web::scope("/api")
		// auth
		.service(register_handler)
		.service(login_handler)
		.service(get_me_handler)
		.service(logout_handler)
		// parsers
		// .service(firms_crawler_handler)
		// .service(firms_info_crawler_handler)
		// .service(firms_address_crawler_handler)
		// .service(firms_description_crawler_handler)
		// .service(firms_reviews_crawler_handler)
		// .service(firms_images_crawler_handler)
		// .service(exact_firm_images_crawler_handler)
		// .service(firms_prices_crawler_handler)
		// .service(firms_rating_crawler_handler)
		// .service(mir_far_crawler_handler)
		// processing
		// .service(images_processing_handler)
		// .service(reviews_processing_handler)
		// .service(description_processing_handler)
		// .service(sitemap_processing_handler)
		// .service(reviews_count_processing_handler)
		// .service(urls_processing_handler)
		//user
		.service(get_users_handler)
		.service(get_user_handler)
		.service(update_user_handler)
		// quote
		.service(get_quotes_handler)
		.service(get_quote_handler)
		.service(add_quote_handler)
		// firm
		.service(get_firms_by_abbr_handler)
		.service(get_firms_by_abbr_for_map_handler)
		.service(get_firm_by_url_handler)
		.service(get_firms_search_handler)
		.service(update_firm_by_url_handler)
		// cities
		.service(get_city_handler)
		.service(get_cities_handler)
		// categories
		.service(get_category_handler)
		.service(get_category_by_abbreviation_handler)
		.service(get_categories_handler)
		// types
		.service(get_types_handler)
		// reviews
		.service(get_reviews_handler)
		.service(get_reviews_by_url_handler)
		.service(add_review_handler)
		.service(get_oai_reviews_by_url_handler)
		// description
		.service(get_oai_description_by_firm_handler)
		.service(get_oai_description_by_url_handler)
		// images
		.service(get_images_handler)
		.service(get_image_by_url_handler)
		.service(get_images_by_url_handler)
		// prices
		.service(get_prices_handler)
		.service(get_prices_by_url_handler)
		// pages
		.service(get_page_by_url_handler)
		.service(get_pages_handler)
		.service(get_pages_by_firm_handler)
		// avito
		.service(avito_crawler_handler)
		.service(get_avito_requests_handler)
		.service(get_all_avito_requests_handler)
		.service(get_avito_requests_by_user_handler)
		.service(get_avito_token_handler)
		.service(get_avito_items)
		.service(get_avito_user_profile)
		.service(get_avito_item_analytics)
		.service(get_avito_balance)
		.service(update_avito_price)
		.service(import_avito_xml)
		.service(avito_create_ad)
		.service(get_avito_categories_tree)
		.service(get_avito_category_fields)
		.service(get_avito_feeds)
		.service(get_avito_feed_by_id)
		.service(get_avito_feed_ad)
		.service(fetch_and_update_avito_ads)
		.service(create_avito_request_handler)
		.service(get_ads_by_avito_request_id_handler)
		.service(get_ads_by_avito_request_id_csv_handler)
		.service(get_avito_accounts_handler)
		.service(get_avito_account_by_id_handler)
		.service(create_avito_account_handler)
		.service(update_avito_account_handler)
		.service(delete_avito_account_handler)
		.route(
			"/ws",
			web::get().to(
				|req: HttpRequest, body: web::Payload, data: web::Data<AppState>| async move {
					let websocket_connections = data.websocket_connections.clone();
					websocket_handler(req, body, websocket_connections).await
				},
			),
		);

	conf.service(scope);
}
