use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
  #[serde(default)]
  pub defaults: Defaults,
  #[serde(default)]
  pub sessions: Sessions,
  #[serde(default)]
  pub remember: Remember,
  #[serde(default)]
  pub ui: Ui,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
  pub debug: Option<String>,
  pub command: Option<String>,
  pub env: Option<Vec<String>>,
  pub user_min_uid: Option<u16>,
  pub user_max_uid: Option<u16>,
  #[serde(default)]
  pub power_no_setsid: bool,
  pub shutdown_command: Option<String>,
  pub reboot_command: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sessions {
  pub wayland_paths: Option<Vec<PathBuf>>,
  pub wayland_wrapper: Option<String>,
  pub x11_paths: Option<Vec<PathBuf>>,
  pub x11_wrapper: Option<String>,
  #[serde(default)]
  pub x11_wrapper_disabled: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Remember {
  #[serde(default)]
  pub last_user: bool,
  #[serde(default)]
  pub last_session: bool,
  #[serde(default)]
  pub last_user_session: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ui {
  pub greeting: Option<String>,
  #[serde(default)]
  pub use_issue: bool,
  #[serde(default)]
  pub show_time: bool,
  pub time_format: Option<String>,
  #[serde(default)]
  pub show_user_menu: bool,
  #[serde(default)]
  pub show_asterisks: bool,
  pub asterisks_char: Option<char>,
  pub width: Option<u64>,
  pub window_padding: Option<u64>,
  pub container_padding: Option<u64>,
  pub prompt_padding: Option<u64>,
  pub command_f_key: Option<u8>,
  pub sessions_f_key: Option<u8>,
  pub power_f_key: Option<u8>,
}
