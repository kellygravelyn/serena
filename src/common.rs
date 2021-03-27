use hyper::{Body, Response, Result};

pub fn not_found() -> Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(404)
            .body(Body::from(""))
            .unwrap()
    )
}
