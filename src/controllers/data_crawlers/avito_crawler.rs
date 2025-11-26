use crate::{
	api::Driver,
	AppState,
};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use csv::Writer;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[allow(unreachable_code)]
#[get("/crawler/avito")]
async fn avito_crawler_handler(
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let _: Result<(), Box<dyn std::error::Error>> = match crawler(data.clone()).await {
		Ok(x) => Ok(x),
		Err(e) => {
			println!("{:?}", e);
			Err(Box::new(e))
		}
	};

	let json_response = serde_json::json!({
		"status":  "success",
	});
	HttpResponse::Ok().json(json_response)
}

async fn crawler(_data: web::Data<AppState>) -> WebDriverResult<()> {
	let search_query = "ремонт фар";
	let region_query = "Санкт-Петербург";

	let url = format!("https://www.avito.ru");

	let utc: DateTime<Utc> = Utc::now() + chrono::Duration::try_hours(3).expect("hours err");

	let mut wtr = Writer::from_path(format!("avito_{}.csv", utc.format("%d-%m-%Y_%H:%M:%S")))
		.expect("no file");

	wtr.write_record(&[
		"Дата",
		"Город",
		"Метро",
		"Район",
		"Запрос",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
	])
	.expect("write record err");

	wtr.write_record(&[
		format!("{}", utc.format("%d-%m-%Y_%H:%M:%S")).as_str(),
		region_query,
		"-",
		"-",
		search_query,
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
		"",
	])
	.expect("write record err");

	wtr.write_record(&[
		"Поз.",
		"id",
		"Название",
		"Ссылка",
		"Цена",
		"Продвижение",
		"Категории",
		"Поиск (запрос)",
		"id Продавца",
		"Продавец",
		"Рейтинг",
		"Кол-во отзывов",
		"Дата регистрации",
		"Кол-во объявлений",
		"Описание",
		"Город (запрос)",
		"Адрес",
		"Дата",
		"Просмотров",
		"Просмотров сегодня",
	])
	.expect("write record err");

	let driver = <dyn Driver>::get_driver().await?;

	driver.goto(url).await?;

	sleep(Duration::from_secs(2)).await;

	// выбор региона
	let region_arr = match find_elements(
		driver.clone(),
		"//div[contains(@class, \"main-richTitleWrapper__content\")]".to_string(),
		"//body/div[1]/div/buyer-location/div/div/div[2]/div/div[1]/div/div/div[4]/div[1]/div"
			.to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let region_btn = region_arr.get(0).expect("no region_btn");
	// открытие модального окна
	region_btn.click().await?;
	sleep(Duration::from_secs(5)).await;

	// кнопка очистки поиска
	let clear_arr = match find_elements(
		driver.clone(),
		"//div[contains(@class, \"styles-module-controlIcon\")]".to_string(),
		"//body/div[4]/div[43]/div/div[2]/div/div/div/div/div[1]/div[1]/div/div[2]".to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let clear_btn = clear_arr.get(0).expect("no clear_btn");
	// очистка строки поиска региона
	clear_btn.click().await?;
	sleep(Duration::from_secs(2)).await;

	// строка выбора региона
	let region_input_arr = match find_elements(
		driver.clone(),
		"//input[contains(@class, \"styles-module-searchInput\")]".to_string(),
		"//body/div[4]/div[43]/div/div[2]/div/div/div/div/div[1]/div/div/div[1]/div/input"
			.to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let region_input = region_input_arr.get(0).expect("no region_input");
	region_input.send_keys(region_query).await?;
	sleep(Duration::from_secs(2)).await;

	// подсказки выбора региона
	let region_suggest_arr = match find_elements(
		driver.clone(),
		"//div[contains(@class, \"styles-module-dropdown\")]/div/div/button[1]".to_string(),
		"//body/div[4]/div[44]/div/div/div/div/div/button[1]".to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let region_suggest = region_suggest_arr.get(0).expect("no region_suggest");
	// нажимаем на подсказку выбора региона
	region_suggest.click().await?;
	sleep(Duration::from_secs(2)).await;

	// кнопка подтверждение выбора региона
	let confirm_region_btn_arr = match find_elements(
			driver.clone(),
			"//div[contains(@class, \"popup-buttons-\")]/div[2]/button".to_string(),
			"//body/div[4]/div[43]/div/div[2]/div/div/div/div/div[2]/div[2]/div/div[2]/div[2]/div[2]/button".to_string(),
		)
		.await
		{
			Ok(res) => res,
			Err(e) => {
				println!("error while searching categories block: {}", e);
				driver.clone().quit().await?;
				Vec::new()
			}
		};

	let confirm_region_btn = confirm_region_btn_arr
		.get(0)
		.expect("no confirm_region_btn");
	// нажимаем на кнопку подтверждение выбора региона
	confirm_region_btn.click().await?;
	sleep(Duration::from_secs(5)).await;

	// заполнение поиска
	let input_arr = match find_elements(driver.clone(),
		"//div[contains(@class, \"suggest-input\")]/label/div/div/input".to_string(),
		"//body/div[1]/div/div[4]/div/div[1]/div/div/div[3]/div[2]/div[1]/div/div/label/div/div/input".to_string()
	).await {
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let input = input_arr.get(0).expect("no input");

	let search_array = search_query.split(" ");
	for item in search_array {
		input.send_keys(item).await?;
		sleep(Duration::from_secs(1)).await;
		input.send_keys(" ").await?;
	}

	sleep(Duration::from_secs(2)).await;

	let suggest_arr = match find_elements(
		driver.clone(),
		"//div[contains(@class, \"suggest-dropdownItems\")]/button[1]".to_string(),
		"//body/div[3]/div[2]/div/div/div/div/div/div".to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let suggest = suggest_arr.get(0).expect("no suggest");
	// нажимаем на подсказку
	suggest.click().await?;
	sleep(Duration::from_secs(5)).await;

	let categories = match find_elements(
		driver.clone(),
		"//div[contains(@class, \"breadcrumbs-root\")]/span/a".to_string(),
		"//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[1]/span[1]/a".to_string(),
	)
	.await
	{
		Ok(res) => res,
		Err(e) => {
			println!("error while searching categories block: {}", e);
			driver.clone().quit().await?;
			Vec::new()
		}
	};

	let mut category_arr = Vec::new();

	for category in categories {
		let res = category.find(By::Tag("span")).await?.text().await?;
		if res != "Главная" {
			category_arr.push(res);
		}
	}

	let categories_str = category_arr
		.iter()
		.map(|x| format!("{}; ", x.to_string()))
		.collect::<String>();

	// page-title-count
	let ads_count = match find_text(
		driver.clone(),
		"//span[contains(@class, \"page-title-count\")]".to_string(),
		"//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[2]/div/span".to_string(),
	)
	.await
	{
		Ok(elem) => elem.parse::<f32>().unwrap_or(0.0),
		Err(e) => {
			println!("error while searching ads_count block: {}", e);
			driver.clone().quit().await?;
			0.0
		}
	};

	println!("ads_count {}", ads_count.clone());

	let mut position;

	'outer: for j in 0..=(ads_count / 50.0).ceil() as i32 {
		// feed
		//scroll

		let blocks = match find_elements(
			driver.clone(),
			"//div[contains(@class, \"items-items\")][1]/div".to_string(),
			"//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[4]/div[2]/div".to_string(),
		)
		.await
		{
			Ok(res) => res,
			Err(e) => {
				println!("error while searching blocks block: {}", e);
				driver.clone().quit().await?;
				Vec::new()
			}
		};

		let last = blocks.last().expect("no blocks");
		last.scroll_into_view().await?;

		for (i, block) in blocks.clone().into_iter().enumerate() {
			let count = i + 1;

			position = count - 1;
			if j != 0 {
				position = (50 * j) as usize + count as usize - 1;
			}

			// проверяем есть ли рекламное объявление
			let ads_banner_exists = match check_if_block_exists(driver.clone(),
				"//div[contains(@class, \"items-banner\")]".to_string(),
				"body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[4]/div[2]/div[1]".to_string()
			).await {
				Ok(elem) => elem,
				Err(e) => {
					println!("error while searching ads_banner_arr block: {}", e);
					driver.clone().quit().await?;
					false
				}
			};

			if ads_banner_exists {
				if count == 1 {
					continue;
				}
			};

			block.scroll_into_view().await?;

			let href_full = match find_href_block(driver.clone(),
				format!("//div[contains(@class, \"items-items\")]/div[{}]/div/div/div[2]/div[2]/div/a", count),
				format!("//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[3]/div[2]/div[{}]/div/div/div[2]/div[2]/div/a", count)
			).await {
				Ok(elem) => elem,
				Err(e) => {
					println!("error while searching href block: {}", e);
					driver.clone().quit().await?;
					"".to_string()
				}
			};

			let href_arr = href_full.split("?").collect::<Vec<&str>>();
			let href_str = href_arr.get(0).expect("no href_str");
			let href = format!("https://avito.ru{}", href_arr.get(0).expect("no href_str"));
			let id = href_str.split("_").last().expect("no href");

			let title = match find_text(driver.clone(),
				format!("//div[contains(@class, \"items-items\")]/div[{}]/div/div/div[2]/div[2]/div/a/h3", count),
				format!("//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[3]/div[2]/div[{}]/div/div/div[2]/div[2]/div/a/h3", count)).await {
				Ok(elem) => elem,
				Err(e) => {
					println!("error while searching title block: {}", e);
					driver.clone().quit().await?;
					"".to_string()
				}
			};

			let price = match find_price_block(driver.clone(),
				format!("//div[contains(@class, \"items-items\")]/div[{}]/div/div/div[2]/div[3]/span/div/p/meta[2]", count),
				format!("//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[3]/div[2]/div[{}]/div/div/div[2]/div[3]/span/div/p/meta[2]", count)
			).await {
				Ok(elem) => elem,
				Err(e) => {
					println!("error while searching price_block block: {}", e);
					driver.clone().quit().await?;
					"".to_string()
				}
			};

			let paid = match check_if_block_exists(driver.clone(),
				format!("//div[contains(@class, \"items-items\")]/div[{}]/div/div/div[2]/div[last()]/div[2]/div/i", count),
				format!("//body/div[1]/div/buyer-location/div/div/div[2]/div/div[2]/div[3]/div[3]/div[3]/div[2]/div[{}]/div/div/div[2]/div[last()]/div[2]/div/i", count)
			).await {
				Ok(elem) => elem,
				Err(e) => {
					println!("error while searching paid block: {}", e);
					driver.clone().quit().await?;
					false
				}
			};

			// Переход в новую вкладку
			let handle = driver.window().await?;
			driver.new_tab().await?;
			let handles = driver.windows().await?;
			driver.switch_to_window(handles[1].clone()).await?;
			driver
				.goto(format!("https://www.avito.ru{}", &href_str))
				.await?;
			sleep(Duration::from_secs(5)).await;

			let seller_name_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-seller-info-name\")]/a".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[2]/div[1]/div/div/div[3]/div[2]/div/div/div/div[1]/div[1]/div[1]/div[1]/div[1]/a".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching seller_name block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let seller_link_id_full = match seller_name_arr.get(0) {
				Some(x) => x.attr("href").await?.expect("no seller_link_id"),
				None => "".to_string(),
			};

			let seller_id = match seller_link_id_full.split("?").collect::<Vec<&str>>().get(0) {
				Some(x) => x,
				None => "",
			};

			let seller_name = match seller_name_arr.get(0) {
				Some(x) => x.text().await?,
				None => "".to_string(),
			};

			// рейтинг
			let rating_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-seller-info-rating-score\")]".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[2]/div[1]/div/div/div[3]/div[2]/div/div/div/div[1]/div[1]/div[1]/div[1]/div[2]/span[1]".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching rating block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let rating = match rating_arr.get(0) {
				Some(x) => x.text().await?,
				None => "".to_string(),
			};

			// кол-во отзывов
			let reviews_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-seller-info-rating\")]/a".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[2]/div[1]/div/div/div[3]/div[2]/div/div/div/div[1]/div[1]/div[1]/div[1]/div[2]/a".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching reviews block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let reviews = match reviews_arr.get(0) {
				Some(x) => x
					.text()
					.await?
					.replace("отзывов", "")
					.replace("отзыва", "")
					.replace("отзыв", "")
					.replace(" ", ""),
				None => "".to_string(),
			};

			// когда зареган
			let register_date_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-seller-info-value\")][last()]/div[last()]".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[2]/div[1]/div/div/div[3]/div[2]/div/div/div/div[1]/div[1]/div[1]/div[3]/div".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching register_date block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let register_date_text = match register_date_arr.get(0) {
				Some(x) => x.text().await?,
				None => "".to_string(),
			};

			let register_date = if register_date_text.contains("Пользователь")
				|| register_date_text.contains("отзыв")
				|| register_date_text.contains("Завершено")
			{
				"".to_string()
			} else {
				register_date_text.replace("На Авито ", "")
			};

			// кол-во объявлений продавца
			let seller_ads_count_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-seller-info-favorite-seller-buttons\")]/div/div/div[1]/a/span".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[2]/div[1]/div/div/div[3]/div[2]/div/div/div/div[2]/div/div/div/div/a/span".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching seller_ads_count block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let seller_ads_count = match seller_ads_count_arr.get(0) {
				Some(x) => x
					.text()
					.await?
					.replace("пользователя", "")
					.replace("объявления", "")
					.replace("объявлений", "")
					.replace(" ", ""),
				None => "".to_string(),
			};

			// Описание
			let description_blocks_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-item-description-html\")]/p".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[1]/div[2]/div[3]/div/div/p".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching seller_ads_count block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let mut descriptions_arr: Vec<String> = Vec::new();

			for desc_block in description_blocks_arr {
				desc_block.scroll_into_view().await?;

				let description = desc_block.text().await?;
				descriptions_arr.push(description);
			}

			let description_string = descriptions_arr.join("; ");

			// Адрес
			let address_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-item-address__string\")]".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[1]/div[2]/div[1]/div[3]/div/div[1]/div[1]/div/span".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching address block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let address = match address_arr.get(0) {
				Some(x) => x.text().await?,
				None => "".to_string(),
			};

			// Дата
			let date_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-item-footer\")]/article/p/span[2]".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[1]/div[2]/div[4]/div/article/p/span[2]".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching date block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let date = match date_arr.get(0) {
				Some(x) => x.text().await?.replace("На Авито ", "").replace("· ", ""),
				None => "".to_string(),
			};

			// Просмотров
			let views_arr = match find_elements(
				driver.clone(),
				"//div[contains(@class, \"style-item-footer\")]/article/p/span[3]".to_string(),
				"//body/div[1]/div/div[3]/div[1]/div/div[2]/div[3]/div/div[1]/div[2]/div[4]/div/article/p/span[3]".to_string(),
			)
			.await
			{
				Ok(res) => res,
				Err(e) => {
					println!("error while searching views block: {}", e);
					driver.clone().quit().await?;
					Vec::new()
				}
			};

			let views_full = match views_arr.get(0) {
				Some(x) => x
					.text()
					.await?
					.replace("просмотра", "")
					.replace("просмотров", "")
					.replace("просмотр", "")
					.replace("· ", "")
					.replace("  ", ""),
				None => "".to_string(),
			};

			let views_str = views_full.split("+").collect::<Vec<&str>>();
			let views = match views_str.get(0) {
				Some(x) => x.replace("(", "").replace("  ", ""),
				None => "".to_string(),
			};

			let views_today = match views_str.get(1) {
				Some(x) => x
					.replace(")", "")
					.replace("+", "")
					.replace("сегодня", "")
					.replace(" ", ""),
				None => "".to_string(),
			};

			// === RESULT ===

			driver.close_window().await?;
			driver.switch_to_window(handle.clone()).await?;
			sleep(Duration::from_secs(2)).await;

			println!("id {}", &id);

			wtr.write_record(&[
				position.to_string().as_str(),
				id,
				title.replace("\"", "").as_str(),
				href.as_str(),
				price.as_str(),
				paid.to_string().as_str(),
				categories_str.as_str(),
				search_query,
				seller_id,
				seller_name.as_str(),
				rating.as_str(),
				reviews.as_str(),
				register_date.as_str(),
				seller_ads_count.as_str(),
				description_string.as_str(),
				region_query,
				address.as_str(),
				date.as_str(),
				views.as_str(),
				views_today.as_str(),
			])
			.expect("write record err");

			// reviews.push(SaveReview {
			// 	firm_id: firm.firm_id.clone(),
			// 	two_gis_firm_id: firm.two_gis_firm_id.clone().expect(),
			// 	author: author.clone(),
			// 	date: date.clone(),
			// 	text: text.replace("\n", " "),
			// 	rating,
			// });
		}

		let main_arr = match find_elements(
			driver.clone(),
			"//div[contains(@class, \"index-content\")]".to_string(),
			"//div[contains(@class, \"index-inner\")]".to_string(),
		)
		.await
		{
			Ok(elem) => elem,
			Err(e) => {
				println!("error while searching main_arr block: {}", e);
				driver.clone().quit().await?;
				Vec::new()
			}
		};

		let main = main_arr.get(0).expect("no main");

		if main.inner_html().await?.contains("других городов") {
			println!("====== break ======");
			driver.clone().quit().await?;
			break 'outer;
		}

		// пагинация
		let button_arr = match find_elements(
			driver.clone(),
			"//div[contains(@class, \"pagination-pagination\")]/nav/ul/li[last()]/a".to_string(),
			"//body/div[1]/div/div[4]/div/div[2]/div[3]/div[3]/div[5]/nav/ul/li[6]/a".to_string(),
		)
		.await
		{
			Ok(elem) => elem,
			Err(e) => {
				println!("error while searching button_arr block: {}", e);
				driver.clone().quit().await?;
				Vec::new()
			}
		};

		let button = button_arr.get(0).expect("no button");

		button.click().await?;
		sleep(Duration::from_secs(5)).await;

		// запись в бд
		// for review in reviews {
		// 	let _ = sqlx::query_as!(
		// 		Review,
		// 		"INSERT INTO reviews (firm_id, two_gis_firm_id, author, date, text, rating, parsed) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
		// 		review.firm_id,
		// 		review.two_gis_firm_id,
		// 		review.author,
		// 		review.date,
		// 		review.text,
		// 		review.rating,
		// 		true
		// 	)
		// 	.fetch_one(&data.db)
		// 	.await;

		// 	dbg!(&review);
		// }

		println!("{}", "======");
	}
	wtr.flush()?;
	driver.clone().quit().await?;

	Ok(())
}

pub async fn find_href_block(
	driver: WebDriver,
	xpath: String,
	xpath2: String,
) -> Result<String, WebDriverError> {
	let elem = driver
		.query(By::XPath(&xpath))
		.or(By::XPath(&xpath2))
		.nowait()
		.first()
		.await?
		.attr("href")
		.await?
		.expect("no href");

	Ok(elem)
}

pub async fn find_price_block(
	driver: WebDriver,
	xpath: String,
	xpath2: String,
) -> Result<String, WebDriverError> {
	let res = driver
		.query(By::XPath(&xpath))
		.or(By::XPath(&xpath2))
		.nowait()
		.first()
		.await?
		.attr("content")
		.await?
		.expect("no price");

	Ok(res)
}

pub async fn find_text(
	driver: WebDriver,
	xpath: String,
	xpath2: String,
) -> Result<String, WebDriverError> {
	let el_exists = match check_if_block_exists(driver.clone(), xpath.clone(), xpath2.clone()).await
	{
		Ok(elem) => elem,
		Err(e) => {
			println!("error while searching find_text block: {}", e);
			driver.clone().quit().await?;
			false
		}
	};

	if el_exists {
		let elem = driver
			.query(By::XPath(&xpath))
			.or(By::XPath(&xpath2))
			.nowait()
			.first()
			.await?
			.text()
			.await?;

		Ok(elem)
	} else {
		Ok("".to_string())
	}
}

pub async fn find_elements(
	driver: WebDriver,
	xpath: String,
	xpath2: String,
) -> Result<Vec<WebElement>, WebDriverError> {
	let el_exists = match check_if_block_exists(driver.clone(), xpath.clone(), xpath2.clone()).await
	{
		Ok(elem) => elem,
		Err(e) => {
			println!("error while searching ads_banner_arr block: {}", e);
			driver.clone().quit().await?;
			false
		}
	};

	if el_exists {
		let elems = driver
			.query(By::XPath(&xpath))
			.or(By::XPath(&xpath2))
			.nowait()
			.all_from_selector_required()
			.await?;

		Ok(elems)
	} else {
		Ok(Vec::new())
	}
}

pub async fn check_if_block_exists(
	driver: WebDriver,
	xpath: String,
	xpath2: String,
) -> Result<bool, WebDriverError> {
	let exists = driver
		.query(By::XPath(&xpath))
		.or(By::XPath(&xpath2))
		.nowait()
		.exists()
		.await?;

	Ok(exists)
}
