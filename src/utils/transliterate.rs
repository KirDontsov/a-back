use translit::{CharsMapping, Transliterator};

pub struct Translit {}

impl Translit {
	pub fn convert(name: Option<String>) -> String {
		let table: CharsMapping = [
			("а", "a"),
			("б", "b"),
			("в", "v"),
			("г", "g"),
			("д", "d"),
			("е", "e"),
			("ё", "e"),
			("ж", "j"),
			("з", "z"),
			("и", "i"),
			("к", "k"),
			("л", "l"),
			("м", "m"),
			("н", "n"),
			("о", "o"),
			("п", "p"),
			("р", "r"),
			("с", "s"),
			("т", "t"),
			("у", "u"),
			("ф", "f"),
			("х", "h"),
			("ц", "c"),
			("ч", "ch"),
			("ш", "sh"),
			("щ", "shch"),
			("ы", "y"),
			("э", "e"),
			("ю", "u"),
			("я", "ya"),
			("й", "i"),
			("ъ", ""),
			("ь", ""),
			(" ", "-"),
		]
		.iter()
		.cloned()
		.collect();

		let trasliterator = Transliterator::new(table);
		trasliterator.convert(&name.unwrap_or("".to_string()).to_lowercase(), false)
	}
}
