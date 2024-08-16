use greetd_ipc::Request;

pub trait SafeDebug {
  fn safe_repr(&self) -> String;
}

impl SafeDebug for Request {
  fn safe_repr(&self) -> String {
    match self {
      msg @ &Request::CancelSession => format!("{:?}", msg),
      msg @ &Request::CreateSession { .. } => format!("{:?}", msg),
      &Request::PostAuthMessageResponse { .. } => "PostAuthMessageResponse".to_string(),
      msg @ &Request::StartSession { .. } => format!("{:?}", msg),
    }
  }
}

macro_rules! fl {
  ($message_id:literal) => {{
    i18n_embed_fl::fl!($crate::ui::MESSAGES, $message_id).replace(&['\u{2068}', '\u{2069}'], "")
  }};

  ($message_id:literal, $($args:expr),*) => {{
    i18n_embed_fl::fl!($crate::ui::MESSAGES, $message_id, $($args),*).replace(&['\u{2068}', '\u{2069}'], "")
  }};
}
