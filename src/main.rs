use base64::{Engine as _, engine::general_purpose};
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::protocol::cdp::Runtime::RemoteObject;
use input_method::input;
use reqwest::blocking::Client;
use serde::Serialize;
use std::default;
use std::time::Duration;

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

pub fn llm_response(
    client: &Client,
    input: &str,
    max_tokens: i32,
) -> Result<String, reqwest::Error> {
    let payload = ChatPayload {
        model: /*"google/gemma-4-e2b"*/ "frame2kg-smolvlm-500m-json",
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
        max_tokens: max_tokens,
        stream: false,
    };

    let response = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&payload)
        .send()?;

    response.text()
}

pub fn send_image_to_llm(
    client: &reqwest::blocking::Client,
    text_prompt: &str,
    base64_image: &str,
    max_tokens: i32,
) -> Result<String, reqwest::Error> {
    let payload = serde_json::json!({
        "model": "frame2kg-smolvlm-500m-json",
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": text_prompt },
                {
                    "type": "image_url",
                    "image_url": { "url": format!("data:image/jpeg;base64,{}", base64_image) }
                }
            ]
        }],
        "temperature": 0.0,
        "max_tokens": max_tokens,
    });

    let response = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&payload)
        .send()?;

    response.text()
}

const OBSERVER_PROMPT: &str = "
### ROLE
Visual State Encoder.

### TASK
Describe the screen for a deterministic browser agent. Focus ONLY on interactive elements and their current state.
";

const NEXTCONTEXTPROMPT: &str = "
### ROLE
Planning agent. Output the single next logical action string from the COMMAND SCHEMA. 

### COMMAND SCHEMA
TAB!, ENTER!, UPARROW!, DOWNARROW!, TYPING>...<ENDTYPING!, SEARCH>...<ENDTYPING!, DUCKDUCKGO!, END!

### GUIDELINES
- If the last action was TYPING or SEARCH and the UI element is focused, suggest ENTER!.
- If the target information is found, suggest END!.
- Do not output explanations. Only output the raw string.

### LAST AGENT ACTION:
";

const BROWSERPROMPT: &str = "
### ROLE
Deterministic Browser State Machine.
Goal: Reach target info via $RUN commands.
Constraint: OUTPUT ONLY THE COMMAND. NO MARKDOWN, NO EXPLANATIONS, NO JSON.

### COMMAND SCHEMA
$RUN TAB!
$RUN ENTER!
$RUN UPARROW!
$RUN DOWNARROW!
$RUN TYPING>input your text in here<ENDTYPING!
$RUN SEARCH>your text should be in here<ENDTYPING!
$RUN DUCKDUCKGO!
$RUN END!

### PROTOCOL
1. If no action required: Output 'WAITING'.
2. If action required: Output '$RUN [COMMAND]'.
3. Strict adherence to delimiters (>, <, !).
4. Do not prefix or suffix with non-schema text.
";

fn get_split(input: &str) -> Option<&str> {
    let (_, front) = input.rsplit_once('>')?;
    let (both, _) = front.rsplit_once('<')?;
    Some(both)
}

fn main() {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120)) // Increase to 2 minutes or more
        .build()
        .unwrap();

    /*
    println!("Please enter your prompt for the research task:");
    let scout_task = input();
    */
    let scout_task = "Find Hadrian Lazic's github page, and then find out what he programs";

    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();

    tab.navigate_to("https://duckduckgo.com").unwrap();
    tab.wait_until_navigated().unwrap();
    tab.enable_stealth_mode().unwrap();

    let mut next_context: String =
        "This is the first search, there is no plans yet, add plans after this response"
            .to_string();

    // scout loop
    loop {
        let mut focused_ele_string: String = "Nothing in focus".to_string();

        if let Ok(focused_element_info) = tab.evaluate(
            r#"
        (function() {
            const el = document.activeElement;
            return {
                tagName: el.tagName,
                id: el.id,
                name: el.getAttribute('name') || '',
                placeholder: el.getAttribute('placeholder') || '',
                innerText: el.innerText ? el.innerText.substring(0, 50) : '',
                ariaLabel: el.getAttribute('aria-label') || ''
            };
        })()
        "#,
            true,
        ) {
            focused_ele_string = format!("{:?}", focused_element_info);
        }

        let jpeg_data = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)
            .unwrap_or_default();

        std::fs::write("screenshot.jpeg", jpeg_data.clone()).unwrap_or_default(); // debugging

        let base64_encoded = general_purpose::STANDARD.encode(&jpeg_data);

        let response = send_image_to_llm(
            &client,
            &format!(
                "{}{} ### PLAN BASED OFF LAST ACTION {} ### METADATA OF FOCUSED ELEMENT {}",
                BROWSERPROMPT, scout_task, next_context, focused_ele_string
            ),
            &base64_encoded,
            300,
        )
        .unwrap();

        println!("{}", response); // debug

        let command_start = (response.rsplit_once('$'))
            .unwrap_or(("xxxxxxxxxxxx", "xxxxxxxxxxxxxx"))
            .1;
        unsafe {
            match command_start.get_unchecked(0..7) {
                "RUN TAB" => {
                    tab.press_key("Tab");
                }
                "RUN ENT" => {
                    tab.press_key("Enter");
                }
                "RUN UPA" => {
                    tab.press_key("ArrowUp");
                }
                "RUN DOW" => {
                    tab.press_key("ArrowDown");
                }
                "RUN TYP" => {
                    let both_split = get_split(command_start)
                        .unwrap_or("Failed to parse your text, from command sequence.");
                    tab.type_str(both_split);
                }
                "RUN SEA" => {
                    let both_split = get_split(command_start)
                        .unwrap_or("Failed to parse your text, from command sequence.");
                    tab.navigate_to(&format!("https://duckduckgo.com/{}", both_split));
                }
                "RUN DUC" => {
                    tab.navigate_to("https://duckduckgo.com");
                }
                "RUN END" => {
                    break; // TODO 
                }
                _ => {
                    println!("FAILED COMMAND:{}", command_start.get_unchecked(0..7))
                }
            }
        }

        next_context = llm_response(
            &client,
            &format!(
                "{}{} ### SEARCH TASK {}",
                NEXTCONTEXTPROMPT, response, scout_task
            ),
            100,
        )
        .unwrap();

        println!("{}", next_context); //debug
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
