# open_notify [![Rust](https://github.com/fightling/open_notify/actions/workflows/rust.yml/badge.svg)](https://github.com/fightling/open_notify/actions/workflows/rust.yml)

Fetch information about spotting International Space Station from [open-notify.org](http://open-notify.org).

# open_notify

...is a *rust crate* which lets you easily access current spotting information from [open-notify.org](https://open-notify.org/). This is an *unofficial* extension I have made to learn *rust* a little but I hope you have fun with it.

## Contents

<!-- MDTOC maxdepth:6 firsth1:2 numbering:0 flatten:0 bullets:1 updateOnSave:1 -->

- [Contents](#contents)   
- [How to use](#how-to-use)   
   - [Get continuous ISS updates](#get-continuous-iss-updates)   
      - [First: Start polling](#first-start-polling)   
      - [Then: Get ISS spotting updates](#then-get-iss-spotting-updates)   
         - [Nothing New: `None`](#nothing-new-none)   
         - [ISS spotting Update: `Vec<Spot>`](#iss-spotting-update-vecspot)   
         - [Some Error: `Err`](#some-error-err)   
   - [Get ISS spots just once](#get-iss-spots-just-once)   
- [Reference Documentation](#reference-documentation)   
- [Links](#links)   
   - [Website](#website)   
   - [*github* repository](#github-repository)   
   - [on *crates.io*](#on-cratesio)   
- [License](#license)   

<!-- /MDTOC -->

## How to use

First add this crate to your dependencies in you `Cargo.toml` file:

```toml
[dependencies]
open_notify = "0.1.7"
```

### Get continuous ISS updates

Then use the crate in your rust source file by calling `open_notify::init()` which returns a receiver object.
You can then use this receiver object to call `open_notify::update()` to get ISS spotting updates like in the following example:

```rust
extern crate open_notify;

use open_notify::{find_current, init, update};

fn main() {
    // start our observatory via OWM
    let receiver = &init(52.520008, 13.404954, 0.0, 90);
    loop {
        match update(receiver) {
            Some(response) => match response {
                Ok(spots) => println!(
                    "ISS is {}",
                    match find_current(spots,None) {
                        Some(_s) => "visible",
                        None => "invisible",
                    }
                ),
                Err(e) => println!("Could not fetch ISS spotting info because: {}", e),
            },
            None => (),
        }
    }
}
```

#### First: Start polling

`init()` spawns a thread which then will periodically poll *api.open-notify.org* for the current ISS position.
You then can use `update()` to ask for it.

#### Then: Get ISS spotting updates

There are three possible kinds of result you get from `update()` which you will have to face:

##### Nothing New: `None`

`update()` returns `None` if there is currently no new update available.
Which means: **You wont get any update twice!**
In other words: `update()` is not caching the last update for you.

##### ISS spotting Update: `Vec<Spot>`

If a new update was downloaded by the polling thread `update()` returns some `Vec<Spot>` object.
`Vec<Spot>` includes a list of spotting events.

##### Some Error: `Err`
On error `update()` returns some `String` object which includes a brief error description.

Errors may occur...
- initially while **there is no update yet** you will get an `Err` which includes exactly the String `"loading..."` (predefined in `open_notify::LOADING`).
- if a **server error** response was received (e.g. `500 Internal Server Error` if an **invalid API key** was used).
- on **json errors** while parsing the response from *api.open_notify.org*.

### Get ISS spots just once

If you just need the current ISS spotting events just once you may use the method `spot()` which envelopes `init()` and `update()` into one single synchronous or asynchronous call.
After the first successful spotting update the spawned thread will stop immediately and you get the result in return.

```rust
extern crate open_notify;
use open_notify::blocking::spot;

fn main() {
    // start our observatory via OWM
    match &spot(52.520008, 13.404954, 0.0) {
        Ok(spots) => println!(
            "ISS is {}",
            match find_current(spots,None) {
                Some(_s) => "visible",
                None => "invisible",
            }
        ),
        Err(e) => println!("Could not fetch ISS spotting info because: {}", e),
    }
}
```

There is a *blocking* and a *non-blocking* variant of `spot()`:

- The above example uses the synchronous (*blocking*) variant `open_notify::blocking::spot` which wont return until there is a new update.
- If you like to deal with the returned *future* by yourself just use `open_notify::spot` and asynchronously await the result until there is any.

## Reference Documentation

Beside this introduction there is a reference documentation which can be found [here](https://docs.rs/open_notify).

## Links

### Website

This README tastes better at [open_notify.thats-software.com](https://open_notify.thats-software.com).

### *github* repository

For the source code see [this repository](https://github.com/fightling/open_notify) at *github.com*.

### on *crates.io*

Published at [*crates.io*](https://crates.io/crates/open_notify).

## License

open_notify is licensed under the *MIT license* (LICENSE-MIT or http://opensource.org/licenses/MIT)
