//! Line transport for the worker (invocation protocol §9), as a service
//! function over injectable input and output. One request per line, one
//! output line per input line, strictly in order. No process, no socket, no
//! thread: the caller owns the streams and the `Worker`.
//!
//! Policies, all tested:
//! - A line is complete only when its `\n` arrives. Input is read in bounded
//!   chunks and never accumulated beyond `max_line_bytes`; an over-long line
//!   is discarded to its end and answered with a framing fault.
//! - An empty line is a framing fault, not a request.
//! - Bytes left at EOF without `\n` are an incomplete line: not executed,
//!   answered with a framing fault, and the service stops with
//!   `StopReason::IncompleteLine`.
//! - Decoding and validation (`wire`) happen before `Worker::handle`.
//! - A response that would exceed `max_output_bytes` is never truncated: an
//!   `OutputLimit` fault names the action and the size instead. The action
//!   executed and its result exists in the worker, but it is not deliverable
//!   under this budget: replaying the `action_id` yields the same fault, and
//!   the limit cannot be raised during the episode.
//! - A write error stops the service and is returned to the caller; the
//!   request that was being answered has already executed.
//! - The service writes nothing but protocol lines; diagnostics are returned
//!   in `Served`, never mixed into the stream.

use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};

use crate::wire::{WIRE_VERSION, WireError, WireLimits, WireResponse, decode_envelope};
use residuum::experiments::worker::Worker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportLimits {
    /// Maximum bytes of one input line, excluding the newline.
    pub max_line_bytes: usize,
    /// Maximum bytes of one output line, excluding the newline.
    pub max_output_bytes: usize,
    pub wire: WireLimits,
}

impl Default for TransportLimits {
    fn default() -> Self {
        let wire = WireLimits::default();
        Self {
            max_line_bytes: wire.max_bytes,
            max_output_bytes: 16 * 1024 * 1024,
            wire,
        }
    }
}

/// Everything the service writes is one of these, one per line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Line {
    Response(WireResponse),
    Fault(Fault),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultClass {
    /// Framing: empty, over-long or incomplete line.
    Framing,
    /// Wire transport: bytes that are not a message.
    Transport,
    /// Wire validation: a message that is not acceptable.
    Validation,
    /// The response exists but exceeds the output limit; nothing was sent.
    OutputLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fault {
    pub wire_version: String,
    pub class: FaultClass,
    pub action_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    Eof,
    IncompleteLine,
    WriteError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Served {
    pub lines: usize,
    pub responses: usize,
    pub faults: usize,
    pub stop: StopReason,
}

enum Frame {
    Line(Vec<u8>),
    TooLong { kept: usize, discarded: usize },
    Incomplete(usize),
    Eof,
}

/// Read one frame without accumulating beyond `max`. On overflow the rest
/// of the line is consumed chunk by chunk and counted, not stored.
fn read_frame<R: BufRead>(reader: &mut R, max: usize) -> std::io::Result<Frame> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(if line.is_empty() {
                Frame::Eof
            } else {
                Frame::Incomplete(line.len())
            });
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(i) => {
                let room = max.saturating_sub(line.len());
                if i > room {
                    let discarded = i - room;
                    reader.consume(i + 1);
                    return Ok(Frame::TooLong {
                        kept: line.len() + room,
                        discarded,
                    });
                }
                line.extend_from_slice(&available[..i]);
                reader.consume(i + 1);
                return Ok(Frame::Line(line));
            }
            None => {
                let n = available.len();
                let room = max.saturating_sub(line.len());
                if n > room {
                    // Over the limit: keep nothing more, discard to the newline.
                    let kept = line.len() + room;
                    let mut discarded = n - room;
                    reader.consume(n);
                    loop {
                        let chunk = reader.fill_buf()?;
                        if chunk.is_empty() {
                            return Ok(Frame::Incomplete(kept + discarded));
                        }
                        match chunk.iter().position(|&b| b == b'\n') {
                            Some(j) => {
                                discarded += j;
                                reader.consume(j + 1);
                                return Ok(Frame::TooLong { kept, discarded });
                            }
                            None => {
                                discarded += chunk.len();
                                let len = chunk.len();
                                reader.consume(len);
                            }
                        }
                    }
                }
                line.extend_from_slice(available);
                reader.consume(n);
            }
        }
    }
}

fn fault(class: FaultClass, action_id: Option<String>, message: impl Into<String>) -> Line {
    Line::Fault(Fault {
        wire_version: WIRE_VERSION.into(),
        class,
        action_id,
        message: message.into(),
    })
}

fn emit<W: Write>(writer: &mut W, line: &Line) -> Result<(), String> {
    let text = serde_json::to_string(line).map_err(|e| e.to_string())?;
    writer
        .write_all(text.as_bytes())
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|e| e.to_string())
}

/// Serve requests from `reader` to `writer` until EOF, an incomplete line
/// or a write error. Strictly sequential: read, decode, validate, handle,
/// write, flush, next.
pub fn serve<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    worker: &mut Worker,
    limits: &TransportLimits,
) -> Served {
    let mut served = Served {
        lines: 0,
        responses: 0,
        faults: 0,
        stop: StopReason::Eof,
    };
    loop {
        let frame = match read_frame(reader, limits.max_line_bytes) {
            Ok(f) => f,
            Err(e) => {
                served.stop = StopReason::WriteError(format!("read: {e}"));
                return served;
            }
        };
        let line = match frame {
            Frame::Eof => return served,
            Frame::Incomplete(bytes) => {
                served.lines += 1;
                served.faults += 1;
                let f = fault(
                    FaultClass::Framing,
                    None,
                    format!("incomplete line of {bytes} bytes at end of input; not executed"),
                );
                if let Err(e) = emit(writer, &f) {
                    served.stop = StopReason::WriteError(e);
                    return served;
                }
                served.stop = StopReason::IncompleteLine;
                return served;
            }
            Frame::TooLong { kept, discarded } => {
                served.lines += 1;
                served.faults += 1;
                fault(
                    FaultClass::Framing,
                    None,
                    format!(
                        "line exceeds {} bytes ({} kept, {} discarded); not executed",
                        limits.max_line_bytes, kept, discarded
                    ),
                )
            }
            Frame::Line(bytes) if bytes.iter().all(|b| b.is_ascii_whitespace()) => {
                served.lines += 1;
                served.faults += 1;
                fault(FaultClass::Framing, None, "empty line is not a request")
            }
            Frame::Line(bytes) => {
                served.lines += 1;
                match decode_envelope(&bytes, &limits.wire) {
                    Err(WireError::Transport(m)) => {
                        served.faults += 1;
                        fault(FaultClass::Transport, None, m)
                    }
                    Err(WireError::Validation(m)) => {
                        served.faults += 1;
                        fault(FaultClass::Validation, None, m)
                    }
                    Ok(envelope) => {
                        let action_id = envelope.action_id.clone();
                        let response = worker.handle(envelope);
                        let line = Line::Response(WireResponse::from(&response));
                        match serde_json::to_string(&line) {
                            Err(e) => {
                                served.faults += 1;
                                fault(FaultClass::Transport, Some(action_id), e.to_string())
                            }
                            Ok(text) if text.len() > limits.max_output_bytes => {
                                served.faults += 1;
                                fault(
                                    FaultClass::OutputLimit,
                                    Some(action_id),
                                    format!(
                                        "response of {} bytes exceeds the output limit of {}; \
                                         nothing was truncated or sent; the action executed but \
                                         its result is not deliverable under this budget",
                                        text.len(),
                                        limits.max_output_bytes
                                    ),
                                )
                            }
                            Ok(_) => {
                                served.responses += 1;
                                line
                            }
                        }
                    }
                }
            }
        };
        if let Err(e) = emit(writer, &line) {
            served.stop = StopReason::WriteError(e);
            return served;
        }
    }
}

/// Decode one output line, for controllers and tests. `max_bytes` bounds
/// the line before parsing, like the input side.
pub fn decode_line(bytes: &[u8], max_bytes: usize) -> Result<Line, WireError> {
    if bytes.len() > max_bytes {
        return Err(WireError::Transport(format!(
            "output line of {} bytes exceeds the limit of {max_bytes}",
            bytes.len()
        )));
    }
    serde_json::from_slice(bytes).map_err(|e| match e.classify() {
        serde_json::error::Category::Data => WireError::Validation(e.to_string()),
        _ => WireError::Transport(e.to_string()),
    })
}
