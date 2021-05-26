// Note this useful idiom: importing names from outer (for mod tests) scope.
use super::*;

#[test]
fn test_spot() {
    match blocking::spot(52.520008, 13.404954, 0.0) {
        Ok(spots) => {
            // count upcoming spots
            let upcoming = find_upcoming(&spots,&None);
            assert!(upcoming.is_some());
            let upcoming = upcoming.unwrap();
            println!("{}", upcoming.risetime );
            match find_current(&spots,&None) {
                Some(current) => {
                    println!("{}", current.risetime );
                    assert!(upcoming.risetime != current.risetime);
                }
                _ => ()
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            assert!(false);
        }
    }
}
