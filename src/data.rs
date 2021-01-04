use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerType {
    Bool,
    Number,
    Short,
    Long,
    FillIn,
    Match(Vec<(String, String)>),
    Single {
        options: Vec<String>,
        shuffle: bool,
    },
    Multiple {
        options: Vec<String>,
        shuffle: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerValidation {
    Bool {
        expected: bool,
    },
    Exact {
        case_sensitive: bool,
        expected: String,
    },
    NumberRange {
        min: f64,
        max: f64,
    },
    Regex {
        case_sensitive: bool,
        expr: String,
    },
    Multiple {
        case_sensitive: bool,
        expected: Vec<String>,
    },
    External {
        // for running external, locally installed validation programs/scripts.
        // TODO: Think about code injection. Can't just insert answers.
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Part {
    Content {
        id: Uuid,
        title: String,
        text: String,
    },
    Interact {
        id: Uuid,
        text: String,
        ans: AnswerType,

        time_limit: Option<Duration>,

        value: f32,
        validation: Option<AnswerValidation>,
        partial: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Answer {
    Bool(bool),
    Number(f64),
    Short(String),
    Long(String),
    FillIn(Vec<String>),
    Single(i32),
    Multiple(Vec<i32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    id: Uuid,
    name: String,
    started_on: DateTime<Utc>,
    answers: HashMap<Uuid, Answer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    id: Uuid,
    name: String,
    desc: String,
    author: String,
    created: DateTime<Utc>,
    parts: Vec<Part>,

    time_limit: Option<Duration>,
    expect_focus: bool,
    show_answer: bool,
    show_results: bool,

    public: bool,
    open_on: Option<DateTime<Utc>>,
    close_on: Option<DateTime<Utc>>,
    begin_buffer: Option<Duration>,
    participants: Vec<String>,
}
