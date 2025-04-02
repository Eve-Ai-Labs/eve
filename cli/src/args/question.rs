use crate::{
    display::DisplayAnswer,
    echoln,
    profiles::{Profiles, DEFAULT_SESSION_NAME},
    utils::{check_name, ProfileName},
    OUTPUT_JSON,
};
use clap::Parser;
use cli_utils::Prompt;
use color_eyre::eyre::{ensure, eyre, ContextCompat, Result};
use orchestrator_client::ClientWithKey;
use rustyline::DefaultEditor;
use std::io::{stdin, stdout, Write};
use termion::{
    clear,
    color::{Fg, Reset},
    cursor,
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
};
use tracing::{error, instrument};

/// Send a question and expect a response
#[derive(Debug, Parser)]
pub(crate) struct Question {
    /// Request to AI
    #[arg()]
    query: Option<String>,

    #[clap(flatten)]
    profile: ProfileName,

    /// Response waiting time in seconds
    #[arg(short, long)]
    waiting_time: Option<u64>,

    /// Session name(chat name). Used to store the history
    #[arg(short,long, value_parser = check_name, default_value = DEFAULT_SESSION_NAME)]
    session: String,

    /// Erase session history
    #[arg(short, long, visible_aliases = ["new", "erase"])]
    clean: bool,

    /// JSON output
    #[arg(short, long)]
    json: bool,

    #[command(flatten)]
    prompt: Prompt,
}

impl Question {
    #[instrument(level = "debug", skip_all)]
    pub(crate) async fn execute(&mut self) -> Result<()> {
        if self.json {
            OUTPUT_JSON.set(self.json)?;
            self.prompt.no_display = true;
        }

        let mut profiles = Profiles::load()?;

        let profile = profiles
            .get_mut(&self.profile)
            .with_context(|| eyre!("Profile {:?} not found", self.profile))?;
        let mut client: ClientWithKey = profile.client()?;

        // session
        let mut back_query = profile.session(&self.session);
        if back_query.is_some() && self.clean && self.prompt.prompt_yes("Start a new session?") {
            back_query = None;
        }

        let mut query = self.query.clone().unwrap_or_default();
        loop {
            if query.is_empty() {
                let result = ask();
                println!();
                match result {
                    Some(q) => {
                        query = q;
                    }
                    None => break,
                }
            }
            // balance
            let balance = client.balance().await?;
            let cost = client.cost(4_000).await?;
            ensure!(balance > cost, "There is not enough balance to perform the operation. Balance: {balance}. Maximum request cost: {cost}");

            if !self.prompt.prompt_yes(format!(
                "The maximum cost of a request is {cost}. Send a request?"
            )) {
                return Ok(());
            }

            // send
            echoln!("Sending a request...");

            if let Some(back_query) = back_query {
                client.history(back_query).await?;
            }

            let query_id = client.query(&query).await?;
            echoln!("The request has been sent. QueryID: {query_id}");

            profiles.set_and_save_session(&self.profile, &self.session, query_id)?;

            // wait
            echoln!("Waiting for a response...");

            let result = client.answer_wait(&query_id, None).await?;

            // display the response
            if self.json {
                let mut result = result.response;

                result.sort_by(|a, b| b.cmp(a));
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }

            let mut display: DisplayAnswer = result.into();
            display.draw()?;

            query.clear();
        }

        Ok(())
    }
}

fn ask() -> Option<String> {
    println!("Enter a request. To exit, press `ESC`");
    let prompt = "//> ";
    print!(
        "{prompt}\x1B[90mAsk the following question.{color_reset}\r{right}",
        color_reset = Fg(Reset),
        right = cursor::Right(4)
    );
    stdout().flush().unwrap();

    let mout = MouseTerminal::from(
        stdout()
            .into_raw_mode()
            .inspect_err(|err| error!("StdOut: {err}"))
            .ok()?,
    );
    let ev = stdin().keys().filter_map(|ev| ev.ok()).next();
    mout.suspend_raw_mode().unwrap();

    let mut input = String::new();
    match ev? {
        Key::Esc | Key::Char('\n') => {
            return None;
        }
        Key::Char(ch) => input.push(ch),
        _ => return None,
    };

    mout.suspend_raw_mode()
        .inspect_err(|err| error!("Switch to original mode: {err}"))
        .ok()?;

    print!("\r{}", clear::AfterCursor);
    stdout().flush().unwrap();

    let mut rl = DefaultEditor::new()
        .inspect_err(|err| error!("Create editor: {err}"))
        .ok()?;
    let input = rl
        .readline_with_initial(prompt, (input.as_str(), ""))
        .inspect_err(|err| error!("Read line: {err}"))
        .ok()?;

    let input = match input.trim() {
        "q" | "/q" => return None,
        v => v,
    };

    if input.is_empty() {
        return None;
    }

    Some(input.into())
}
