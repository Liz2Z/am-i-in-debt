pub mod error;
pub mod logger;
pub mod login;
pub mod menu;
pub mod provider;
pub mod provider_switch;
pub mod providers;
pub mod state;

pub use error::{AppError, Result};
pub use menu::{build_menu, update_menu};
pub use provider::{Provider, UsageInfo};
pub use provider_switch::{
    get_current_selected_provider, load_selection_state, merge_settings, AppSelectionState,
};
pub use providers::{get_provider_by_id, PROVIDERS};
pub use state::AppState;
