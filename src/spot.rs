use chrono;

pub type Duration = chrono::Duration;
pub type DateTime = chrono::DateTime<chrono::Local>;

pub fn from_utc_timestamp(t: i64) -> DateTime {
    let t = chrono::NaiveDateTime::from_timestamp(t, 0);
    let t: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(t, chrono::Utc);
    return chrono::DateTime::from(t);
}

pub struct DayTime {
    sunrise: DateTime,
    sunset: DateTime,
}

impl DayTime {
    pub fn from_utc(sunrise_utc: i64, sunset_utc: i64) -> DayTime {
        DayTime {
            sunrise: from_utc_timestamp(sunrise_utc),
            sunset: from_utc_timestamp(sunset_utc),
        }
    }
    pub fn at_night_utc(&self, datetime_utc: i64) -> bool {
        self.at_night(from_utc_timestamp(datetime_utc))
    }
    pub fn at_night(&self, datetime: DateTime) -> bool {
        datetime < self.sunrise || datetime > self.sunset
    }
}

#[derive(Copy, Clone)]
pub struct Spot {
    pub duration: Duration,
    pub risetime: DateTime,
}

impl Spot {
    pub fn is_spottable(&self) -> bool {
        chrono::Local::now() >= self.risetime
            && chrono::Local::now() < self.risetime + self.duration
    }
    pub fn at_night(&self, daytime: DayTime) -> bool {
        daytime.at_night(self.risetime)
    }
}

pub fn find_upcoming(spots: &Vec<Spot>, daytime: Option<DayTime>) -> Option<&Spot> {
    // count upcoming spots
    for spot in spots {
        if spot.risetime > chrono::Local::now() {
            match daytime {
                Some(dt) => {
                    if !spot.at_night(dt) {
                        return None;
                    }
                }
                _ => (),
            }
            return Some(spot);
        }
    }
    return None;
}

pub fn find_current(spots: &Vec<Spot>, daytime: Option<DayTime>) -> Option<&Spot> {
    // count upcoming spots
    for spot in spots {
        if spot.is_spottable() {
            match daytime {
                Some(dt) => {
                    if !spot.at_night(dt) {
                        return None;
                    }
                }
                _ => (),
            }
            return Some(spot);
        }
    }
    return None;
}
