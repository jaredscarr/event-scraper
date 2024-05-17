use std::ops::Add;
use scraper::{Html, Selector, ElementRef};
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
use scraper::html::Select;

// This is the main body for the function.
// Write your code inside it.

#[derive(Clone, Debug)]
struct Event {
    date: String,
    headliner: String,
    support_talent: String,
    showtime: String,
    venue: String,
    age: String,
}

fn get_string_from_selector(selector: String, document: &ElementRef) -> String {
    let selector = Selector::parse(&selector).unwrap();
    let option = document.select(&selector).next().map(|element|element.text().collect::<String>().trim().to_owned());
    option.unwrap_or_else(|| String::from(""))
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // TODO: create a handler that will scrape based on the date. If the date hasn't changed don't run again
    let response = reqwest::get(
        "https://elcorazonseattle.com/",
    )
        .await?
        .text()
        .await?;

    // grap one selector to get how many events are there
    let document = Html::parse_document(&response);
    let event_content = Selector::parse("div.event-info-block").unwrap();
    let mut event_vec: Vec<Event> = Vec::new();

    for event in document.select(&event_content) {
        let date = get_string_from_selector(String::from("p.event-date"), &event);
        let headliner = get_string_from_selector(String::from("p.headliners"), &event);
        let support_talent = get_string_from_selector(String::from("p.supporting-talent"), &event);
        let showtime = get_string_from_selector(String::from("p.doortime-showtime"), &event);
        let venue = get_string_from_selector(String::from("p.venue"), &event);
        let age = get_string_from_selector(String::from("span.ages"), &event);

        let new_event = Event {
            date,
            headliner,
            support_talent,
            showtime,
            venue,
            age,
        };
        event_vec.push(new_event);
    }
    println!("{event_vec:?}");

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
        .header("content-type", "text/html")// TODO: switch to json
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
