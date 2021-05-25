extern crate reqwest;
extern crate serde_json;

use chrono;
use futures::executor;
use http::StatusCode;
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;
use std::time;

#[cfg(test)]
mod tests;

#[derive(Deserialize, Debug, Copy, Clone)]
struct Pass {
    duration: u64,
    risetime: i64,
}

#[derive(Deserialize, Debug)]
struct Request {
    altitude: f64,
    datetime: u64,
    /// geo location, latitude
    latitude: f64,
    /// geo location, longitude
    longitude: f64,
    passes: u64,
}

#[derive(Deserialize, Debug)]
struct OpenNotifyResponse {
    message: String,
    request: Request,
    response: Vec<Pass>,
}

#[derive(Copy,Clone)]
pub struct Spot {
    pub duration: chrono::Duration,
    pub risetime: chrono::DateTime<chrono::Local>,
}
impl Spot {
    pub fn spottable(&self) -> chrono::Duration {
        self.risetime - chrono::Local::now()
    }
    pub fn is_spottable(&self) -> bool {
        chrono::Local::now() >= self.risetime && chrono::Local::now() < self.risetime + self.duration
    }
}

/// Receiver object you get from `init()` and have top handle to `update()`.
pub type Receiver = mpsc::Receiver<Result<Vec<Spot>, String>>;
/// Loading error messaage you get at the first call of `update()`.
pub const LOADING: &str = "loading...";

pub fn find_upcoming( spots: &Vec<Spot> ) -> Option<&Spot> {
    // count upcoming spots
    for spot in spots {
        if spot.spottable() > chrono::Duration::zero() {
            return Some(spot);
        }
    }
    return None;
}

pub fn find_current( spots: &Vec<Spot> ) -> Option<&Spot> {
    // count upcoming spots
    for spot in spots {
        if spot.is_spottable() {
            return Some(spot);
        }
    }
    return None;
}

fn from_utc_timestamp(t: i64) -> chrono::DateTime<chrono::Local> {
    let t = chrono::NaiveDateTime::from_timestamp(t, 0);
    let t: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(t, chrono::Utc);
    return chrono::DateTime::from(t);
}

/// Spawns a thread which fetches the current ISS spotting from
/// [http://api.open-notify.org](https://http://api.open-notify.org) periodically.
/// #### Parameters
/// - `latitude`: latitude in decimal degrees of the ground station. required Range: -90..90
/// - `longitude`: longitude in decimal degress of the ground station. required Range: -180..180
/// - `altitude`: altitude in meters of the ground station. optional. Range: 0..10000
/// - `poll_mins`: Update interval:
///     - `> 0`: duration of poll period in minutes (`10` is recommended)
///     - `= 0`: thread will terminate after the first successful update.
/// #### Return value
/// - `open_notify::Receiver`: Handle this to `open_notify::update()` to get the latest ISS spotting update.
///
///    The return value is a `mpsc` *channel receiver*:
///    ```rust
///     pub type Receiver = std::sync::mpsc::Receiver<Result<open_notify::Spot, String>>;
///    ```
pub fn init(latitude: f64, longitude: f64, altitude: f64, poll_mins: u64) -> Receiver {
    // generate correct request URL depending on city is id or name
    let url = format!(
        "http://api.open-notify.org/iss/v1/?lat={}&lon={}&altitude={}",
        latitude, longitude, altitude
    );
    // fork thread that continuously fetches ISS spotting updates every <poll_mins> minutes
    let period = time::Duration::from_secs(60 * poll_mins);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(Err(LOADING.to_string())).unwrap_or(());
        loop {
            match reqwest::blocking::get(&url) {
                Ok(response) => match response.status() {
                    StatusCode::OK => {
                        let text = response.text().unwrap();
                        match serde_json::from_str(&text) {
                            Ok(w) => {
                                let mut result = Vec::new();
                                let w: OpenNotifyResponse = w;
                                for r in w.response {
                                    result.push( Spot {
                                        duration: chrono::Duration::seconds(
                                            r.duration as i64,
                                        ),
                                        risetime: from_utc_timestamp(r.risetime),
                                    } );
                                }
                                tx.send(Ok(result)).unwrap_or(());
                                if period == time::Duration::new(0, 0) {
                                    break;
                                }
                                thread::sleep(period);
                            }
                            Err(e) => tx.send(Err(e.to_string())).unwrap_or(()),
                        }
                    }
                    _ => tx.send(Err(response.status().to_string())).unwrap_or(()),
                },
                Err(_e) => (),
            }
        }
    });
    // return receiver that provides the updated ISS spotting as json string
    return rx;
}

/// Get current ISS spotting update that the spawned thread could fetched.
/// #### Parameters
/// - `receiver`: the *channel receiver* from preceded call to `opennotify_org::init()`
/// #### Returng value
/// - ⇒ `None`: No update available
/// - ⇒ `Some(Result)`: Update available
///     - ⇒ `Ok(Vec<Spot>)`: vector of upcoming spotting events
///         (see also [*open-notify* documentation](https://open-notify-api.readthedocs.io/en/latest/iss_pass.html) for details)
///     - ⇒ `Err(String)`: Error message about any occured http or json issue
///         - e.g. `500 Internal Server Error"
///         - some json parser error message if response from open-notify.org could not be parsed
pub fn update(receiver: &Receiver) -> Option<Result<Vec<Spot>, String>> {
    match receiver.try_recv() {
        Ok(spots) => Some(spots),
        Err(_e) => None,
    }
}

/// Fetch current ISS spotting update once and stop thread immediately after success.
/// Returns the result in a *future*.
/// #### Parameters
/// - `latitude`: latitude in decimal degrees of the ground station. required Range: -90..90
/// - `longitude`: longitude in decimal degress of the ground station. required Range: -180..180
/// - `altitude`: altitude in meters of the ground station. optional. Range: 0..10000
/// - `poll_mins`: Update interval:
///     - `> 0`: duration of poll period in minutes (`10` is recommended)
///     - `= 0`: thread will terminate after the first successful update.
/// #### Return value
/// - ⇒ `Ok(Vec<Spot>)`: vector of upcoming spotting events
///     (see also [*open-notify* documentation](https://open-notify-api.readthedocs.io/en/latest/iss_pass.html) for details)
/// - ⇒ `Err(String)`: Error message about any occured http or json issue
///         - e.g. `500 Internal Server Error"
///         - some json parser error message if response from open-notify.org could not be parsed
pub async fn spot(latitude: f64, longitude: f64, altitude: f64) -> Result<Vec<Spot>, String> {
    let r = init(latitude, longitude, altitude, 0);
    loop {
        match update(&r) {
            Some(response) => match response {
                Ok(spots) => return Ok(spots),
                Err(e) => {
                    if e != LOADING {
                        return Err(e);
                    }
                }
            },
            None => (),
        }
    }
}

/// synchronous functions
pub mod blocking {
    use super::*;
    /// Fetches a ISS spotting update once and stops the thread immediately after success then returns the update.
    /// #### Parameters
    /// - `latitude`: latitude in decimal degrees of the ground station. required Range: -90..90
    /// - `longitude`: longitude in decimal degress of the ground station. required Range: -180..180
    /// - `altitude`: altitude in meters of the ground station. optional. Range: 0..10000
    /// - `poll_mins`: Update interval:
    ///     - `> 0`: duration of poll period in minutes (`10` is recommended)
    ///     - `= 0`: thread will terminate after the first successful update.
    /// #### Return value
    /// - ⇒ `Ok(Vec<Spot>)`: vector of upcoming spotting events
    ///     (see also [*open-notify.org* documentation](https://openweathermap.org/current#parameter) for details)
    /// - ⇒ `Err(String)`: Error message about any occured http or json issue
    ///         - e.g. `500 Internal Server Error"
    ///         - some json parser error message if response from open-notify.org could not be parsed
    pub fn spot(latitude: f64, longitude: f64, altitude: f64) -> Result<Vec<Spot>, String> {
        // wait for result
        executor::block_on(super::spot(latitude, longitude, altitude))
    }
}
