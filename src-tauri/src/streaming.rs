//! Streaming response support
//! Handles SSE-style streaming for AI responses

use anyhow::Result;
use futures::Stream;
use pin_project::pin_project;
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream event parser for Server-Sent Events
#[pin_project]
pub struct EventStream<S> {
    #[pin]
    stream: S,
    buffer: String,
}

impl<S> EventStream<S>
where
    S: Stream<Item = Result<String>>,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: String::new(),
        }
    }
}

impl<S> Stream for EventStream<S>
where
    S: Stream<Item = Result<String>>,
{
    type Item = Result<String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // Process any buffered lines
            if let Some(pos) = this.buffer.find('\n') {
                let line = this.buffer[..pos].to_string();
                *this.buffer = this.buffer[pos + 1..].to_string();

                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        return Poll::Ready(None);
                    }
                    return Poll::Ready(Some(Ok(data.to_string())));
                }
                continue;
            }

            // Need more data
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    this.buffer.push_str(&chunk);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(None) => {
                    if !this.buffer.is_empty() {
                        let line = std::mem::take(this.buffer);
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                return Poll::Ready(None);
                            }
                            return Poll::Ready(Some(Ok(data.to_string())));
                        }
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Parse a JSON stream chunk
pub fn parse_stream_chunk<T>(chunk: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(chunk).map_err(|e| anyhow::anyhow!("Failed to parse chunk: {}", e))
}

/// Accumulator for streaming text content
#[derive(Debug, Default)]
pub struct TextAccumulator {
    content: String,
}

impl TextAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_text(&mut self, text: impl AsRef<str>) {
        self.content.push_str(text.as_ref());
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn into_content(self) -> String {
        self.content
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_accumulator() {
        let mut acc = TextAccumulator::new();
        acc.add_text("Hello");
        acc.add_text(" ");
        acc.add_text("World");
        assert_eq!(acc.content(), "Hello World");
    }

    #[test]
    fn test_text_accumulator_into() {
        let mut acc = TextAccumulator::new();
        acc.add_text("Test");
        let content = acc.into_content();
        assert_eq!(content, "Test");
    }
}
