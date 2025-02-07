use std::{num::NonZero, time::Duration};

use itertools::Itertools;
use ratelimit_meter::{NonConformance, GCRA};
use reqwest::{blocking::RequestBuilder, header::{ACCEPT, USER_AGENT}};
use serde::de::DeserializeOwned;

use crate::{game::CardSource, scryfall::{error::ScryfallError, types}, PROJECT_NAME};

use super::{convert, types::CardCollectionRequest};

const SCRYFALL_API_ENDPOINT: &'static str = "https://api.scryfall.com";

type HttpClient = reqwest::blocking::Client;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

const MAX_CARDS_PER_COLLECTION_REQUEST: usize = 75;

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

    /// Helper method to wrap http requests to scryfall apis
    fn make_request<T: DeserializeOwned, F: FnOnce(&mut HttpClient) -> RequestBuilder>(&mut self, builder: F) -> Result<T, ScryfallError> {
        // must acquire the rate limit
        self.rate_limiter.acquire()?;
        // user agent and accept headers are required:
        //  https://scryfall.com/docs/api
        let request = builder(&mut self.http_client)
            .header(USER_AGENT, PROJECT_NAME)
            .header(ACCEPT, "application/json");

        log::debug!("about to make request: {request:#?}");
        let response = request.send()?;
        log::debug!("received response: {response:#?}");

        let response = ScryfallError::raise_on_error(response)?;
        let text = response.text()?;
        let output = serde_json::from_str(text.as_str())?;

        Ok(output)
    }

    /// Make an API request to https://scryfall.com/docs/api/cards/named
    pub fn get_card_named(&mut self, card_name: &str) -> Result<types::CardCollectionResponse, ScryfallError> {
        let url = format!("{}/cards/named", self.endpoint);

        let data: types::CardCollectionResponse = self.make_request(
            |http| http.get(url).query(&[("exact", card_name)])
        )?;

        Ok(data)
    }

    /// Make an API request to https://scryfall.com/docs/api/cards/collection
    pub fn get_card_collection<'a, I: IntoIterator<Item=&'a str>>(&mut self, card_names: I) -> Result<types::CardCollectionResponse, ScryfallError> {
        use types::CardIdentifier;
        use types::CardCollectionResponse;
        use types::CardCollectionRequest;

        let url = format!("{}/cards/collection", self.endpoint);

        let mut data = CardCollectionResponse::empty();
        let chunks = card_names.into_iter().chunks(MAX_CARDS_PER_COLLECTION_REQUEST);
        for chunk in chunks.into_iter() {
            let identifiers = chunk
                .into_iter()
                .map(|name| CardIdentifier::name(name))
                .collect_vec();
                
            let request_body = CardCollectionRequest {
                identifiers
            };
     
           let resp: types::CardCollectionResponse = self.make_request(
                |http| http.post(&url).json(&request_body)
           )?;

           data.extend(resp);
        };


        for not_found in data.not_found.iter() {
            log::warn!("scryfall could not find this card identifier: {not_found:?}")
        }

        Ok(data)
    }
}

impl CardSource for ScryfallClient {
    fn get_cards_in_bulk(&mut self, card_names: Vec<String>) -> Result<Vec<crate::game::CardData>, Box<dyn std::error::Error>> {
        let input = card_names
            .iter()
            .map(String::as_str);
        let output = self.get_card_collection(input)?;
        let card_data = output.data
            .into_iter()
            .map(convert::convert_card)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(card_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn smoke_test_card_named() {
        let mut client = ScryfallClient::new();

        let resp = client.get_card_named("Lightning Bolt").expect("didn't expect any errors");
        assert_eq!(resp.name.as_str(), "Lightning Bolt");
        assert_eq!(resp.type_line.as_str(), "Instant");
    }

    #[test]
    fn smoke_test_list_cards() {
        let mut client = ScryfallClient::new();

        let resp = client.get_card_collection(["Ancient Tomb", "Lightning Bolt"]).expect("no errors");

        let contains_ancient_tomb = resp.data.iter().any(|card| card.name == "Ancient Tomb");
        let contains_lighhtning_bolt = resp.data.iter().any(|card| card.name == "Lightning Bolt");

        assert!(contains_ancient_tomb);
        assert!(contains_lighhtning_bolt);
        assert_eq!(resp.data.len(), 2);
    }
    */

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
