mod debug;
mod parser;
mod prototype;
mod util;

use prototype::ProtoDatabase;
use util::TrimIndent;
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

        println!("{:#?}", message_a);
        println!("{:#?}", message_b);
    }
}
