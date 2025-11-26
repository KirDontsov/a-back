use crate::{
	api::Driver,
	models::{Count, Firm, Review, SaveReview},
	utils::{get_counter, update_counter},
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/reviews")]
async fn firms_reviews_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	loop {
		let _: Result<(), Box<dyn std::error::Error>> = match crawler(data.clone()).await {
			Ok(x) => Ok(x),
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

async fn crawler(data: web::Data<AppState>) -> WebDriverResult<()> {
	let counter_id: String = String::from("4bb99137-6c90-42e6-8385-83c522cde804");
	let table = String::from("firms");
	let city_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CITY_ID")
			.expect("CRAWLER_CITY_ID not set")
			.as_str(),
	)
	.unwrap();
	let category_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CATEGORY_ID")
			.expect("CRAWLER_CATEGORY_ID not set")
			.as_str(),
	)
	.unwrap();
	let city_name = env::var("CRAWLER_CITY_NAME").expect("CRAWLER_CITY_NAME not set");
	let category_name = env::var("CRAWLER_CATEGOTY_NAME").expect("CRAWLER_CATEGOTY_NAME not set");
	let _rubric_id = env::var("CRAWLER_RUBRIC_ID").expect("CRAWLER_RUBRIC_ID not set");

	let firms_count =
		Count::count_firms_by_city_category(&data.db, table.clone(), city_id, category_id)
			.await
			.unwrap_or(0);

	// получаем из базы начало счетчика
	let start: i64 = get_counter(&data.db, &counter_id).await;
	dbg!(&start);

	let driver = <dyn Driver>::get_driver().await?;

	for j in start.clone()..=firms_count {
		println!("№: {}", &j + 1);
		let firm =
			// Firm::get_firm_with_empty_field(&data.db, table.clone(), empty_field.clone(), j)
			// .await
			// .unwrap();

			Firm::get_firm_by_city_category(&data.db, table.clone(), city_id, category_id, j)
				.await
				.unwrap();
		let mut reviews: Vec<SaveReview> = Vec::new();

		// проверка на дубликат
		let existed_reviews = sqlx::query_as!(
			Review,
			r#"SELECT * FROM reviews WHERE firm_id = $1;"#,
			&firm.firm_id
		)
		.fetch_one(&data.db)
		.await;

		if existed_reviews.is_ok() {
			println!("{}", &firm.firm_id);
			println!("Already exists");
			continue;
		}

		driver
			.goto(format!(
				"https://2gis.ru/{}/search/{}/firm/{}/tab/reviews",
				&city_name,
				&category_name,
				&firm.two_gis_firm_id.clone().unwrap()
			))
			.await?;
		sleep(Duration::from_secs(5)).await;

		let error_block = match find_error_block(driver.clone()).await {
			Ok(img_elem) => img_elem,
			Err(e) => {
				println!("error while searching error block: {}", e);
				"".to_string()
			}
		};

		if error_block.contains("Что-то пошло не так") {
			driver.refresh().await?;
		}

		let main_block = match find_main_block(driver.clone()).await {
			Ok(img_elem) => img_elem,
			Err(e) => {
				let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
				dbg!(&counter);
				println!("error while searching main block: {}", e);
				"".to_string()
			}
		};

		if main_block.contains("Нет отзывов")
			|| main_block.contains("Филиал удалён из справочника")
			|| main_block.contains("Филиал временно не работает")
			|| main_block.contains("Скоро открытие")
		{
			continue;
		}

		let mut author_xpath;
		let mut date_xpath;
		let mut text_xpath;
		let mut rating_xpath;

		let mut blocks: Vec<WebElement> = Vec::new();

		// кол-во отзывов
		let reviews_count = driver
			.query(By::XPath("//*[contains(text(),'Отзывы')]/span"))
			.or(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div/div"))
			.first()
			.await?
			.inner_html()
			.await?
			.parse::<f32>()
			.unwrap_or(0.0);

		if reviews_count == 0.0 {
			continue;
		}

		// let edge: i32 = ((if reviews_count > 500.0 {
		// 	100.0
		// } else {
		// 	reviews_count
		// }) / 12.0)
		// 	.ceil() as i32;

		// скролим в цикле
		for _ in 0..(reviews_count / 12.0).ceil() as i32 {
			blocks = driver.query(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div/div[2]/div[2]/div")).all_from_selector_required().await?;
			let last = blocks.last().unwrap();
			last.scroll_into_view().await?;
			tokio::time::sleep(Duration::from_secs(1)).await;
		}

		for (i, block) in blocks.clone().into_iter().enumerate() {
			let count = i + 1;

			block.scroll_into_view().await?;

			let block_content = match block.inner_html().await {
				Ok(elem) => elem,
				Err(e) => {
					let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
					dbg!(&counter);
					println!("error while searching author block: {}", e);
					"".to_string()
				}
			};

			if block_content.contains("Неподтвержденные отзывы")
				|| block_content.contains("Все отзывы")
				|| block_content.contains("Загрузить ещё")
				|| block_content.contains("официальный ответ")
				|| block_content.contains("С ответами")
				|| block_content.contains("Люди говорят")
				|| block_content.contains("Оцените и оставьте отзыв")
				|| block_content.contains("оценки")
				|| block_content.contains("оценок")
				|| block_content.contains("оценка")
				|| block_content.contains("ответ")
				|| block_content.contains("/5")
			{
				continue;
			}

			author_xpath = format!("//*[@id='root']/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div/div[2]/div[2]/div[{}]/div[1]/div/div[1]/div[2]/span/span[1]/span", count);
			date_xpath = format!("//*[@id='root']/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div/div[2]/div[2]/div[{}]/div[1]/div/div[1]/div[2]/div", count);
			text_xpath = format!("//*[@id='root']/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div/div[2]/div[2]/div[{}]/div[3]/div[1]/a", count);
			rating_xpath = format!("//*[@id='root']/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div/div[2]/div[2]/div[{}]/div[1]/div/div[2]/div/div[1]/span", count);

			let author = match find_block(block.clone(), author_xpath.clone()).await {
				Ok(elem) => elem,
				Err(e) => {
					let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
					dbg!(&counter);
					println!("error while searching author block: {}", e);
					"".to_string()
				}
			};

			let date = match find_block(block.clone(), date_xpath.clone()).await {
				Ok(elem) => elem,
				Err(e) => {
					let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
					dbg!(&counter);
					println!("error while searching date block: {}", e);
					"".to_string()
				}
			};

			let text = match find_block(block.clone(), text_xpath.clone()).await {
				Ok(elem) => elem,
				Err(e) => {
					let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
					dbg!(&counter);
					println!("error while searching text block: {}", e);
					"".to_string()
				}
			};

			let rating = match find_blocks(block.clone(), rating_xpath.clone()).await {
				Ok(elem) => elem.to_string(),
				Err(e) => {
					let counter = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;
					dbg!(&counter);
					println!("error while searching text block: {}", e);
					"".to_string()
				}
			};

			reviews.push(SaveReview {
				firm_id: firm.firm_id.clone(),
				two_gis_firm_id: firm.two_gis_firm_id.clone().unwrap(),
				author: author.clone(),
				date: date.clone(),
				text: text.replace("\n", " "),
				rating,
			});
		}

		// запись в бд
		for review in reviews {
			let _ = sqlx::query_as!(
				Review,
				"INSERT INTO reviews (firm_id, two_gis_firm_id, author, date, text, rating, parsed) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
				review.firm_id,
				review.two_gis_firm_id,
				review.author,
				review.date,
				review.text,
				review.rating,
				true
			)
			.fetch_one(&data.db)
			.await;

			dbg!(&review);
		}
		// обновляем в базе счетчик
		let _ = update_counter(&data.db, &counter_id, &(j + 1).to_string()).await;

		println!("id: {}", &firm.two_gis_firm_id.clone().unwrap());
		println!("{}", "======");
	}

	driver.clone().quit().await?;

	Ok(())
}

pub async fn find_block(elem: WebElement, xpath: String) -> Result<String, WebDriverError> {
	let block_arr = match elem
		.query(By::XPath(&xpath))
		.nowait()
		.all_from_selector_required()
		.await
	{
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while searching block: {}", e);
			Vec::<WebElement>::new()
		}
	};

	let res = match block_arr.get(0).unwrap_or(&elem).text().await {
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while extracting text: {}", e);
			"".to_string()
		}
	};

	Ok(res)
}

pub async fn find_blocks(elem: WebElement, xpath: String) -> Result<usize, WebDriverError> {
	let length = match elem
		.query(By::XPath(&xpath))
		.nowait()
		.all_from_selector_required()
		.await
	{
		Ok(block_elem) => block_elem.len(),
		Err(e) => {
			println!("error while searching block: {}", e);
			Vec::<WebElement>::new().len()
		}
	};

	Ok(length)
}

pub async fn find_error_block(driver: WebDriver) -> Result<String, WebDriverError> {
	let err_block = driver
		.query(By::Id("root"))
		.first()
		.await?
		.inner_html()
		.await?;

	Ok(err_block)
}

pub async fn find_main_block(driver: WebDriver) -> Result<String, WebDriverError> {
	let block = driver
		.query(By::XPath("//*[@id='root']/div/div/div[1]/div[1]/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div[1]/div/div/div/div"))
		.or(By::XPath("//body/div/div/div/div/div/div[3]/div[2]/div/div/div/div/div[2]/div[2]/div/div/div/div/div/div"))
		.first()
		.await?
		.inner_html()
		.await?;

	Ok(block)
}
