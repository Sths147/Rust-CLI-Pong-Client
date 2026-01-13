use reqwest::Client;

pub struct Context {
  pub location: String,
  pub client: Client,
}

impl Context {
  pub fn new(location: String) -> Self {
    Context {
      location: location,
      client: Client::builder()
                      .danger_accept_invalid_certs(true)
                      .build()
                      .expect("Impossible to build new client, try again"),
    }
  }
}

impl Default for Context {
  fn default() -> Self {
    Context {
      location: String::new(),
      client: Client::builder()
                      .danger_accept_invalid_certs(true)
                      .build()
                      .expect("Impossible to build new client, try again"),
    }
  }
}
