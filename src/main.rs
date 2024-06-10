use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response, RequestExt};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use futures::{stream, StreamExt};
use std::collections::HashMap;

// This is the main body for the function.
// Write your code inside it.

const PARALLEL_REQUESTS: usize = 2;

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

fn zero_pad_num_string(n: String) -> String {
    return match n.len() {
        1 => format!("{}{}", "0", n),
        _ => n,
    }
}

async fn get_corazon_events(month_lookup: HashMap<&str, &str>) -> Result<Response<Body>, Error> {
    let response = reqwest::get("https://elcorazonseattle.com/")
        .await?
        .text()
        .await?;

    // grap one selector to get how many events are there
    let document = Html::parse_document(&response);
    let event_content = Selector::parse("div.seetickets-list-event-container").unwrap();
    let mut event_vec: Vec<Event> = Vec::new();

    for event in document.select(&event_content) {
        let mut date = get_string_from_selector("p.event-date".into(), &event);
        let date_vec = date.trim().split(" ").collect::<Vec<_>>();
        // TODO: temp hack to insert the year. Will need to revisit soon to implement hitting each event page like showbox
        // and grabbing the year month and day there.
        let year: String = "2024".into();
        let month = date_vec[1].to_lowercase();
        let month_num = month_lookup.get(&month as &str).unwrap_or(&"");
        let day: String = zero_pad_num_string(date_vec[2].into());
        date = format!("{year}-{month_num}-{day}");
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

async fn get_barboza_events(month_lookup: HashMap<&str, &str>) -> Result<Response<Body>, Error> {
    let response = reqwest::get("https://www.thebarboza.com/events")
        .await?
        .text()
        .await?;
    // grap one selector to get how many events are there
    let document = Html::parse_document(&response);
    let event_content = Selector::parse("div.eventItem").unwrap();
    let mut event_vec: Vec<Event> = Vec::new();

    for event in document.select(&event_content) {
        // let date = get_string_from_selector("div.date".into(), &event);
        let mut date = get_string_from_attr("div.date".into(), &event, "aria-label".into());
        let date_vec = date.split(" ").collect::<Vec<_>>();
        let year: String = date_vec[2].trim().into();
        let month: String = date_vec[0].to_lowercase();
        let month_num = month_lookup.get(&month as &str).unwrap_or(&"");
        let day: String = zero_pad_num_string(date_vec[1].into());
        date = format!("{year}-{month_num}-{day}");
        let headliner = get_string_from_selector("h3.title".into(), &event);
        let url = get_string_from_attr("h3.title > a".into(), &event, "href".into());
        let support_talent = get_string_from_selector("h4.tagline".into(), &event);
        let showtime = get_string_from_selector("div.time".into(), &event);
        let venue = "Barboza".into();
        let age = get_string_from_selector("div.age".into(), &event);

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

async fn get_showbox_events(month_lookup: HashMap<&str, &str>) -> Result<Response<Body>, Error> {
    // grap one selector to get how many events are there
    let mut event_vec: Vec<Event> = vec![];
    let client = reqwest::Client::new();
    // Block to drop the response and document at the end of the block
    {
        let response = client.get("https://www.showboxpresents.com/events/all")
            .send()
            .await?
            .text()
            .await?;
        let document = Html::parse_document(&response);
        let event_content = Selector::parse("div.entry").unwrap();
        // TODO: consider refactor into iterator pattern with take 30 instead of break
        for (i, el) in document.select(&event_content).enumerate() {
            if i == 30 {
                break;
            }
            let mut date = get_string_from_selector("span.date".into(), &el);
            let date_vec = date.split(",").collect::<Vec<_>>();
            let year: String = date_vec[2].trim().into();
            let month_and_day = date_vec[1].trim().split(" ").collect::<Vec<_>>();
            let month: String = month_and_day[0].to_lowercase();
            let month_num = month_lookup.get(&month as &str).unwrap_or(&"");
            let day: String = zero_pad_num_string(month_and_day[1].into());
            date = format!("{year}-{month_num}-{day}");
            let new_event = Event {
                date: date,
                headliner: "".into(),
                url: get_string_from_attr("div.thumb > a".into(), &el, "href".into()),
                support_talent: "".into(),
                showtime: get_string_from_selector("span.time".into(), &el).split('\t').last().unwrap_or_else(|| "").into(),
                venue: get_string_from_selector("span.venue".into(), &el),
                age: "".into(),
            };
            event_vec.push(new_event);
        }
    }

    let responses = stream::iter(event_vec.clone()).map(|event| {
        let client = client.clone();
        tokio::spawn(async move {
            let resp = client.get(event.url)
                .send().await
                .expect("Failed to send request")
                .text()
                .await;
            resp
        })
    })
        .buffer_unordered(PARALLEL_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    for i in 0..responses.len() {
        let res = &responses[i];
        let event = &mut event_vec[i];
        match res {
            Ok(Ok(res)) => {
                let document = Html::parse_document(&res);
                let event_content = Selector::parse("div.event_detail").unwrap();
                for el in document.select(&event_content) {
                    event.headliner = get_string_from_selector("div.page_header_left > h1".into(), &el);
                    event.support_talent = get_string_from_selector("div.page_header_left > h4".into(), &el);
                    event.age = get_string_from_selector("div.age_res".into(), &el);
                }
            }
            Ok(Err(e)) => println!("Got a reqwest::Error: {}", e),
            Err(e) => println!("Got a tokio::JoinError: {}", e),
        }
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
    let month_lookup = HashMap::from([
        ("jan", "01"),
        ("january", "01"),
        ("feb", "02"),
        ("february", "02"),
        ("mar", "03"),
        ("march", "03"),
        ("apr", "04"),
        ("april", "04"),
        ("may", "05"),
        ("jun", "06"),
        ("june", "06"),
        ("jul", "07"),
        ("july", "07"),
        ("aug", "08"),
        ("august", "08"),
        ("sep", "09"),
        ("sept", "09"),
        ("september", "09"),
        ("oct", "10"),
        ("october", "10"),
        ("nov", "11"),
        ("november", "11"),
        ("dec", "12"),
        ("december", "12"),
    ]);

    let uri = _event.query_string_parameters();
    let integration = uri.first("integration").unwrap_or("");

    let resp = match integration {
        "corazon" => get_corazon_events(month_lookup).await?,
        "barboza" => get_barboza_events(month_lookup).await?,
        "showbox" => get_showbox_events(month_lookup).await?,
        _ => not_found()?,
    };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
