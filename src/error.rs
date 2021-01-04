use std::convert::From;
use std::io::Cursor;

use rocket::{Request, Response, response};
use rocket::http::ContentType;
use rocket::http::hyper::header::CONTENT_LANGUAGE;
use rocket::http::Status;
use rocket::response::Responder;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::{Map, Value};

// https://tools.ietf.org/html/rfc7807

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub status: Status,
    pub type_uri: String,
    pub title: String,

    pub detail: Option<String>,
    pub instance_uri: Option<String>,

    pub body: Map<String, Value>,
}

impl Problem {
    pub fn new<TypeURI: Into<String>, Title: Into<String>>(status: Status, type_uri: TypeURI, title: Title) -> Problem {
        Problem {
            status,
            type_uri: type_uri.into(),
            title: title.into(),

            detail: None,
            instance_uri: None,

            body: Map::new(),
        }
    }

    // TODO: Add problem type URIs
    pub fn new_untyped<Title: Into<String>>(status: Status, title: Title) -> Problem {
        Problem {
            status,
            type_uri: "about:blank".to_string(),
            title: title.into(),

            detail: None,
            instance_uri: None,

            body: Map::new(),
        }
    }

    pub fn detail<Detail: Into<String>>(&mut self, value: Detail) -> &mut Problem {
        self.detail = Some(value.into());
        self
    }

    pub fn instance_uri<InstanceURI: Into<String>>(&mut self, value: InstanceURI) -> &mut Problem {
        self.instance_uri = Some(value.into());
        self
    }

    pub fn insert<Key: Into<String>>(&mut self, key: Key, value: Value) -> &mut Problem {
        self.body.insert(key.into(), value);
        self
    }

    pub fn insert_serialized<Key: Into<String>, V: Serialize>(&mut self, key: Key, value: V) -> &mut Problem {
        self.body.insert(
            key.into(),
            serde_json::to_value(value)
                .expect("Data must be JSON serializable."),
        );
        self
    }

    pub fn append(&mut self, value: Map<String, Value>) -> &mut Problem {
        self.body.append(&mut value.clone());
        self
    }

    pub fn append_serialized<Data: Serialize>(&mut self, data: Data) -> &mut Problem {
        let body = serde_json::to_value(data)
            .expect("Data must be JSON serializable.");

        match body {
            Value::Object(mut map) => self.body.append(&mut map),
            _ => panic!("Appended data must be an object when serialized."),
        }

        self
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.status.fmt(f)?;
        write!(f, ": {}", self.title)
    }
}

impl std::error::Error for Problem {}

impl<'r> Responder<'r, 'static> for Problem {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let mut body = self.body.clone();

        // Following are required by rfc7807
        body.insert(String::from("type"), serde_json::Value::from(self.type_uri));
        body.insert(String::from("title"), serde_json::Value::from(self.title));

        // Optional parameters as specified by rfc7807
        if self.detail.is_some() {
            body.insert(String::from("detail"), serde_json::Value::from(self.detail.unwrap()));
        }
        body.insert(String::from("status"), serde_json::Value::from(self.status.code));
        if self.instance_uri.is_some() {
            body.insert(String::from("instance"), serde_json::Value::from(self.instance_uri.unwrap()));
        }

        let body_string = serde_json::to_string(&body)
            .expect("Problem body must be convertible to a String.");

        Response::build()
            .status(self.status)
            .header(ContentType::new("application", "problem+json"))
            .raw_header(CONTENT_LANGUAGE.as_str(), "en")
            .sized_body(body_string.len(), Cursor::new(body_string))
            .ok()
    }
}

pub mod problems {
    use rocket::http::Status;

    use crate::error::Problem;

    #[inline]
    pub fn parse_problem() -> Problem {
        Problem::new_untyped(
            Status::BadRequest,
            "There was a problem parsing part of the request.",
        )
    }
}

impl std::convert::From<mongodb::error::Error> for Problem {
    fn from(e: mongodb::error::Error) -> Self {
        use mongodb::error::ErrorKind;

        fn mongodb_problem() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "MongoDB failed while processing request.",
            )
        }

        fn access_problem() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "Server was unable to access MongoDB.",
            )
        }

        fn bad_db_request() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "MongoDB was unable to process bad server request.",
            )
        }

        fn bson_problem() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "There was a problem with handling MongoDB bson.",
            )
        }

        fn timeout_problem() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "A timeout occurred while accessing MongoDB.",
            )
        }

        match e.kind.as_ref() {
            ErrorKind::AddrParse(_) => access_problem(),
            ErrorKind::ArgumentError { .. } => bad_db_request(),
            ErrorKind::AuthenticationError { .. } => access_problem(),
            ErrorKind::BsonDecode(_) => bson_problem(),
            ErrorKind::BsonEncode(_) => bson_problem(),
            ErrorKind::BulkWriteError(_) => bad_db_request(),
            ErrorKind::CommandError(_) => bad_db_request(),
            ErrorKind::DnsResolve(_) => access_problem(),
            ErrorKind::InternalError { .. } => mongodb_problem(),
            ErrorKind::InvalidDnsName(_) => access_problem(),
            ErrorKind::InvalidHostname { .. } => access_problem(),
            ErrorKind::Io(_) => mongodb_problem()
                .detail("IO error occurred. Submitted data might not be properly stored.")
                .clone(),
            ErrorKind::NoDnsResults(_) => access_problem(),
            ErrorKind::OperationError { .. } => mongodb_problem(),
            ErrorKind::OutOfRangeError(_) => mongodb_problem(),
            ErrorKind::ParseError { .. } => mongodb_problem(),
            ErrorKind::ResponseError { .. } => mongodb_problem(),
            ErrorKind::ServerSelectionError { .. } => access_problem(),
            ErrorKind::SrvLookupError { .. } => access_problem(),
            ErrorKind::TokioTimeoutElapsed(_) => timeout_problem(),
            ErrorKind::RustlsConfig(_) => mongodb_problem(),
            ErrorKind::TxtLookupError { .. } => mongodb_problem(),
            ErrorKind::WaitQueueTimeoutError { .. } => timeout_problem(),
            ErrorKind::WriteError(_) => mongodb_problem()
                .detail("A write error occurred. Submitted data might not be properly stored.")
                .clone(),
            _ => mongodb_problem(),
        }
    }
}

impl std::convert::From<bson::de::Error> for Problem {
    fn from(_: bson::de::Error) -> Self {
        Problem::new_untyped(
            Status::InternalServerError,
            "An error occurred while processing BSON data.",
        )
    }
}

impl std::convert::From<serde_json::Error> for Problem {
    fn from(_: serde_json::Error) -> Self {
        Problem::new_untyped(
            Status::InternalServerError,
            "An error occurred while processing JSON data.",
        )
    }
}

impl std::convert::From<jsonwebtoken::errors::Error> for Problem {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        fn jwt_problem() -> Problem {
            Problem::new_untyped(
                Status::InternalServerError,
                "Error while handling JWT.",
            )
        }

        match e.into_kind() {
            ErrorKind::InvalidToken => jwt_problem(),
            ErrorKind::InvalidSignature => jwt_problem(),
            ErrorKind::InvalidEcdsaKey => jwt_problem(),
            ErrorKind::InvalidRsaKey => jwt_problem(),
            ErrorKind::InvalidAlgorithmName => jwt_problem(),
            ErrorKind::InvalidKeyFormat => jwt_problem(),
            ErrorKind::ExpiredSignature => Problem::new_untyped(
                Status::BadRequest,
                "Expired JWT signature.",
            ),
            ErrorKind::InvalidIssuer => jwt_problem(),
            ErrorKind::InvalidAudience => jwt_problem(),
            ErrorKind::InvalidSubject => jwt_problem(),
            ErrorKind::ImmatureSignature => jwt_problem(),
            ErrorKind::InvalidAlgorithm => jwt_problem(),
            ErrorKind::Base64(_) => jwt_problem(),
            ErrorKind::Json(_) => jwt_problem(),
            ErrorKind::Utf8(_) => jwt_problem(),
            ErrorKind::Crypto(_) => jwt_problem(),
            ErrorKind::__Nonexhaustive => jwt_problem(),
        }
    }
}
