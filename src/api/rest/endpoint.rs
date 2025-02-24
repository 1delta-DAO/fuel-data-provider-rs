use std::convert::Infallible;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::entity::TokenEntity;
use crate::domain::service::persistence::TokenService;

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
                warp::http::StatusCode::from_u16(500).unwrap(),
            ))
        }
    }
}