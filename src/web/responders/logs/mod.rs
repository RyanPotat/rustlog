mod join_iter;
mod json_stream;
mod text_stream;

use self::{json_stream::JsonLogsStream, text_stream::TextLogsStream};
use crate::logs::{schema::Message, stream::LogsStream};
use aide::OperationOutput;
use axum::{
    body::StreamBody,
    http::HeaderValue,
    response::{IntoResponse, IntoResponseParts, Response},
    Json,
};
use futures::TryStreamExt;
use indexmap::IndexMap;
use mime_guess::{
    mime::{APPLICATION_JSON, TEXT_PLAIN_UTF_8},
    Mime,
};
use reqwest::header::CONTENT_TYPE;
use schemars::JsonSchema;

pub struct LogsResponse {
    pub stream: LogsStream,
    pub response_type: LogsResponseType,
}

pub enum LogsResponseType {
    Raw,
    Text,
    Json,
}

/// Used for schema only, actual serialization is manual
#[derive(JsonSchema)]
pub struct JsonLogsResponse<'a> {
    pub messages: Vec<Message<'a>>,
}

impl IntoResponse for LogsResponse {
    fn into_response(self) -> Response {
        match self.response_type {
            LogsResponseType::Raw => {
                let stream = self.stream.map_ok(|mut line| {
                    line.push('\n');
                    line
                });

                (set_content_type(&TEXT_PLAIN_UTF_8), StreamBody::new(stream)).into_response()
            }
            LogsResponseType::Text => {
                let stream = TextLogsStream::new(self.stream);
                (set_content_type(&TEXT_PLAIN_UTF_8), StreamBody::new(stream)).into_response()
            }
            LogsResponseType::Json => {
                let stream = JsonLogsStream::new(self.stream);
                (set_content_type(&APPLICATION_JSON), StreamBody::new(stream)).into_response()
            }
        }
    }
}

fn set_content_type(content_type: &'static Mime) -> impl IntoResponseParts {
    [(
        CONTENT_TYPE,
        HeaderValue::from_static(content_type.as_ref()),
    )]
}

impl OperationOutput for LogsResponse {
    type Inner = Self;

    fn operation_response(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        let mut content = IndexMap::with_capacity(2);

        let json_operation_response =
            Json::<JsonLogsResponse>::operation_response(ctx, operation).unwrap();
        content.extend(json_operation_response.content);

        let plain_response = String::operation_response(ctx, operation).unwrap();
        content.extend(plain_response.content);

        Some(aide::openapi::Response {
            description: "Logs response".into(),
            content,
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        let res = Self::operation_response(ctx, operation).unwrap();

        vec![(Some(200), res)]
    }
}
