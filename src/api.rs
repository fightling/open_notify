pub use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct Pass {
    pub duration: u64,
    pub risetime: i64,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub altitude: f64,
    pub datetime: u64,
    /// geo location, latitude
    pub latitude: f64,
    /// geo location, longitude
    pub longitude: f64,
    pub passes: u64,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub message: String,
    pub request: Request,
    pub response: Vec<Pass>,
}
