mod debug;
mod parser;
mod prototype;
mod util;

use util::TrimIndent;
fn main() {
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

    let proto_db = parser::parse_proto(&source_code);
    println!("{:#?}", proto_db);
}
