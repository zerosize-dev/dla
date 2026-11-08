//! DLA benchmark runner and grader.
//!
//! The grading rules are specified in `bench/GRADING.md` and the fixture
//! contract in `bench/FIXTURES.md`; this crate implements both.

pub mod compare;
pub mod dynamic_cmd;
pub mod fixture;
pub mod grade;
pub mod guard;
pub mod model;
pub mod run;
pub mod schema_cmd;
pub mod static_cmd;
pub mod table;
pub mod validate;

/// Result type of the command implementations: the error is a ready message.
pub type Res<T> = Result<T, String>;
