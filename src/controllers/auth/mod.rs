pub mod get_me;
pub mod login;
pub mod logout;
pub mod register;
pub mod role;

pub use self::get_me::get_me_handler;
pub use self::login::login_handler;
pub use self::logout::logout_handler;
pub use self::register::register_handler;
pub use self::role::{extract, Role};
