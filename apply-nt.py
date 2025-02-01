import re
import argparse
import os
def main(proto_path, translations_path, output_path):
    with open(proto_path, "r") as proto_file:
        proto_content = proto_file.read()

    with open(translations_path, "r") as translations_file:
        translation_content = translations_file.read()

    for (find, replace) in re.findall(r"(\w+) -> (\w+)", translation_content):
        proto_content = proto_content.replace(find, replace)

    with open(output_path, "w+") as translated_file:
        translated_file.write(proto_content)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Apply name translations to a proto file.")
    parser.add_argument("proto_path", help="Path to the proto file")
    parser.add_argument("translations_path", help="Path to the translations file")
    parser.add_argument("-o", metavar="OUTPUT_FILE", default=None, help="Path to the output translated file (default: translated_<proto_path>)")

    args = parser.parse_args()

    if args.o is not None:
        output_path = args.o
    else:
        output_dir = os.path.dirname(args.proto_path)
        output_file = os.path.basename(args.proto_path)
        output_path = os.path.join(output_dir, f"translated_{output_file}")

    main(args.proto_path, args.translations_path, output_path)
