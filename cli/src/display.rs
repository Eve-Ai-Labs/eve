use color_eyre::eyre::Result;
use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
};
use termion::{
    color::{Bg, Black, Fg, Red, Reset, White},
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    style::{Reset as style_reset, Underline as style_underline},
};
use types::ai::{
    query::{NodeResult, Query},
    response::AiResponse,
};

#[derive(Debug, Default)]
pub(crate) struct DisplayAnswer {
    query_id: String,
    query_message: String,
    answers: Vec<Answer>,
    index: usize,
    width: usize,
    height: usize,
}

impl DisplayAnswer {
    pub(crate) fn draw(&mut self) -> Result<()> {
        if self.answers.is_empty() {
            return Ok(());
        }

        let stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        loop {
            self.redraw();

            let index: &mut usize = &mut self.index;

            let Some(ev) = stdin().keys().filter_map(|ev| ev.ok()).next() else {
                return Ok(());
            };

            match ev {
                Key::Left | Key::Up | Key::BackTab => {
                    *index = index
                        .checked_sub(1)
                        .unwrap_or_else(|| self.answers.len() - 1);
                }
                Key::Right | Key::Down | Key::Char('\t') => {
                    *index += 1;
                    if *index >= self.answers.len() {
                        *index = 0;
                    }
                }
                Key::Esc | Key::Char('q') | Key::Char('c') | Key::Char('x') | Key::Char('z') => {
                    break;
                }
                _ => continue,
            }
        }
        stdout.suspend_raw_mode().unwrap();

        Ok(())
    }

    fn question(&self) -> String {
        let width = self.width;
        let name = &self.query_id;
        let name = restrictions_width(format!(" Question: {name:.7}.."), width - 1);
        let body = &self.query_message;

        format!(
            "{bg_white}{color_black}{name:<width$}{color_reset}{bg_reset}\n\n{body}\n\n",
            bg_white = Bg(White),
            color_black = Fg(Black),
            color_reset = Fg(Reset),
            bg_reset = Bg(Reset),
        )
    }

    fn answer(&self) -> String {
        let width = self.width;
        let Answer { name, body, .. } = &self.answers[self.index];
        let name = restrictions_width(format!(" Answer: {name}"), width);

        format!(
            "{bg_white}{color_black}{name:<width$}{color_reset}{bg_reset}\n\n{body}\n\n",
            bg_white = Bg(White),
            color_black = Fg(Black),
            color_reset = Fg(Reset),
            bg_reset = Bg(Reset),
        )
    }

    fn footer(&self) -> String {
        let mut foot = String::new();

        if let Some(comment) = self.answers[self.index].comment.clone() {
            foot += &format!(
                "{style_underline}{:<width$}{style_reset}\n\n{comment}\n\n",
                "Comment on the assessment:",
                width = self.width
            );
        }
        foot += &("—".repeat(self.width) + "\n\r");
        foot += "Answer from the nodes:\n";

        for (index, Answer { name, .. }) in self.answers.iter().enumerate() {
            let name = format!(" {name} ");
            if self.index == index {
                foot += &format!(
                    "{bg_white}{color_black}{name}{color_reset}{bg_reset}\n",
                    bg_white = Bg(White),
                    color_black = Fg(Black),
                    color_reset = Fg(Reset),
                    bg_reset = Bg(Reset)
                );
            } else {
                foot += &name;
                foot += "\n";
            }
        }
        foot += &("—".repeat(self.width) + "\n");

        foot += "↑↓ - Switching responses. ESC - End the viewing\n";
        foot += &("—".repeat(self.width) + "\n");
        foot
    }

    fn clean(&self) {
        print!(
            "\x1Bc{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        );
        stdout().flush().unwrap();
    }

    fn redraw(&mut self) {
        self.clean();
        (self.width, self.height) = size();

        let output = self.to_string().replace("\n", "\r\n");
        print!("{output}");

        stdout().flush().unwrap();
    }
}

impl Display for DisplayAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.answers.is_empty() {
            return Ok(());
        }

        let question = self.question();
        let question_rows = question.count_rows(self.width);
        writeln!(f, "{question}")?;

        let answer = self.answer();
        let answer_rows = answer.count_rows(self.width);
        writeln!(f, "{answer}")?;

        let footer = self.footer();
        let footer_rows = footer.count_rows(self.width);

        let pad = self
            .height
            .checked_sub(question_rows)
            .and_then(|v| v.checked_sub(answer_rows))
            .and_then(|v| v.checked_sub(footer_rows))
            .unwrap_or_default();
        (0..pad).try_for_each(|_| writeln!(f))?;

        write!(f, "{footer}")?;

        Ok(())
    }
}

impl From<Query> for DisplayAnswer {
    fn from(value: Query) -> Self {
        let query_id = value.id.to_string();
        let query_message = value.request.query.message;

        let mut response = value.response;
        response.sort_by(|a, b| b.cmp(a));
        let answers: Vec<Answer> = response.iter().map(Into::into).collect();

        Self {
            index: 0,
            query_id,
            query_message,
            answers,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct Answer {
    name: String,
    comment: Option<String>,
    body: Body,
}

impl From<&NodeResult> for Answer {
    fn from(value: &NodeResult) -> Self {
        match value {
            NodeResult::Error(node, error) => Answer {
                name: format!("{:.7}.. (ERROR)", node.to_string()) + "",
                body: Body::Error(error.clone()),
                comment: None,
            },
            NodeResult::SentRequest(node) => Answer {
                name: format!("{:.7}.. (SENDING)", node.to_string()),
                body: Body::Error("The response has not been received yet".to_string()),
                comment: None,
            },
            NodeResult::Timeout(response) => {
                let mut ans: Answer = response.as_ref().into();
                ans.name = format!("{:.7}.. (TIME OUT)", ans.name);
                ans
            }
            NodeResult::NodeResponse(response) => {
                let mut ans: Answer = { &response.node_response }.into();
                ans.name = format!("{:.7}.. (AWAITING VERIFICATION)", ans.name);
                ans
            }
            NodeResult::Verified(response) => {
                let relevance = &response.result.relevance;
                let ans: Answer = { &response.result.material.node_response }.into();
                Answer {
                    name: format!("{:.7}.. (VERIFIED)({relevance}%)", ans.name),
                    comment: Some(response.result.description.clone()),
                    ..ans
                }
            }
        }
    }
}

impl From<&AiResponse> for Answer {
    fn from(value: &AiResponse) -> Self {
        Answer {
            name: format!("{:.7}..", value.pubkey),
            comment: None,
            body: Body::Success(value.response.clone()),
        }
    }
}

#[derive(Debug)]
enum Body {
    Success(String),
    Error(String),
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(s) => write!(f, "{s}"),
            Self::Error(s) => write!(
                f,
                "{color_red}{s}{color_reset}",
                color_red = Fg(Red),
                color_reset = Fg(Reset)
            ),
        }
    }
}

trait CalcRows: ToString {
    fn count_rows(&self, width: usize) -> usize {
        let width = width as f64;
        self.to_string()
            .lines()
            .map(|v| {
                if v.is_empty() {
                    1
                } else {
                    (v.chars().count() as f64 / width).ceil() as usize
                }
            })
            .sum()
    }
}

impl CalcRows for String {}
impl CalcRows for Body {}
impl CalcRows for DisplayAnswer {}

fn restrictions_width(str: String, width: usize) -> String {
    if str.len() > width {
        format!("{str:.width$}..", width = width - 2)
    } else {
        str.to_string()
    }
}

fn size() -> (usize, usize) {
    termion::terminal_size()
        .map(|(w, h)| (w as usize, h as usize))
        .unwrap_or((100, 300))
}
