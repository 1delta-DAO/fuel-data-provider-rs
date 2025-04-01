use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;
use warp::reject::Reject;
use crate::api::rest::endpoint::{get_status, get_tokens, get_tokens_by_address, get_tokens_by_time_range, get_top_gainers, get_top_losers, get_top_volume, CountQueryParams, QueryParams};
use crate::config::CONFIG;

pub fn routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let get_top_gainers = warp::path!("tokens" / "top-gainers")
        .and(warp::get())
        .and(warp::query::<CountQueryParams>())
        .and_then(get_top_gainers);

    let get_top_losers = warp::path!("tokens" / "top-losers")
        .and(warp::get())
        .and(warp::query::<CountQueryParams>())
        .and_then(get_top_losers);

    let get_top_volume = warp::path!("tokens" / "top-volume")
        .and(warp::get())
        .and(warp::query::<CountQueryParams>())
        .and_then(get_top_volume);

    let get_tokens_by_time = warp::path!("tokens" / "by-time")
        .and(warp::get())
        .and(warp::query::<QueryParams>())
        .and_then(get_tokens_by_time_range);

    let get_tokens_by_address = warp::path!("tokens" / "by-address")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(get_tokens_by_address);

    let tokens_route = warp::path!("tokens")
        .and(warp::get())
        .and_then(get_tokens);

    let status_route = warp::path!("status")
        .and(warp::get())
        .and_then(get_status);

    warp::any()
        .and(with_api_key())
        .and(
            get_top_gainers
                .or(get_top_losers)
                .or(get_top_volume)
                .or(get_tokens_by_time)
                .or(get_tokens_by_address)
                .or(tokens_route)
                .or(status_route),
        )
        .recover(handle_rejection)
        .boxed()
}

fn with_api_key() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::optional::<String>("x-api-key")
        .and_then(|key: Option<String>| async move {
            match key {
                Some(ref k) if k == &CONFIG.default.api_key => Ok(()),
                _ => Err(warp::reject::custom(Unauthorized)),
            }
        })
        .untuple_one()
}

#[derive(Debug)]
struct Unauthorized;

impl Reject for Unauthorized {}

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.find::<Unauthorized>().is_some() {
        Ok(warp::reply::with_status(
            "1Delta.IO - Backend: Unauthorized",
            StatusCode::UNAUTHORIZED,
        ))
    } else {
        Ok(warp::reply::with_status(
            "1Delta.IO - Backend: Internal Server Error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}