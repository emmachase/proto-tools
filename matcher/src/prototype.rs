#![allow(dead_code)] // TODO: Remove this

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtoName(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtoMessage {
    name: ProtoName,
    fields: Vec<ProtoField>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtoField {
    name: ProtoName,
    field_type: ProtoType,
    field_number: u32,
    repeated: bool,
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum ProtoType {
    Bool,
    Int32,
    Int64,
    String,
    Type(ProtoName), // message, enum, etc.
}


impl ProtoField {
    fn has_same_type(&self, other: &ProtoField) -> bool {
        self.field_type == other.field_type && self.repeated == other.repeated
    }
}
