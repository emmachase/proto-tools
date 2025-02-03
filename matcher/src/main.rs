mod debug;
mod parser;
mod prototype;
mod util;

use itertools::Itertools;
use prototype::{LogIfErr, ProtoDatabase, ProtoField, ProtoFieldKind, ProtoMessage, WeakProtoFieldKind};
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
            uint32 number = 1;
        }

        message TestMessage {
            uint32 number = 2;
            uint32 number_2 = 3;
            repeated string string_list = 4;
            AnotherInfo another_info = 15;
            PropExtraInfo extra_info = 10;
            DupStruct dup_struct = 11;
            DupStruct dup_struct_2 = 12;
            map<string, float> float_map = 6;
        }
    ".trim_indent();

    let proto_b = "
        message SingleField {
            uint32 JNLOABDHEIH = 53;
            OQUREKAMCNF QWEUIFSDNAX = 4;
        }

        message TestMessage {
            uint32 JNLOABDHEIH = 1;
            uint32 GWFIOREJPIC = 2;
            OQUREKAMCNF QWEUIFSDNAX = 4;
            OQUREKAMCNF PQIOSKXMANZ = 5;
            repeated string PPAMLEBAFPI = 6;
            PropExtraInfo CIEGHGBOIEO = 3;
            AnotherInfo another_info = 16;
            map<string, float> APOCINBFAAB = 7;
            uint64 PDOQWIJLSAM = 9;
        }
    ".trim_indent();

    let proto_db_a = parser::parse_proto(&proto_a);
    // println!("{:#?}", proto_db_a);

    let proto_db_b = parser::parse_proto(&proto_b);
    // println!("{:#?}", proto_db_b);

    let mut matcher = Matcher::new(proto_db_a, proto_db_b);

    // TODO: Loop until no more matches are found
    matcher.static_match("SingleField");
    matcher.static_match("TestMessage");

    let name_translation = matcher.into_db_b().generate_nametranslation();

    // Print translated proto_b
    let mut translated_proto_b = proto_b.clone();
    for (old_name, new_name) in name_translation {
        translated_proto_b = translated_proto_b.replace(&old_name, &new_name);
    }
    println!("{}", translated_proto_b);
}

struct Matcher {
    proto_db_a: ProtoDatabase,
    proto_db_b: ProtoDatabase,
}

impl Matcher {
    fn into_db_b(self) -> ProtoDatabase {
        self.proto_db_b
    }
}

impl Matcher {
    fn new(proto_db_a: ProtoDatabase, proto_db_b: ProtoDatabase) -> Self {
        Self {
            proto_db_a,
            proto_db_b,
        }
    }

    fn remove_resolved_fields(&self, mut message_a: ProtoMessage, mut message_b: ProtoMessage) -> (ProtoMessage, ProtoMessage) {
        let mut resolved_field_names = Vec::new();
        for field in &message_b.fields {
            if self.proto_db_b.is_resolved(&field.name) {
                resolved_field_names.push(field.name.name(&self.proto_db_b));
            }
        }

        // TODO: Probably a better way to do this
        message_a.fields.retain(|field| !resolved_field_names.contains(&field.name.name(&self.proto_db_a)));
        message_b.fields.retain(|field| !resolved_field_names.contains(&field.name.name(&self.proto_db_b)));

        (message_a, message_b)
    }

    fn static_match(&mut self, message_name: &str) {
        let message_a = self.proto_db_a.get_message(message_name).unwrap();
        let message_b = self.proto_db_b.get_message(message_name).unwrap();

        // Remove fields that are already fully resolved in message_b
        let (message_a, message_b) = self.remove_resolved_fields(message_a, message_b);

        // Group fields by their type
        let fields_by_weak_type_a = self.group_fields_by_weak_type(&message_a.fields);
        let fields_by_weak_type_b = self.group_fields_by_weak_type(&message_b.fields);

        let fields_by_strong_type_a: Vec<_> = self.group_fields_by_type(&message_a.fields).into_iter().collect();
        let fields_by_strong_type_b: Vec<_> = self.group_fields_by_type(&message_b.fields).into_iter().collect();

        // TODO: Even when we have a strong match, we should probably still check sub-type structure 

        // Check by weak type first
        for (type_name, fields_b) in &fields_by_strong_type_b {
            if let Some(fields_a_weak) = fields_by_weak_type_a.get(&WeakProtoFieldKind::from(*type_name)) {
                // Check for the simple case where there is only one field of this type in the other proto
                if fields_b.len() == 1 {
                    // Can directly match fields that are unique by weak type (only one Message or primitive for this type)
                    if fields_a_weak.len() == 1 {
                        println!("Matched unique fields by weak type:");
                        println!("  {} -> {}", dbg!(&self.proto_db_a, fields_a_weak[0].name), dbg!(&self.proto_db_b, fields_b[0].name));

                        fields_a_weak[0].try_resolve_in(&self.proto_db_a, &mut self.proto_db_b, &fields_b[0]).log_if_err();

                        continue;
                    }
                }

                if type_name.is_type_ref() {
                    // Match occurrence patterns
                    // e.g. 1 occurrence of type A, 2 occurrences of type B
                    //   But occurrences count must be unique, otherwise it's ambiguous
                    //   e.g. 2 occurrences of type A, 2 occurrences of type B -> ambiguous
                    //     But if in the same message there is only 1 occurrance of type C, then that one can be decided

                    // TODO: This can probably be done outside of the loop
                    let a_chunks = fields_a_weak
                        .iter().chunk_by(|el| el.field_type)
                        .into_iter()
                        .map(|(_key, chunk)| chunk.map(|el| *el).collect::<Vec<_>>())
                        .collect::<Vec<_>>();

                    let a_chunks_by_occurrence = a_chunks.into_iter()
                        .fold(HashMap::new(), |mut map, chunk| {
                            map.entry(chunk.len())
                               .or_insert_with(Vec::new)
                               .push(chunk);
                            map
                        });

                    let len_b = fields_b.len();
                    if let Some(a_chunks) = a_chunks_by_occurrence.get(&len_b) {
                        if a_chunks.len() == 1 {
                            if len_b == 1 {
                                // Direct match
                                println!("Direct match: {}", dbg!(&self.proto_db_a, a_chunks[0]));

                                a_chunks[0][0].try_resolve_in(&self.proto_db_a, &mut self.proto_db_b, &fields_b[0]).log_if_err();
                            } else {
                                // Can resolve type, but field names can only be resolved by data-match
                                println!("Occurrence match requires data-match: {}", dbg!(&self.proto_db_a, a_chunks[0]));

                                // Only need to resolve first field's type since they are all the same type
                                let first_field = &a_chunks[0][0];
                                let b_type = fields_b[0].field_type.inner_type();
                                first_field.field_type.inner_type().try_resolve_in(&self.proto_db_a, &mut self.proto_db_b, &b_type).log_if_err();
                            }
                        } else {
                            // TODO: If type names are resolved, we can try to match based on that

                            println!("Ambiguous match by occurrence: {}", dbg!(&self.proto_db_a, a_chunks));
                        }
                    } else {
                        println!("No match by occurrence: {}", dbg!(&self.proto_db_a, a_chunks_by_occurrence));
                    }

                    // TODO: When ambiguous, try to match subtype structures to resolve (only if structures are unique)
                    //       For now, we should not allow variation in structure for resolution. 
                    //       In the future, we can maybe implement confidence-based fuzzy match for sub-structures
                } else {
                    // Primitive type, can't be matched any further statically
                    println!("Primitive type with multiple fields, can't be matched any further statically: {}", dbg!(&self.proto_db_b, fields_b));
                }

            } else {
                // New field in b, nothing we can do
                println!("Found field(s) with new type in b: {}", dbg!(&self.proto_db_b, fields_b));
            }
        }
    }

    fn group_fields_by_type(&self, fields: &[ProtoField]) -> HashMap<ProtoFieldKind, Vec<ProtoField>> {
        let mut grouped = HashMap::new();
        for field in fields {
            grouped.entry(field.field_type.clone())
                .or_insert_with(Vec::new)
                .push(field.clone());
        }
        grouped
    }

    fn group_fields_by_weak_type(&self, fields: &[ProtoField]) -> HashMap<WeakProtoFieldKind, Vec<ProtoField>> {
        let mut grouped = HashMap::new();
        for field in fields {
            grouped.entry(field.field_type.clone().into())
                .or_insert_with(Vec::new)
                .push(field.clone());
        }

        // Sort by type name
        for fields in grouped.values_mut() {
            fields.sort_by_key(|field| field.field_type.inner_type().clone());
        }

        grouped
    }
}
