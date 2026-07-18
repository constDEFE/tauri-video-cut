pub mod model;
pub mod storage;

pub use model::AppConfig;
pub use storage::{load_config, set_app_config, set_app_config_var};
