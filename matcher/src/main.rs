mod prototype;
mod util;

use tree_sitter::Parser;
use util::{QueryExecutor, RawBuffer, TrimIndent};
use streaming_iterator::StreamingIterator;
use matcher_macros::tree_sitter_query;

tree_sitter_query!(MessageCaptures, "(message (message_name) @name) @message");

fn main() {
    let mut parser = Parser::new();

    parser.set_language(&tree_sitter_proto::LANGUAGE.into()).expect("Error loading protobuf grammar");

    // let file = std::fs::read_to_string("test.proto").expect("Error reading file");
    let source_code = "
        message MIOMMEDNAFI {
            uint32 CCFNINDAOGJ = 12; // comment
        }

        // MergeFrom: 0x0500C620
        // WriteTo: 0x0500C7C0
        message DEEMDJICKGG {
            uint32 JNLOABDHEIH = 2;
            uint32 OMPJKMGGKCM = 3;
            repeated string PPAMLEBAFPI = 4;
            uint32 AMDCFNKCLEE = 11;
            uint64 KHMIHMPCLJA = 9;
            PropExtraInfo CIEGHGBOIEO = 10;
        }
    ".trim_indent();

    let buffer = RawBuffer::from(source_code.clone());

    let tree = parser.parse(&source_code, None).expect("Error parsing file");

    // println!("{:#}", tree.root_node());
    // let message_name = tree.root_node().child(0).unwrap().child(1).unwrap();
    // println!("{:?}", source_code.extract_from_node(&message_name));

    let root_node = tree.root_node();

    // Query for all message nodes
    
    // let query = create_query(MESSAGE_QUERY);
    // let matches = run_query(root_node, &query, &buffer);
    
    // let mut captures_vec = Vec::new();

    // for capture in matches {
    //     if let Some(message_node) = capture.get("message") {
    //         if let Some(name_node) = capture.get("name") {
    //             // Utilize the QueryExecutor trait to execute and populate the struct
    //             if let Some(msg_capture) = MessageCaptures::execute(message_node, &buffer) {
    //                 captures_vec.push(msg_capture);
    //             }
    //         }
    //     }
    // }

    let captures_vec = MessageCaptures::execute(root_node, &buffer);

    for msg_capture in captures_vec {
        println!("Captured Message: {:?}", msg_capture);
    }

    // let mut cursor = QueryCursor::new();
    // cursor.
    // let matches = cursor.captures(&tree);
    // println!("{:?}", matches);
}
