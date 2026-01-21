use reqwest::Client;

pub(crate) struct Context {
    pub(crate) location: String,
    pub(crate) client: Client,
}

impl Context {
    pub(crate) fn new(location: String) -> Self {
        Context {
            location,
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Impossible to build new client, try again"),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(String::new())
    }
}
