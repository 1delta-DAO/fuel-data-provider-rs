use warp::Filter;
use crate::api::rest::endpoint::{get_status, get_tokens, get_tokens_by_address, get_tokens_by_time_range, get_top_gainers, get_top_losers, get_top_volume, CountQueryParams, QueryParams};

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
        .and(
            get_top_gainers
                .or(get_top_losers)
                .or(get_top_volume)
                .or(get_tokens_by_time)
                .or(get_tokens_by_address)
                .or(tokens_route)
                .or(status_route),
        )
        .boxed()
}
