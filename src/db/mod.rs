pub mod questdb;

pub use questdb::QuestDatabase;

// Type alias for backward compatibility
pub type SignalDatabase = QuestDatabase;
