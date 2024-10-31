use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub backend: Backend,
    pub frontend: Frontend,
}

#[derive(Deserialize, Debug)]
pub struct Backend {
    pub proxy_pass: String,
    pub ssl_certificate: String,
    pub ssl_certificate_key: String,
}

#[derive(Deserialize, Debug)]
pub struct Frontend {
    pub listen: String,
    pub server_name: String,
}
