extern crate reqwest;
extern crate serde_json;

use futures::executor;
use http::StatusCode;
use std::sync::mpsc;
use std::thread;
use std::time;

mod api;
mod spot;

#[cfg(test)]
mod tests;

pub use spot::*;

/// Receiver object you get from `init()` and have top handle to `update()`.
pub type Receiver = mpsc::Receiver<Result<Vec<Spot>, String>>;
/// Loading error messaage you get at the first call of `update()`.
pub const LOADING: &str = "loading...";

/// Spawns a thread which fetches the current ISS spotting from
/// [http://api.open-notify.org](https://http://api.open-notify.org) periodically.
/// #### Parameters
/// - `latitude`: latitude in decimal degrees of the ground station. required Range: -90..90
/// - `longitude`: longitude in decimal degress of the ground station. required Range: -180..180
/// - `altitude`: altitude in meters of the ground station. optional. Range: 0..10000
/// - `n`: number of spotting events to fetch (<=100)
/// - `poll_mins`: Update interval:
///     - `> 0`: duration of poll period in minutes (`90` is recommended)
///     - `= 0`: thread will terminate after the first successful update.
/// #### Return value
/// - `open_notify::Receiver`: Handle this to `open_notify::update()` to get the latest ISS spotting update.
///
///    The return value is a `mpsc` *channel receiver*:
///    ```rust
///     pub type Receiver = std::sync::mpsc::Receiver<Result<open_notify::Spot, String>>;
///    ```
pub fn init(latitude: f64, longitude: f64, altitude: f64, n: u8, poll_mins: u64) -> Receiver {
    // generate correct request URL depending on city is id or name
    let url = format!(
        "http://api.open-notify.org/iss/v1/?lat={}&lon={}&altitude={}&n={}",
        latitude, longitude, altitude, n
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
                                // convert response into Vec<Spot>
                                let w: api::Response = w;
                                for r in w.response {
                                    result.push(Spot {
                                        duration: Duration::seconds(r.duration as i64),
                                        risetime: from_utc_timestamp(r.risetime),
                                    });
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
/// - `n`: number of spotting events to fetch (<=100)
/// #### Return value
/// - ⇒ `Ok(Vec<Spot>)`: vector of upcoming spotting events
///     (see also [*open-notify* documentation](https://open-notify-api.readthedocs.io/en/latest/iss_pass.html) for details)
/// - ⇒ `Err(String)`: Error message about any occured http or json issue
///         - e.g. `500 Internal Server Error"
///         - some json parser error message if response from open-notify.org could not be parsed
pub async fn spot(latitude: f64, longitude: f64, altitude: f64, n: u8) -> Result<Vec<Spot>, String> {
    let r = init(latitude, longitude, altitude, n, 0);
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
    /// - `n`: number of spotting events to fetch (<=100)
    /// #### Return value
    /// - ⇒ `Ok(Vec<Spot>)`: vector of upcoming spotting events
    /// - ⇒ `Err(String)`: Error message about any occured http or json issue
    ///         - e.g. `500 Internal Server Error"
    ///         - some json parser error message if response from open-notify.org could not be parsed
    pub fn spot(latitude: f64, longitude: f64, altitude: f64, n: u8) -> Result<Vec<Spot>, String> {
        // wait for result
        executor::block_on(super::spot(latitude, longitude, altitude, n))
    }
}
