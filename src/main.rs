use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;
use input_method::input;
use std::process::Command;

use reqwest::blocking::Client;
use serde::Serialize;

use rusqlite::Connection;

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatPayload<'a> {
    model: &'static str,
    messages: [Message<'a>; 2],
    temperature: f32,
    max_tokens: i32,
    stream: bool,
}

pub fn llm_response(client: &Client, input: &str) -> Result<String, reqwest::Error> {
    let payload = ChatPayload {
        model: "google/gemma-4-e2b",
        messages: [
            Message {
                role: "system",
                content: "Respond",
            },
            Message {
                role: "system",
                content: input,
            },
        ],
        temperature: 0.0,
        max_tokens: -1,
        stream: false,
    };

    let response = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&payload) // Serializes directly into the request body buffer
        .send()?;

    response.text()
}

fn main() {
    println!("Please enter your prompt for the task to research:");
    let scout_task = input();

    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();

    tab.navigate_to("https://duckduckgo.com").unwrap();
    tab.wait_until_navigated().unwrap();

    // scout loop
    loop {
        let jpeg_data = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
            .unwrap();
        std::fs::write("screenshot.jpeg", jpeg_data).unwrap();
    }

    /*
    // Wait for network/javascript/dom to make the search-box available
    // and click it.
    tab.wait_for_element("input#searchInput")
        .unwrap()
        .click()
        .unwrap();

    // Type in a query and press `Enter`
    tab.type_str("WebKit").unwrap().press_key("Enter").unwrap();

    // We should end up on the WebKit-page once navigated
    let elem = tab.wait_for_element("#firstHeading").unwrap();
    assert!(tab.get_url().ends_with("WebKit"));

    /*
    // Take a screenshot of the entire browser window
    let jpeg_data = tab
        .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
        .unwrap();
    // Save the screenshot to disc
    std::fs::write("screenshot.jpeg", jpeg_data).unwrap();
    */

    /*
    // Take a screenshot of just the WebKit-Infobox
    let png_data = tab
        .wait_for_element("#mw-content-text > div > table.infobox.vevent")
        .unwrap()
        .capture_screenshot(Page::CaptureScreenshotFormatOption::Png)
        .unwrap();
    // Save the screenshot to disc
    std::fs::write("screenshot.png", png_data).unwrap();
    */

    // Run JavaScript in the page
    let remote_object = elem
        .call_js_fn(
            r#"
       function getIdTwice () {
           // `this` is always the element that you called `call_js_fn` on
           const id = this.id;
           return id + id;
       }
   "#,
            vec![],
            false,
        )
        .unwrap();
    match remote_object.value {
        Some(returned_string) => {
            dbg!(&returned_string);
            assert_eq!(returned_string, "firstHeadingfirstHeading".to_string());
        }
        _ => unreachable!(),
    }; */

    //println!("{}", llm_response(&Client::new(), "Hello").unwrap());
}
