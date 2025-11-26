use crate::AppState;
use actix_web::{web, HttpRequest};

use crate::controllers::auth::*;
use crate::controllers::avito_accounts::*;
use crate::controllers::avito_ads::*;
use crate::controllers::avito_client::*;
use crate::controllers::avito_editor::*;
use crate::controllers::avito_feeds::*;
use crate::controllers::avito_requests::*;
use crate::controllers::user::*;
use crate::controllers::websocket::*;

pub fn config(conf: &mut web::ServiceConfig) {
	let scope = web::scope("/api")
		// auth
		.service(register_handler)
		.service(login_handler)
		.service(get_me_handler)
		.service(logout_handler)
		//user
		.service(get_users_handler)
		.service(get_user_handler)
		.service(update_user_handler)
		// avito
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
