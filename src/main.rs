use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response, RequestExt};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

// This is the main body for the function.
// Write your code inside it.

#[derive(Serialize, Clone, Debug)]
struct Event {
    date: String,
    headliner: String,
    url: String,
    support_talent: String,
    showtime: String,
    venue: String,
    age: String,
}

fn get_string_from_selector(selector: String, document: &ElementRef) -> String {
    let selector = Selector::parse(&selector).unwrap();
    let option = document
        .select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned());
    option.unwrap_or_else(|| "".into())
}

fn get_string_from_attr(selector: String, document: &ElementRef, attribute: String) -> String {
    let selector = Selector::parse(&selector).unwrap();
    let option = document.select(&selector).next().unwrap().value().attr(&attribute);
    option.unwrap_or_else(|| "".into()).to_owned()
}

async fn get_corazon_events() -> Result<Response<Body>, Error> {
    let response = reqwest::get("https://elcorazonseattle.com/")
        .await?
        .text()
        .await?;

    // grap one selector to get how many events are there
    let document = Html::parse_document(&response);
    let event_content = Selector::parse("div.seetickets-list-event-container").unwrap();
    let mut event_vec: Vec<Event> = Vec::new();

    for event in document.select(&event_content) {
        let date = get_string_from_selector("p.event-date".into(), &event);
        let headliner = get_string_from_selector("p.headliners".into(), &event);
        let url = get_string_from_attr("p.event-title > a".into(), &event, "href".into());
        let support_talent = get_string_from_selector("p.supporting-talent".into(), &event);
        let showtime = get_string_from_selector("p.doortime-showtime".into(), &event);
        let venue = get_string_from_selector("p.venue".into(), &event);
        let age = get_string_from_selector("span.ages".into(), &event);

        let new_event = Event {
            date,
            headliner,
            url,
            support_talent,
            showtime,
            venue,
            age,
        };
        event_vec.push(new_event);
    }


    let json_message = serde_json::to_string(&event_vec).unwrap();
    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(json_message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn get_cafe_racer_events() -> Result<Response<Body>, Error> {
    let response = reqwest::get("https://caferacer.wpengine.com/#/events")
        .await?
        .text()
        .await?;

    // grap one selector to get how many events are there
    let document = Html::parse_document(&response);
    let event_content = Selector::parse("div.event-card").unwrap();
    let mut event_vec: Vec<Event> = Vec::new();

    for event in document.select(&event_content) {
        let date = get_string_from_selector("div.date".into(), &event);
        let headliner = get_string_from_selector("div.event-name".into(), &event);
        // let url = get_string_from_attr("p.event-title > a".into(), &event, "href".into());
        // TODO: continue to search for ways to find the id of the modal and provide that url
        let url = "https://caferacer.wpengine.com/#/events".into();
        let support_talent = get_string_from_selector("div.support".into(), &event);
        let showtime = get_string_from_selector("span.time".into(), &event);
        let venue = "Cafe Racer".into();
        let age = get_string_from_selector("div.age-caption".into(), &event);

        let new_event = Event {
            date,
            headliner,
            url,
            support_talent,
            showtime,
            venue,
            age,
        };
        event_vec.push(new_event);
    }


    let json_message = serde_json::to_string(&event_vec).unwrap();
    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(json_message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

fn not_found() -> Result<Response<Body>, Error> {
    let resp = Response::builder()
        .status(400)
        .header("content-type", "application/json")
        .body("Resource Not Found".into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    let uri = _event.query_string_parameters();
    let integration = uri.first("integration").unwrap_or("");

    let resp = match integration {
        "corazon" => get_corazon_events().await?,
        "cafe_racer" => get_cafe_racer_events().await?,
        _ => not_found()?,
    };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
