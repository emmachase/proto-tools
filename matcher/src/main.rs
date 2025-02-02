mod debug;
mod prototype;
mod util;

use debug::DebugWithName;
use prototype::{ProtoDatabase, ProtoField, ProtoFieldKind, ProtoMessage, ProtoType};
use tree_sitter::{Node, Parser};
use util::{ExtractText, QueryExecutor, RawBuffer, TrimIndent};
use streaming_iterator::StreamingIterator;
use matcher_macros::tree_sitter_query;

tree_sitter_query! {
    IdentifierQuery("(identifier) @name")
    MessageQuery("(message (message_name) @name) @node")
    FieldQuery("
        (field
            \"repeated\"? @repeated
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

    let mut proto_db = ProtoDatabase::new();

    // Register all identifiers first
    let identifiers = IdentifierQuery::execute(root_node, &buffer);
    for identifier in identifiers {
        if let Some(name) = identifier.name {
            proto_db.register_identifier(name.text(&buffer));
        }
    }

    println!("Identifier DB: {:?}", proto_db.identifier_db);

    let messages = MessageQuery::execute(root_node, &buffer);
    for message in messages {
        let mut result = ProtoMessage {
            name: proto_db.lookup_name(message.name.unwrap().text(&buffer).as_str()),
            fields: Vec::new(),
        };

        let fields = FieldQuery::execute(message.node.unwrap(), &buffer);
        for field in fields {
            result.fields.push(ProtoField {
                name: proto_db.lookup_name(field.name.unwrap().text(&buffer).as_str()),
                field_type: {
                    if field.is_map_field() {
                        let key_type = get_simple_field_type(&field.key_type.unwrap(), &buffer, &proto_db);
                        let value_type = get_simple_field_type(&field.value_type.unwrap(), &buffer, &proto_db);

                        ProtoFieldKind::Map(Box::new(key_type), Box::new(value_type))
                    } else {
                        let field_type_scalar = get_simple_field_type(&field.typ.unwrap(), &buffer, &proto_db);
                        
                        match field.repeated.is_some() {
                            true => ProtoFieldKind::Repeated(Box::new(field_type_scalar)),
                            false => ProtoFieldKind::Scalar(field_type_scalar),
                        }
                    }
                },
                field_number: field.number.unwrap().text(&buffer).parse().expect("Failed to parse field number"),
            });
        }
        
        proto_db.register_message(result);
    }
    
    println!("Message DB: {}", proto_db.message_db.debug_with_name(&proto_db));
}

fn get_simple_field_type(type_node: &Node, buffer: &RawBuffer, proto_db: &ProtoDatabase) -> ProtoType {
    match type_node.kind() {
        "bool" => ProtoType::Bool,
        "float" => ProtoType::Float,
        "double" => ProtoType::Double,
        "int32" => ProtoType::Int32,
        "int64" => ProtoType::Int64,
        "uint32" => ProtoType::Uint32,
        "uint64" => ProtoType::Uint64,
        "sint32" => ProtoType::Sint32,
        "sint64" => ProtoType::Sint64,
        "fixed32" => ProtoType::Fixed32,
        "fixed64" => ProtoType::Fixed64,
        "sfixed32" => ProtoType::Sfixed32,
        "sfixed64" => ProtoType::Sfixed64,
        "string" => ProtoType::String,
        "bytes" => ProtoType::Bytes,
        "message_or_enum_type" => ProtoType::Type(proto_db.lookup_name(type_node.text(&buffer).as_str())),
        _ => panic!("Unknown type: {}", type_node.kind()),
    }
}
