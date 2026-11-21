//! Command interface shared by the desktop shell and the development server:
//! a command name and a JSON payload in, JSON out.

use crate::session::{AdvanceRequest, ApiError, Delegate, NewMatch, Session, SubRequest};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

/// Holds the match being played.
#[derive(Default)]
pub struct App {
    session: Option<Session>,
}

fn parse<T: DeserializeOwned>(payload: Value) -> Result<T, ApiError> {
    // An absent payload means "all defaults".
    let payload = if payload.is_null() { json!({}) } else { payload };
    serde_json::from_value(payload).map_err(|e| ApiError::new("bad_request", e.to_string()))
}

fn out<T: serde::Serialize>(value: &T) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|e| ApiError::new("internal", e.to_string()))
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    fn session(&mut self) -> Result<&mut Session, ApiError> {
        self.session.as_mut().ok_or_else(|| ApiError::new("no_match", "no match has been started"))
    }

    /// Run a command:
    ///
    /// * `new_match` (`NewMatch`) -> `{ view, state }`
    /// * `view` -> `MatchView`, `state` -> `MatchState`
    /// * `advance` (`AdvanceRequest`) -> `AdvanceView`
    /// * `substitute` (`SubRequest`) -> `MatchState`
    /// * `delegate` (`Delegate`) -> `MatchState`
    pub fn call(&mut self, cmd: &str, payload: Value) -> Result<Value, ApiError> {
        match cmd {
            "new_match" => {
                let cfg: NewMatch = parse(payload)?;
                let session = Session::new(&cfg);
                let result = json!({ "view": session.view(), "state": session.state() });
                self.session = Some(session);
                Ok(result)
            }
            "view" => out(self.session()?.view()),
            "state" => out(&self.session()?.state()),
            "advance" => {
                let req: AdvanceRequest = parse(payload)?;
                out(&self.session()?.advance(&req))
            }
            "substitute" => {
                let req: SubRequest = parse(payload)?;
                out(&self.session()?.substitute(&req)?)
            }
            "delegate" => {
                let d: Delegate = parse(payload)?;
                out(&self.session()?.set_delegate(d))
            }
            other => Err(ApiError::new("unknown_command", format!("unknown command '{other}'"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_round_trip_through_json() {
        let mut app = App::new();
        assert_eq!(app.call("state", Value::Null).unwrap_err().code, "no_match");
        let started = app.call("new_match", json!({ "seed": 9, "rules": "kbo", "control": "away" })).unwrap();
        assert_eq!(started["view"]["user_team"], 0);
        assert_eq!(started["view"]["rules"]["id"], "kbo");
        assert_eq!(started["view"]["teams"][0]["players"].as_array().unwrap().len(), 22);
        assert_eq!(started["state"]["situation"]["inning"], 1);
        // The away side bats first: tell the lead-off man to take, one pitch at a time.
        let step = app.call("advance", json!({ "mode": "pitch", "offense": { "approach": "take" } })).unwrap();
        let events = step["events"].as_array().unwrap();
        let pitch = events.iter().find(|e| e["type"] == "pitch").expect("a pitch was thrown");
        assert_eq!(pitch["approach"], "take");
        assert_eq!(pitch["swung"], false);
        assert!(pitch["speed_kmh"].as_f64().unwrap() > 100.0);
        // Unknown commands and malformed payloads are reported, not panicked on.
        assert_eq!(app.call("advance", json!({ "mode": "sideways" })).unwrap_err().code, "bad_request");
        assert_eq!(app.call("dance", Value::Null).unwrap_err().code, "unknown_command");
        let done = app.call("advance", json!({ "mode": "game" })).unwrap();
        assert_eq!(done["state"]["situation"]["over"], true);
        assert!(done["state"]["result"]["innings"].as_u64().unwrap() >= 9);
    }
}
