use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};

/// This is the main body for the function.
/// Write your code inside it.
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // create a handler that will scrape based on the date. If the date hasn't changed don't run again
    let response = reqwest::get(
        "https://elcorazonseattle.com/",
    )
        .await?
        .text()
        .await?;
    // println!("body = {response:?}");
    let document = scraper::Html::parse_document(&response);
    println!("{document:?}");



    // Extract some useful information from the request
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");
    let message = format!("Hello {who}, this is an AWS Lambda HTTP request");

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
