use crate::{api::Driver, models::TwoGisFirm, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use std::env;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[get("/crawler/firms")]
async fn firms_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let _ = crawler(data).await;

	let json_response = serde_json::json!({
		"status":  "success",
	});

	HttpResponse::Ok().json(json_response)
}

async fn crawler(data: web::Data<AppState>) -> WebDriverResult<()> {
	let driver = <dyn Driver>::get_driver().await?;
	let _city_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CITY_ID")
			.expect("CRAWLER_CITY_ID not set")
			.as_str(),
	)
	.unwrap();
	let _category_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CATEGORY_ID")
			.expect("CRAWLER_CATEGORY_ID not set")
			.as_str(),
	)
	.unwrap();
	let city_name = env::var("CRAWLER_CITY_NAME").expect("CRAWLER_CITY_NAME not set");
	let category_name = env::var("CRAWLER_CATEGOTY_NAME").expect("CRAWLER_CATEGOTY_NAME not set");
	let rubric_id = env::var("CRAWLER_RUBRIC_ID").expect("CRAWLER_RUBRIC_ID not set");
	dbg!(&city_name);

	let url = format!(
		"https://2gis.ru/{}/search/{}/rubricId/{}",
		&city_name, &category_name, &rubric_id
	);

	driver.goto(url).await?;
	sleep(Duration::from_secs(1)).await;

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

	let number_of_elements_xpath = String::from("//span[contains(@class, \"_1xhlznaa\")]");

	let number_of_elements = match find_main_block(driver.clone(), number_of_elements_xpath).await {
		Ok(elem) => elem,
		Err(e) => {
			println!("error while searching name block: {}", e);
			"".to_string()
		}
	};

	let edge: i32 = (number_of_elements.parse::<f32>().unwrap_or(0.0) / 12.0).ceil() as i32;

	println!("{:?}", &edge);

	// кол-во организаций/12
	for j in 0..=edge {
		let firm_elems: Vec<WebElement> = driver.find_all(By::XPath("//body/div/div/div/div/div/div[3]/div/div/div[2]/div/div/div/div[2]/div[2]/div/div/div/div[contains(@style, 'width: 352px')]/div[2]/div/div/div")).await?;
		println!("страница: {}", j);
		sleep(Duration::from_secs(1)).await;

		let first = firm_elems
			.first()
			.unwrap_or(firm_elems.get(1).unwrap_or(firm_elems.last().unwrap()));
		let last = firm_elems
			.last()
			.unwrap_or(firm_elems.get(1).unwrap_or(firm_elems.first().unwrap()));

		let _ = last.scroll_into_view().await?;
		sleep(Duration::from_secs(2)).await;

		let _ = first.scroll_into_view().await?;
		sleep(Duration::from_secs(1)).await;

		let mut name_xpath;
		let mut firm_id_xpath;

		// номер страницы после которой все упало
		if j >= 0 {
			for (i, firm_elem) in firm_elems.clone().into_iter().enumerate() {
				println!("фирма: {}", &i);

				let _ = firm_elem.scroll_into_view().await;

				let firms_content = firm_elem.inner_html().await?;

				if firms_content.contains("_h2n9mw")
					|| firms_content.contains("_2p5l74")
					|| firms_content.contains("↗")
					|| firms_content.contains("Это интересно")
				{
					continue;
				}

				name_xpath = [
				"//*[@id='root']/div/div/div[1]/div[1]/div[3]/div/div/div[2]/div/div/div/div[2]/div[2]/div[1]/div/div/div/div[2]/div/div[",
				format!("{}", i + 1).as_str(),
				"]/div/div/a/span/span[1]",
				]
				.concat()
				.to_string();

				//*[@id='root']/div/div/div[1]/div[1]/div[3]/div/div/div[2]/div/div/div/div[2]/div[2]/div[1]/div/div/div/div[2]/div/div[12]/div/div[1]/a
				firm_id_xpath = [
				"//*[@id='root']/div/div/div[1]/div[1]/div[3]/div/div/div[2]/div/div/div/div[2]/div[2]/div[1]/div/div/div/div[2]/div/div[",
				format!("{}", i + 1).as_str(),
				"]/div/div/a",
				]
				.concat()
				.to_string();

				let firm_name_block = find_block(firm_elem.clone(), name_xpath.clone()).await?;

				let firm_id_block = match find_id_block(firm_elem.clone(), firm_id_xpath).await {
					Ok(elem) => elem,
					Err(e) => {
						println!("error while searching id block: {}", e);
						"".to_string()
					}
				};

				let split_target = String::from("/firm/");

				// TODO: попробовать заменить на regexp
				let url_part_one = *firm_id_block
					.split(&split_target)
					.collect::<Vec<&str>>()
					.get_mut(1)
					.unwrap_or(&mut "-?");

				let firm_id = *url_part_one
					.split("?")
					.collect::<Vec<&str>>()
					.get(0)
					.unwrap_or(&mut "");

				// let _ = firm_name_block.click().await?;
				// sleep(Duration::from_secs(5)).await;

				// let url_with_coords = driver.current_url().await?;
				// println!("{}", &url_with_coords);

				// let mut url_parts = url_with_coords
				// 	.path_segments()
				// 	.unwrap()
				// 	.collect::<Vec<&str>>();
				// let firm_id;
				// let coords;

				// if url_parts.contains(&"branches") {
				// 	driver.back().await?;
				// }

				let firm_name = firm_name_block.text().await?;

				// if j == 0 {
				// 	firm_id = *url_parts.get_mut(6).unwrap_or(&mut "-");
				// 	coords = *url_parts.get_mut(7).unwrap_or(&mut "-");
				// } else {
				// 	firm_id = *url_parts.get_mut(8).unwrap_or(&mut "-");
				// 	coords = *url_parts.get_mut(9).unwrap_or(&mut "-");
				// }

				// if j == 0 {
				// 	firm_id = *url_parts.get_mut(4).unwrap_or(&mut "-");
				// 	coords = *url_parts.get_mut(5).unwrap_or(&mut "-");
				// } else {
				// 	firm_id = *url_parts.get_mut(6).unwrap_or(&mut "-");
				// 	coords = *url_parts.get_mut(7).unwrap_or(&mut "-");
				// }

				dbg!(&firm_id);
				dbg!(&firm_name);
				// dbg!(&coords);

				// запись в бд
				let _ = sqlx::query_as!(
					TwoGisFirm,
					"INSERT INTO two_gis_firms (name, two_gis_firm_id, category_id) VALUES ($1, $2, $3) RETURNING *",
					&firm_name.to_string(),
					&firm_id.to_string(),
					"restaurants".to_string(),
					// &coords.replace("%2C", ", ").to_string(),
				)
				.fetch_one(&data.db)
				.await;
			}
		}

		let _ = last.scroll_into_view().await?;
		sleep(Duration::from_secs(1)).await;

		let button = find_element(driver.clone(),"//body/div/div/div/div/div/div[3]/div/div/div[2]/div/div/div/div[2]/div[2]/div/div/div/div/div[3]/div[2]/div[2]".to_string()).await?;

		// переключение пагинации
		let _ = button.click().await;
		sleep(Duration::from_secs(5)).await;
	}

	driver.quit().await?;

	Ok(())
}

pub async fn find_element(driver: WebDriver, xpath: String) -> Result<WebElement, WebDriverError> {
	let block = driver.query(By::XPath(&xpath.to_owned())).first().await?;

	Ok(block)
}

pub async fn find_main_block(driver: WebDriver, xpath: String) -> Result<String, WebDriverError> {
	let err_block = driver
		.query(By::XPath(&xpath.to_owned()))
		.or(By::XPath("//span[contains(@class, \"_1al0wlf\")]"))
		.first()
		.await?
		.inner_html()
		.await?;

	Ok(err_block)
}

pub async fn find_block(elem: WebElement, xpath: String) -> Result<WebElement, WebDriverError> {
	let block_arr = match elem
		.query(By::XPath(&xpath))
		.or(By::XPath("//span[contains(@class, \"_1al0wlf\")]"))
		.nowait()
		.all_from_selector_required()
		.await
	{
		Ok(block_elems) => block_elems,
		Err(e) => {
			println!("error while searching block: {}", e);
			Vec::<WebElement>::new()
		}
	};

	let res = block_arr.get(0).unwrap_or(&elem).to_owned();

	Ok(res)
}

pub async fn find_id_block(elem: WebElement, xpath: String) -> Result<String, WebDriverError> {
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

	let res = match block_arr.get(0).unwrap_or(&elem).attr("href").await {
		Ok(block_elem) => block_elem,
		Err(e) => {
			println!("error while extracting text: {}", e);
			Some("".to_string())
		}
	};

	Ok(res.unwrap_or("".to_string()))
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
