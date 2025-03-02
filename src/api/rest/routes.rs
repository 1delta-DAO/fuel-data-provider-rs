use warp::Filter;
use crate::api::rest::endpoint::{get_status, get_tokens, get_tokens_by_time_range, QueryParams};

pub fn routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let tokens_route = warp::path("tokens")
        .and(warp::get())
        .and_then(get_tokens);
    let get_tokens_by_time = warp::path("tokens")
        .and(warp::path("by-time"))
        .and(warp::get())
        .and(warp::query::<QueryParams>())
        .and_then(get_tokens_by_time_range);
    let status_route = warp::path("status")
        .and(warp::get())
        .and_then(get_status);
    tokens_route.or(get_tokens_by_time).or(status_route).boxed()
}
