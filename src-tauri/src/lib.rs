pub mod api;
pub mod error;
pub mod login;
pub mod menu;
pub mod models;
pub mod provider_switch;
pub mod state;

pub use api::{fetch_kimi_usage, fetch_zhipu_usage, fetch_usage_for_provider, fetch_all_usage};
pub use error::{AppError, Result};
pub use login::run_login_script;
pub use menu::{build_menu, update_menu};
pub use models::{Provider, CodingPlan, UsageInfo, ALL_PROVIDERS, get_provider_by_id};
pub use provider_switch::{merge_settings, get_current_selected_provider, load_selection_state, AppSelectionState};
pub use state::AppState;
