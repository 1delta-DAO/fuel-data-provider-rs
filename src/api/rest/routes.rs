use warp::Filter;
use crate::api::rest::endpoint::get_tokens;

pub fn routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    warp::path("tokens")
        .and(warp::get())
        .and_then(get_tokens)
        .boxed()
}