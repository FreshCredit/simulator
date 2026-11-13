// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Agent Memory Subsystem (P1-SIM-4)
//!
//! Provides episodic and semantic memory storage for agents with
//! retrieval, consolidation, and forgetting mechanisms.

pub mod consolidation;
pub mod retrieval;
pub mod store;
pub mod types;

pub use consolidation::MemoryConsolidator;
pub use retrieval::MemoryRetriever;
pub use store::{AgentMemory, MemoryStore};
pub use types::{
    EpisodicMemory, MemoryEntry, MemoryImportance, MemoryQuery, MemoryRetrievalResult, MemoryType,
    SemanticMemory,
};
