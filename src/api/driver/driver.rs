use thirtyfour::{prelude::*, PageLoadStrategy};

pub trait Driver {}

impl dyn Driver {
	pub async fn get_driver() -> Result<WebDriver, WebDriverError> {
		let mut caps = DesiredCapabilities::chrome();

		// без загрузки изображений
		// caps.insert_browser_option(
		// 	"prefs",
		// 	serde_json::json!({
		// 		"profile.default_content_settings": {
		// 			"images": 2
		// 		},
		// 		"profile.managed_default_content_settings": {
		// 			"images": 2
		// 		}
		// 	}),
		// )?;
		// let _ = caps.set_headless();
		let _ = caps.set_page_load_strategy(PageLoadStrategy::Eager)?;
		let _ = caps.add_arg("enable-automation");
		let _ = caps.add_arg("--no-sandbox");
		let _ = caps.add_arg("--disable-extensions");
		let _ = caps.add_arg("--dns-prefetch-disable");
		let _ = caps.add_arg("--disable-gpu");
		let _ = caps.add_arg("enable-features=NetworkServiceInProcess");

		let driver = WebDriver::new("http://localhost:9515", caps).await;

		// firefox
		// let mut caps = DesiredCapabilities::firefox();
		// let _ = caps.set_page_load_strategy(PageLoadStrategy::None)?;
		// let driver = WebDriver::new("http://localhost:4444", caps).await;
		driver
	}
}
