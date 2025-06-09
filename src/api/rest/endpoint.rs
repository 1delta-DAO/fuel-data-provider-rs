use crate::domain::service::persistence::{PriceDataService, SyncStatusService, TokenService};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::collections::HashSet;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

/*
`getNewTokens(start:number, end:number)`

- return tokens created within the given timeframe

`getPricesAndVolume(assets:address[])`

- returns current price and 24h price

`getTrending(count: number)`

- get trending assets, for now, just 3 groups, with `count`  amount of assets in each group
    - Top gainers
    - Top losers
    - Top volume
 */

pub async fn get_status() -> Result<impl Reply, Infallible> {
    match SyncStatusService::get_status().await {
        Ok(Some(sync_status)) => {
            Ok(warp::reply::with_status(warp::reply::json(&sync_status), StatusCode::OK))
        }
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::json(&"No sync status found"),
            StatusCode::NOT_FOUND,
        )),
        Err(err) => {
            log::error!("Error fetching sync status: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_tokens() -> Result<impl warp::Reply, Infallible> {
    match TokenService::find_all_tokens().await {
        Ok(tokens) => Ok(warp::reply::with_status(
            warp::reply::json(&tokens),
            warp::http::StatusCode::from_u16(200).unwrap(),
        )),
        Err(err) => {
            log::error!("Database error: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                warp::http::StatusCode::OK,
            ))
        }
    }
}

pub async fn get_tokens_by_time_range(params: QueryParams) -> Result<impl Reply, Rejection> {
    match (parse_datetime(&params.start), parse_datetime(&params.end)) {
        (Some(start), Some(end)) => match TokenService::find_by_created_between(start, end).await {
            Ok(tokens) => {
                Ok(warp::reply::with_status(warp::reply::json(&tokens), warp::http::StatusCode::OK))
            }
            Err(err) => {
                log::error!("Failed to fetch tokens by time range: {:?}", err);
                Ok(warp::reply::with_status(
                    warp::reply::json(&"Internal server error"),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        _ => Ok(warp::reply::with_status(
            warp::reply::json(&"Invalid date format"),
            warp::http::StatusCode::BAD_REQUEST,
        )),
    }
}

pub async fn get_tokens_by_address(params: AddressQueryParams) -> Result<impl Reply, Infallible> {
    // Ensure there are addresses provided
    if params.addresses.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Address list cannot be empty"),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Filter out empty or invalid addresses
    let addresses: HashSet<String> =
        params.addresses.into_iter().filter(|addr| !addr.trim().is_empty()).collect();

    if addresses.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"All provided addresses were empty or invalid"),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Query the database for the valid addresses
    match TokenService::find_by_addresses(addresses.into_iter().collect()).await {
        Ok(tokens) => Ok(warp::reply::with_status(warp::reply::json(&tokens), StatusCode::OK)),
        Err(err) => {
            log::error!("Error fetching tokens by addresses: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

//Trending assets

pub async fn get_top_gainers(params: CountQueryParams) -> Result<impl Reply, Infallible> {
    match TokenService::find_biggest_gainers().await {
        Ok(tokens) => {
            let limited_tokens = tokens.into_iter().take(params.count).collect::<Vec<_>>();
            Ok(warp::reply::with_status(warp::reply::json(&limited_tokens), StatusCode::OK))
        }
        Err(err) => {
            log::error!("Error fetching top gainers: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_top_losers(params: CountQueryParams) -> Result<impl Reply, Infallible> {
    match TokenService::find_biggest_losers().await {
        Ok(tokens) => {
            let limited_tokens = tokens.into_iter().take(params.count).collect::<Vec<_>>();
            Ok(warp::reply::with_status(warp::reply::json(&limited_tokens), StatusCode::OK))
        }
        Err(err) => {
            log::error!("Error fetching top losers: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_top_volume(params: CountQueryParams) -> Result<impl Reply, Infallible> {
    match TokenService::find_highest_volume().await {
        Ok(tokens) => {
            let limited_tokens = tokens.into_iter().take(params.count).collect::<Vec<_>>();
            Ok(warp::reply::with_status(warp::reply::json(&limited_tokens), StatusCode::OK))
        }
        Err(err) => {
            log::error!("Error fetching top volume tokens: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn get_token_prices(params: TokenAddressParams) -> Result<impl Reply, Infallible> {
    // First get the token by address
    match TokenService::find_by_address(&params.address).await {
        Ok(Some(token)) => {
            // Then get all price records for this token
            match PriceDataService::find_all_by_token_id(&token.id).await {
                Ok(prices) => {
                    // Mapuj PriceDataEntity na PriceDataDto
                    let price_dtos: Vec<PriceDataDto> = prices
                        .into_iter()
                        .map(|p| PriceDataDto { price: p.price, timestamp: p.timestamp })
                        .collect();

                    Ok(warp::reply::with_status(warp::reply::json(&price_dtos), StatusCode::OK))
                }
                Err(err) => {
                    log::error!("Error fetching token prices: {:?}", err);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&"Internal server error"),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::json(&"Token not found"),
            StatusCode::NOT_FOUND,
        )),
        Err(err) => {
            log::error!("Error fetching token: {:?}", err);
            Ok(warp::reply::with_status(
                warp::reply::json(&"Internal server error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct QueryParams {
    start: String,
    end: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AddressQueryParams {
    addresses: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CountQueryParams {
    count: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct TokenAddressParams {
    address: String,
}

#[derive(serde::Serialize)]
struct PriceDataDto {
    price: f64,
    timestamp: DateTime<Utc>,
}

fn parse_datetime(datetime_str: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .and_then(|ndt| Utc.from_local_datetime(&ndt).single())
}
