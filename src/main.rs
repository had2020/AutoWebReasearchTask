use base64::{Engine as _, engine::general_purpose};
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::protocol::cdp::Runtime::RemoteObject;
use input_method::input;
use reqwest::blocking::Client;
use serde::Serialize;
use std::default;
use std::fmt::format;
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

const PLANPROMPT: &str = "
### ROLE
Planning agent. 
Analyze the current state against the global goal and determine the next logical step.

### PROTOCOL
1. [OBSERVATION]: Identify the primary blocker or missing data.
2. [STRATEGY]: Choose the most efficient sequence of UI interactions to resolve the blocker.
3. [NEXT_STEP]: Define a single, atomic action (e.g., 'submit login', 'click search button', 'navigate to X').

### TASK GOAL:
";

const ACTIONPROMPT: &str = "
### ROLE
Deterministic Browser State Machine.
GOAL: choose one of the following commands in the schema acording to the reasoning.
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

const MEMORYPROMPT: &str = "
### ROLE
State Compression Engineer.

### TASK
Condense the provided interaction history into a 'Persistent State Buffer'. 
Your goal is to maximize entropy per token while maintaining the causal trail for the browser agent.

### COMPRESSION PROTOCOL
1. **TRIM:** Remove all conversational pleasantries, redundant meta-commentary, and repetitive UI descriptions.
2. **RESOLVE:** Collapse sequential navigation steps into a single state change (e.g., instead of 'clicked search, typed X, clicked Y', output 'Searched for X, arrived at Y').
3. **PRESERVE:** 
   - Retain the absolute current goal.
   - Retain only the most recent 'successful' navigation path.
   - Explicitly note any 'blocked' states or encountered errors (crucial for future branching).
   - Retain key data extracted from previous pages (identifiers, URLs, specific values).

### OUTPUT FORMAT (Data-Oriented)
[CURRENT_GOAL]: ...
[LAST_SUCCESSFUL_STATE]: ...
[BLOCKED_BY]: (None or Error Type)
[KEY_DATA]: (Key-value pairs only)
[REMAINING_STEPS]: (Brief list of next likely actions)

### CONSTRAINTS
- Strict brevity: Target under 200 tokens.
- No prose. Use telegraphic, technical syntax.
- Maintain a strict causal link between past errors and the current path.
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

    let mut long_term_context: String = "This is your first State Compression".to_string();

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

        // decode visual
        let response = send_image_to_llm(
            &client,
            &format!(
                "{} ### METADATA OF FOCUSED ELEMENT {}",
                OBSERVER_PROMPT, focused_ele_string
            ),
            &base64_encoded,
            500,
        )
        .unwrap();

        // plan
        let plan = llm_response(
            &client,
            &format!(
                "{}{} ### Long Term Task History {} ### Current Visual {}",
                PLANPROMPT, scout_task, long_term_context, response
            ),
            500,
        )
        .unwrap();

        let action = llm_response(&client, &plan, 50).unwrap();

        let command_start = (action.rsplit_once('$'))
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

        // remember
        long_term_context = llm_response(
            &client,
            &format!(
                "Last State Compression: {}, Current plan used: {}, Current action done: {}",
                long_term_context, plan, action
            ),
            500,
        )
        .unwrap();
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
