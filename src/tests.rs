// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;
use chrono::TimeZone;

#[test]
fn test_spot() {
    let now = chrono::Local::now();
    match blocking::spot(52.520008, 13.404954, 0.0, 100) {
        Ok(spots) => match find_upcoming(&spots, None, now) {
            Some(upcoming) => {
                println!("{}", upcoming.risetime);
                match find_current(&spots, None, now) {
                    Some(current) => {
                        println!("{}", current.risetime);
                        assert!(upcoming.risetime != current.risetime);
                    }
                    _ => (),
                }
            }
            _ => {
                eprintln!("no upcoming event");
                assert!(false);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            assert!(false);
        }
    }
}

fn time(t: &str) -> spot::DateTime {
    chrono::Local
        .datetime_from_str(t, "%d.%m.%Y %H:%M")
        .unwrap()
}

fn spots(a: Vec<&str>) -> Vec<Spot> {
    let mut spots: Vec<Spot> = Vec::new();
    for t in a {
        spots.push(Spot {
            duration: chrono::Duration::minutes(30),
            risetime: time(t),
        });
    }
    return spots;
}

fn daytime() -> DayTime {
    DayTime {
        sunrise: time("01.06.2021 07:00"),
        sunset: time("01.06.2021 21:00"),
    }
}

fn alltime() -> DayTime {
    DayTime {
        sunrise: time("01.06.2021 00:00"),
        sunset: time("01.06.2021 23:59"),
    }
}

#[test]
fn test_daytime_many() {
    let s = spots(
        [
            "01.06.2021 00:00",
            "01.06.2021 06:00",
            "01.06.2021 09:00",
            "01.06.2021 12:00",
            "01.06.2021 18:00",
            "01.06.2021 18:00",
            "01.06.2021 22:00",
            "02.06.2021 00:00",
        ]
        .to_vec(),
    );
    assert!(
        find_upcoming(&s, Some(&daytime()), time("01.06.2021 13:00")).is_some(),
    );
    assert!(
        find_upcoming(&s, Some(&daytime()), time("01.06.2021 23:00")).is_some(),
    );
    assert!(
        find_upcoming(&s, Some(&alltime()), time("01.06.2021 13:00")).is_none(),
    );
}

#[test]
fn test_daytime_one() {
    let s = spots(["01.06.2021 00:00"].to_vec());

    assert!(
        find_upcoming(&s, Some(&daytime()), time("31.05.2021 13:00")).is_some(),
    );
    assert!(
        find_upcoming(&s, Some(&daytime()), time("01.06.2021 13:00")).is_none(),
    );
}
