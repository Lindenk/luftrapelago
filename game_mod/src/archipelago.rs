use archipelago_rs::{self as ap_rs, ConnectionOptions, UpdatedField::ServerTags};
use ustr::Ustr;

#[derive(Default)]
pub struct Archipelago {
  connection: ap_rs::Connection<()>,
  items: (),
  locations: (),
}

impl Archipelago {
  pub fn with_connection<S, N>(url: S, name: N) -> Self
  where
    S: Into<String>,
    N: Into<Ustr>,
  {
    let options = ConnectionOptions::default();

    let connection = ap_rs::Connection::new(url, name, "Luftrausers".into(), options);

    Self {
      connection,
      items: (),
      locations: (),
    }
  }
}
