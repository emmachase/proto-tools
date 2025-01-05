import os
import re
from typing import Set, Dict, List
from collections import defaultdict

class DependencyGraph:
    def __init__(self):
        self.graph = defaultdict(set)
        
    def add_dependency(self, from_type: str, to_type: str):
        """Add a dependency edge to the graph"""
        # Don't add self-dependencies
        if from_type != to_type:
            self.graph[from_type].add(to_type)
    
    def get_strongly_connected_components(self) -> List[Set[str]]:
        """Find strongly connected components using Kosaraju's algorithm"""
        def dfs_first_pass(node: str, visited: Set[str], stack: List[str]):
            visited.add(node)
            # Convert to list for stable iteration
            for neighbor in sorted(list(self.graph[node])):
                if neighbor not in visited:
                    dfs_first_pass(neighbor, visited, stack)
            stack.append(node)
        
        def dfs_second_pass(node: str, visited: Set[str], component: Set[str], transposed: Dict[str, Set[str]]):
            visited.add(node)
            component.add(node)
            # Convert to list for stable iteration
            for neighbor in sorted(list(transposed[node])):
                if neighbor not in visited:
                    dfs_second_pass(neighbor, visited, component, transposed)
        
        # First pass: fill the stack
        visited = set()
        stack = []
        # Convert to list for stable iteration
        for node in sorted(list(self.graph.keys())):
            if node not in visited:
                dfs_first_pass(node, visited, stack)
        
        # Create transposed graph
        transposed = defaultdict(set)
        # Convert to list for stable iteration
        for node in list(self.graph.keys()):
            for neighbor in list(self.graph[node]):
                transposed[neighbor].add(node)
        
        # Second pass: find SCCs
        visited = set()
        components = []
        
        while stack:
            node = stack.pop()
            if node not in visited:
                component = set()
                dfs_second_pass(node, visited, component, transposed)
                if len(component) > 1:  # Only include components with cycles (size > 1)
                    components.append(component)
        
        return components

def extract_dependencies(message_content: str) -> Set[str]:
    """
    Extract all custom type dependencies from a message definition.
    Excludes built-in types like uint32, string, etc.
    """
    built_in_types = {
        'double', 'float', 'int32', 'int64', 'uint32', 'uint64', 
        'sint32', 'sint64', 'fixed32', 'fixed64', 'sfixed32', 'sfixed64',
        'bool', 'string', 'bytes', 'map', 'oneof'
    }
    
    types = set()
    
    # Find map type definitions
    map_pattern = r'map\s*<\s*(\w+)\s*,\s*(\w+)\s*>'
    for map_match in re.finditer(map_pattern, message_content):
        key_type, value_type = map_match.groups()
        if key_type not in built_in_types:
            types.add(key_type)
        if value_type not in built_in_types:
            types.add(value_type)
    
    # Find regular type definitions
    type_pattern = r'(?:repeated\s+)?(\w+)\s+\w+\s*=\s*\d+;'
    types.update(re.findall(type_pattern, message_content))
    
    return {t for t in types if t not in built_in_types}

def find_matching_brace(content: str, start_pos: int) -> int:
    """Find the position of the matching closing brace"""
    count = 1
    pos = start_pos + 1
    while count > 0 and pos < len(content):
        if content[pos] == '{':
            count += 1
        elif content[pos] == '}':
            count -= 1
        pos += 1
    return pos if count == 0 else -1

def extract_block(content: str, start_pos: int) -> tuple[str, int]:
    """Extract a complete block of content including nested braces"""
    opening_brace_pos = content.find('{', start_pos)
    if opening_brace_pos == -1:
        return "", -1
    
    closing_brace_pos = find_matching_brace(content, opening_brace_pos)
    if closing_brace_pos == -1:
        return "", -1
        
    return content[start_pos:closing_brace_pos], closing_brace_pos

def parse_proto_file(input_file: str, output_dir: str = "proto_split") -> None:
    """Parse the proto-like file and split it into separate files"""
    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()

    os.makedirs(output_dir, exist_ok=True)

    # Extract syntax declaration
    syntax_match = re.search(r'syntax\s*=\s*"proto3"\s*;', content)
    syntax_declaration = syntax_match.group(0) if syntax_match else ''

    # Initialize dependency graph
    dep_graph = DependencyGraph()

    # First pass: collect all defined types
    defined_types = set()
    
    # Find all enum definitions
    pos = 0
    enums = []
    while True:
        enum_match = re.search(r'enum\s+(\w+)\s*{', content[pos:])
        if not enum_match:
            break
        enum_name = enum_match.group(1)
        defined_types.add(enum_name)
        enum_start = pos + enum_match.start()
        enum_content, next_pos = extract_block(content, enum_start)
        if next_pos == -1:
            break
        enums.append((enum_name, enum_content))
        pos = next_pos

    # Find all message definitions
    messages_and_deps = []
    pos = 0
    while True:
        message_match = re.search(r'\/\/[^\n]*\n\/\/[^\n]*\nmessage\s+(\w+)\s*{', content[pos:])
        if not message_match:
            break
        
        message_name = message_match.group(1)
        defined_types.add(message_name)
        message_start = pos + message_match.start()
        message_content, next_pos = extract_block(content, message_start)
        
        if next_pos == -1:
            break
            
        messages_and_deps.append((message_name, message_content))
        pos = next_pos

    # Build dependency graph
    message_dependencies = {}
    for message_name, message_content in messages_and_deps:
        dependencies = extract_dependencies(message_content)
        dependencies = {dep for dep in dependencies if dep in defined_types and dep != message_name}
        message_dependencies[message_name] = dependencies
        for dep in dependencies:
            dep_graph.add_dependency(message_name, dep)

    # Find strongly connected components (circular dependencies)
    components = dep_graph.get_strongly_connected_components()
    
    # Track which messages are part of circular dependencies
    messages_in_components = set()
    for component in components:
        messages_in_components.update(component)
        # Create a combined file for the component
        component_name = "_".join(sorted(component))
        output_file = os.path.join(output_dir, f"{component_name}.proto")
        
        # Get all dependencies for the component
        component_deps = set()
        for message in component:
            component_deps.update(message_dependencies[message])
        component_deps -= component  # Remove internal dependencies
        
        with open(output_file, 'w', encoding='utf-8') as f:
            if syntax_declaration:
                f.write(f"{syntax_declaration}\n\n")
            
            # Write external dependencies
            for dep in sorted(component_deps):
                f.write(f'import "{dep}.proto";\n')
            
            if component_deps:
                f.write("\n")
            
            # Write all messages in the component
            for message_name, message_content in messages_and_deps:
                if message_name in component:
                    f.write(message_content + "\n\n")

    # Process enums
    for enum_name, enum_content in enums:
        if enum_name not in messages_in_components:  # Skip if part of a component
            output_file = os.path.join(output_dir, f"{enum_name}.proto")
            with open(output_file, 'w', encoding='utf-8') as f:
                if syntax_declaration:
                    f.write(f"{syntax_declaration}\n\n")
                f.write(enum_content + "\n")

    # Process remaining messages (those not in circular dependencies)
    for message_name, message_content in messages_and_deps:
        if message_name not in messages_in_components:
            dependencies = message_dependencies[message_name]
            output_file = os.path.join(output_dir, f"{message_name}.proto")
            
            with open(output_file, 'w', encoding='utf-8') as f:
                if syntax_declaration:
                    f.write(f"{syntax_declaration}\n\n")
                
                # Create a stable list of dependencies for iteration
                deps_list = sorted(list(dependencies))
                for dep in deps_list:
                    dep_filename = dep
                    # If dependency is part of a component, use the component filename
                    for component in components:
                        if dep in component:
                            dep_filename = "_".join(sorted(component))
                            break
                    f.write(f'import "{dep_filename}.proto";\n')
                
                if dependencies:
                    f.write("\n")
                
                f.write(message_content + "\n")

    # Print information about combined files
    if components:
        print("\nCircular dependencies detected and combined into single files:")
        for component in components:
            print(f"Combined file: {output_dir}/{('_'.join(sorted(component)))}.proto")
            print("Contains messages:", " -> ".join(sorted(component)))
            print()

def main():
    import sys
    import argparse

    parser = argparse.ArgumentParser(description="Split a proto file into individual files based on dependencies")
    parser.add_argument("input_file", help="Path to the input proto file")
    parser.add_argument("-o", metavar="OUTPUT_DIR", default="proto_split", help="Output directory for split files (default: proto_split)")
    parser.add_argument("--clean", action="store_true", help="Remove the output directory before processing")

    args = parser.parse_args()
    input_file = args.input_file
    output_dir = args.o

    if args.clean and os.path.exists(output_dir):
        import shutil
        shutil.rmtree(output_dir)

    if not os.path.exists(input_file):
        print(f"Error: File '{input_file}' does not exist.")
        sys.exit(1)

    try:
        parse_proto_file(input_file, output_dir)
        print(f"Successfully split proto file into individual files in the '{output_dir}' directory.")
    except Exception as e:
        print(f"Error processing file: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
