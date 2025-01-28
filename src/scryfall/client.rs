use std::{num::NonZero, str::FromStr, time::Duration};

use ratelimit_meter::{NonConformance, GCRA};
use reqwest::{header::{ACCEPT, USER_AGENT}, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{game, scryfall::types, PROJECT_NAME};

const SCRYFALL_API_ENDPOINT: &'static str = "https://api.scryfall.com";

type HttpClient = reqwest::blocking::Client;
type HttpResponse = reqwest::blocking::Response;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

fn build_http_client() -> HttpClient {
    HttpClient::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .expect("to be able to build http client")
}

type RateLimiter = ratelimit_meter::DirectRateLimiter::<GCRA>;

fn build_rate_limiter() -> RateLimiter {
    // Rate limit the requests made to 10 per 1 second,
    // as described here: https://scryfall.com/docs/api.
    // We will aim for 5 per second to be safe
    let num_tokens = NonZero::new(5).expect("num tokens must be nonzero");
    let time = Duration::from_secs(1);
    RateLimiter::new(num_tokens, time)
}

const RATE_LIMIT_MAX_RETRIES: u32 = 10;

trait RateLimiterExt {
    fn acquire(&mut self) -> Result<(), ScryfallError>; // block until can be acquired
}
impl RateLimiterExt for RateLimiter {
    fn acquire(&mut self) -> Result<(), ScryfallError> {
        let begin = std::time::Instant::now();
        for _ in 0..RATE_LIMIT_MAX_RETRIES {
            
            let Err(negative_decision) = self.check() else {
                return Ok(()) // rate limit successfully acquired
            };
            let now = std::time::Instant::now();
            let wait = negative_decision.wait_time_from(now);
            std::thread::sleep(wait);
        }

        let end = std::time::Instant::now();
        let time_elapsed = end - begin;

        Err(ScryfallError::MaxRetries {
            times_tried: RATE_LIMIT_MAX_RETRIES,
            time_elapsed,
        })
    }
}

pub struct ScryfallClient {
    endpoint: String,
    http_client: HttpClient,
    rate_limiter: RateLimiter
}

impl ScryfallClient {
    pub fn new() -> Self {
        Self {
            endpoint: SCRYFALL_API_ENDPOINT.to_string(),
            http_client: build_http_client(),
            rate_limiter: build_rate_limiter(),
        }
    }
    fn make_request<Q: Serialize, T: DeserializeOwned>(&mut self, url: String, query: Q) -> Result<T, ScryfallError> {
        // must acquire the rate limit
        self.rate_limiter.acquire()?;
        // user agent and accept headers are required:
        //  https://scryfall.com/docs/api
        let request = self.http_client.get(url)
            .header(USER_AGENT, PROJECT_NAME)
            .header(ACCEPT, "application/json")
            .query(&query);

        log::trace!("about to make request: {request:?}");
        let response = request.send()?;
        log::trace!("received response: {response:?}");

        let response = ScryfallError::raise_on_error(response)?;
        let text = response.text()?;
        let output = serde_json::from_str(text.as_str())?;

        Ok(output)
    }
    pub fn card_named(&mut self, card_name: &str) -> Result<types::CardData, ScryfallError> {
        let url = format!("{}/cards/named", self.endpoint);

        let data: types::CardData = self.make_request(
            url,
            [("exact", card_name)]
        )?;

        Ok(data)
    }
}

#[derive(Debug)]
pub enum ScryfallError {
    MaxRetries {
        times_tried: u32,
        time_elapsed: Duration
    },
    HttpError(reqwest::Error),
    HttpErrorWithResponse {
        http_error: reqwest::Error,
        response: serde_json::Value
    },
    Deserialization(serde_json::Error)
}
impl ScryfallError {
    pub fn raise_on_error(response: HttpResponse) -> Result<HttpResponse, ScryfallError> {
        let http_error = match response.error_for_status_ref() {
            Ok(_) => return Ok(response),
            Err(e) => e
        };
        let text = match response.text() {
            Ok(t) => t,
            Err(decode_error) => {
                log::error!("could not decode response during error recovery: {decode_error}");
                // simply replace it with the original error
                return Err(http_error.into());
            }
        };
        let json_value : serde_json::Value = match serde_json::from_str(text.as_str()) {
            Ok(v) => v,
            Err(parse_error) => {
                log::error!("could not parse json during error recovery: {parse_error}");
                // again, simply replace with original error
                return Err(http_error.into());
            }
        };
        Err(ScryfallError::HttpErrorWithResponse { http_error, response: json_value })
    }
}

impl std::fmt::Display for ScryfallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScryfallError::MaxRetries { times_tried, time_elapsed } => write!(f, "hit maximum retries while calling api. tried: {times_tried} attempts to acquire rate limit, {time_elapsed:?} has elapsed since the call began"),
            ScryfallError::HttpError(e) => e.fmt(f),
            ScryfallError::HttpErrorWithResponse { http_error, response } => write!(f, "{http_error}: {response}"),
            ScryfallError::Deserialization(e) => e.fmt(f)
        }
    }
}
impl std::error::Error for ScryfallError { }

impl From<reqwest::Error> for ScryfallError {
    fn from(e: reqwest::Error) -> Self {
        ScryfallError::HttpError(e)
    }
}
impl From<serde_json::Error> for ScryfallError {
    fn from(e: serde_json::Error) -> Self {
        ScryfallError::Deserialization(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut client = ScryfallClient::new();

        let resp = client.card_named("Lightning Bolt").expect("didn't expect any errors");
        assert_eq!(resp.name.as_str(), "Lightning Bolt");
        assert_eq!(resp.type_line.as_str(), "Instant");
    }

    #[test]
    fn test_rate_limiter() {
        let mut rl = build_rate_limiter();

        let begin = std::time::Instant::now();

        let count = 20;
        eprintln!("beginning rate limit test");
        for i in 0..count {
            eprintln!("starting to acquire, {i} / {count}");
            rl.acquire().expect("failed to acquire rate limiter");
        }
        eprintln!("done");

        let end = std::time::Instant::now();

        let time_elapsed = end - begin;
        let min_allowable = Duration::from_secs(3); // TODO: fails, but within an acceptable margin

        eprintln!("time_elapsed: {time_elapsed:?}");
        eprintln!("min allowable: {min_allowable:?}");

        assert!(min_allowable < time_elapsed); 
    }
}
