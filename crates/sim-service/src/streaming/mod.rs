// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Server-Sent Events (SSE) Streaming Module
//!
//! Provides real-time simulation metrics streaming to clients.

pub mod sse;
pub mod websocket;

pub use sse::{
    create_broadcast_channel, SimulationMetricsEvent, SseBroadcaster, SseStream, StreamingState,
};
