use crate::{api::Driver, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/mir_far")]
async fn mir_far_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	loop {
		let _: Result<(), Box<dyn std::error::Error>> = match crawler(data.clone()).await {
			Ok(item) => Ok(item),
			Err(e) => {
				println!("{:?}", e);
				Err(Box::new(e))
			}
	};
	}
	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn crawler(_data: web::Data<AppState>) -> WebDriverResult<()> {
	let driver = <dyn Driver>::get_driver().await?;

	driver.goto("https://mirfar.com/catalog/fari-audi/").await?;

	sleep(Duration::from_secs(3)).await;

	let mut blocks: Vec<WebElement> = Vec::new();
	// let mut item_xpath;
	// let mut reviews: Vec<SaveReview> = Vec::new();

	blocks = driver
		.query(By::XPath("//div[contains(@class,'item-title')]/a"))
		.all_from_selector_required()
		.await?;

	for (i, block) in blocks.clone().into_iter().enumerate() {
		let _count = i + 1;
		let block_content = block.attr("href").await?.unwrap();

		dbg!(&block_content);
	}

	println!("{}", "======");

	driver.clone().quit().await?;

	Ok(())
}
