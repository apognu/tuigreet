macro_rules! fl {
  ($message_id:literal) => {{
    i18n_embed_fl::fl!($crate::ui::MESSAGES, $message_id)
  }};

  ($message_id:literal, $($args:expr),*) => {{
    i18n_embed_fl::fl!($crate::ui::MESSAGES, $message_id, $($args), *)
  }};
}
