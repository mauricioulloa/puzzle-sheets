//! Printable puzzle worksheets for students and educators. Chess for now;
//! the sheet itself does not know that, so other puzzle types can follow.
//!
//! The crate is split into a library and a thin CLI binary so the web and MCP
//! surfaces can be exercised directly from integration tests.

pub mod chess;
pub mod i18n;
pub mod mcp;
pub mod serve;
pub mod sheet;
pub mod web;
pub mod worksheet;
