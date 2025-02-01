mod prototype;
mod util;

use std::collections::HashMap;

use tree_sitter::Parser;
use util::{ExtractText, QueryExecutor, RawBuffer, TrimIndent};
use streaming_iterator::StreamingIterator;
use matcher_macros::tree_sitter_query;

tree_sitter_query! {
    IdentifierQuery("(identifier) @name")
    MessageQuery("(message (message_name) @name) @node")
    FieldQuery("
        (field
            (type _ @typ)
            (identifier) @name
            (field_number) @number
        ) @node

        (map_field
            (key_type _ @key_type)
            (type _ @value_type)
            (identifier) @name
            (field_number) @number
        ) @node
    ")
}

impl<'tree> FieldQuery<'tree> {
    fn is_map_field(&self) -> bool {
        self.key_type.is_some()
    }
}

fn main() {
    let mut parser = Parser::new();

    parser.set_language(&tree_sitter_proto::LANGUAGE.into()).expect("Error loading protobuf grammar");

    let source_code = "
        message MIOMMEDNAFI {
            uint32 CCFNINDAOGJ = 12; // comment
        }

        // MergeFrom: 0x0500C620
        // WriteTo: 0x0500C7C0
        message DEEMDJICKGG {
            uint32 JNLOABDHEIH = 2;
            repeated string PPAMLEBAFPI = 4;
            PropExtraInfo CIEGHGBOIEO = 10;
            map<string, float> APOCINBFAAB = 6;
        }
    ".trim_indent();
    let buffer = RawBuffer::from(source_code.clone());

    let tree = parser.parse(&source_code, None).expect("Error parsing file");
    let root_node = tree.root_node();


    let mut identifier_counter = 0;
    let mut identifier_db = HashMap::new();

    let identifiers = IdentifierQuery::execute(root_node, &buffer);

    for identifier in identifiers {
        if let Some(name) = identifier.name {
            let text = name.text(&buffer);
            if !identifier_db.contains_key(&text) {
                identifier_db.insert(text, identifier_counter);
                identifier_counter += 1;
            }
        }
    }

    println!("Identifier DB: {:?}", identifier_db);

    let messages = MessageQuery::execute(root_node, &buffer);

    for message in messages {
        println!("Message: {:#}", message.node.unwrap());
        let fields = FieldQuery::execute(message.node.unwrap(), &buffer);
        for field in fields {
            if field.is_map_field() {
                println!("Field: {:#} map<{}, {}>", field.node.unwrap(), field.key_type.unwrap().kind(), field.value_type.unwrap().kind());
            } else {
                println!("Field: {:#} {} {}", field.node.unwrap(), field.typ.unwrap().text(&buffer), field.typ.unwrap().kind());
            }
        }
    }
}
