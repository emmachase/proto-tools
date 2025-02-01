use ropey::Rope;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor, QueryMatches, TextProvider};
use tree_sitter_proto::LANGUAGE;
use std::collections::HashMap;

pub trait TrimIndent {
    fn trim_indent(&self) -> String;
}

impl TrimIndent for String {
    fn trim_indent(&self) -> String {
        let first_indent = self.lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .unwrap_or(0);

        let mut lines = self.lines()
            .skip_while(|line| line.trim().is_empty())
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    line.to_string()
                        .chars()
                        .skip(first_indent.min(line.len() - line.trim_start().len()))
                        .collect()
                }
            })
            .collect::<Vec<String>>();

        // Drop empty lines at the end
        while lines.last().unwrap().is_empty() {
            lines.pop();
        }

        lines.join("\n")
    }
}

impl TrimIndent for &str {
    fn trim_indent(&self) -> String {
        self.to_string().trim_indent()
    }
}

pub trait ExtractText {
    // fn extract_from_node(&self, node: &Node) -> String;
    fn text(&self, buffer: &RawBuffer) -> String;
}

impl ExtractText for Node<'_> {
    fn text(&self, buffer: &RawBuffer) -> String {
        let byte_range = self.byte_range();
        buffer.rope.slice(byte_range.start..byte_range.end).to_string()
    }
}

// impl ExtractText for String {
//     fn extract_from_node(&self, node: &Node) -> String {
//         let byte_range = node.byte_range();
//         self.get(byte_range.start..byte_range.end).unwrap_or_default().to_string()
//     }
// }

/// RawBuffer structure as used in main and macro
pub struct RawBuffer {
    pub rope: Rope,
}

impl RawBuffer {
    pub fn from(source: String) -> Self {
        Self {
            rope: Rope::from_str(&source),
        }
    }
}

pub struct RopeByteChunks<'a>(ropey::iter::Chunks<'a>);

impl<'a> Iterator for RopeByteChunks<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(str::as_bytes)
    }
}

pub struct RopeTextProvider<'a>(&'a RawBuffer);

impl<'a> TextProvider<&'a [u8]> for RopeTextProvider<'a> {
    type I = RopeByteChunks<'a>;

    fn text(&mut self, node: Node) -> Self::I {
        let buffer_range = node.byte_range();
        let slice = self.0.rope.slice(buffer_range.start..buffer_range.end);

        RopeByteChunks(slice.chunks())
    }
}

impl<'a> From<&'a RawBuffer> for RopeTextProvider<'a> {
    fn from(buffer: &'a RawBuffer) -> Self {
        Self(buffer)
    }
}

/// Creates a Tree-sitter Query from a string
pub fn create_query(query_str: &str) -> Query {
    // Assuming LANGUAGE is already defined and loaded
    Query::new(&tree_sitter_proto::LANGUAGE.into(), query_str).expect("Invalid query")
}

/// Runs a Tree-sitter Query against a node and buffer
// pub fn run_query<'tree, 'query>(node: Node<'tree>, query: &'query Query, buffer: &'tree RawBuffer) -> QueryMatches<'query, 'tree, RopeTextProvider<'tree>, &'tree [u8]> {
    
// }

/// Trait to execute a Tree-sitter query and populate the struct with captures.
pub trait QueryExecutor<'tree> {
    fn execute(node: Node<'tree>, buffer: &RawBuffer) -> Vec<Self>
    where
        Self: Sized;
}

// pub struct Capture<'a> {
//     pub text: String,
//     pub node: Node<'a>,
// }
