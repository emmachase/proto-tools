mod debug;
mod parser;
mod prototype;
mod util;

use prototype::{ProtoDatabase, ProtoField, ProtoFieldKind, WeakProtoFieldKind};
use util::TrimIndent;
use std::collections::HashMap;
use crate::debug::DebugWithName;

macro_rules! dbg {
    ($db:expr, $arg:expr) => {
        ($arg.debug_with_name($db))
    };
}

fn main() {
    let proto_a = "
        message SingleField {
            uint32 field = 1;
        }

        message TestMessage {
            uint32 number = 2;
            uint32 number_2 = 3;
            repeated string string_list = 4;
            PropExtraInfo extra_info = 10;
            DupStruct dup_struct = 11;
            DupStruct dup_struct_2 = 12;
            map<string, float> float_map = 6;
        }
    ".trim_indent();

    let proto_b = "
        message SingleField {
            uint32 CCFNINDAOGJ = 53;
        }

        message TestMessage {
            uint32 JNLOABDHEIH = 1;
            uint32 GWFIOREJPIC = 2;
            OQUREKAMCNF QWEUIFSDNAX = 4;
            OQUREKAMCNF PQIOSKXMANZ = 5;
            repeated string PPAMLEBAFPI = 6;
            PropExtraInfo CIEGHGBOIEO = 3;
            map<string, float> APOCINBFAAB = 7;
        }
    ".trim_indent();

    let proto_db_a = parser::parse_proto(&proto_a);
    println!("{:#?}", proto_db_a);

    let proto_db_b = parser::parse_proto(&proto_b);
    println!("{:#?}", proto_db_b);

    let mut matcher = Matcher::new(proto_db_a, proto_db_b);
    matcher.static_match("TestMessage");
}

struct Matcher {
    proto_db_a: ProtoDatabase,
    proto_db_b: ProtoDatabase,
}

impl Matcher {
    fn new(proto_db_a: ProtoDatabase, proto_db_b: ProtoDatabase) -> Self {
        Self {
            proto_db_a,
            proto_db_b,
        }
    }

    fn static_match(&mut self, message_name: &str) {
        let message_a = self.proto_db_a.get_message(message_name).unwrap();
        let message_b = self.proto_db_b.get_message(message_name).unwrap();

        // Group fields by their type
        let fields_by_type_a = self.group_fields_by_type(&message_a.fields);
        let fields_by_type_b = self.group_fields_by_type(&message_b.fields);

        // Match fields that are unique by type
        for (type_name, fields_a) in &fields_by_type_a {
            if fields_a.len() == 1 {
                if let Some(fields_b) = fields_by_type_b.get(type_name) {
                    if fields_b.len() == 1 {
                        println!("Matched unique fields by type:");
                        println!("  {} -> {}", dbg!(&self.proto_db_a, fields_a[0].name), dbg!(&self.proto_db_b, fields_b[0].name));
                    }
                }
            }
        }

        // Match fields where count matches
        // TODO: This doesn't work yet, need to discriminate for type name within respective namespaces
        for (type_name, fields_a) in &fields_by_type_a {
            if let Some(fields_b) = fields_by_type_b.get(type_name) {
                if fields_a.len() > 1 && fields_a.len() == fields_b.len() {
                    println!("Potential matches (same type and count):");
                    for (field_a, field_b) in fields_a.iter().zip(fields_b.iter()) {
                        println!("  {} -> {}", dbg!(&self.proto_db_a, field_a.name), dbg!(&self.proto_db_b, field_b.name));
                    }
                }
            }
        }
    }

    fn group_fields_by_type<'a>(&'a self, fields: &'a [ProtoField]) -> HashMap<WeakProtoFieldKind, Vec<&'a ProtoField>> {
        let mut grouped = HashMap::new();
        for field in fields {
            grouped.entry(field.field_type.clone().into())
                .or_insert_with(Vec::new)
                .push(field);
        }
        grouped
    }
}
