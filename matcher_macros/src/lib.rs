use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr, Ident, Token};
use syn::parse::{Parse, ParseStream, Result};
use regex::Regex;

/// Procedural macro to generate a struct with captures from a query and implement `QueryExecutor`.
#[proc_macro]
pub fn generate_query_struct(input: TokenStream) -> TokenStream {
    // Parse the input tokens into our MacroInput structure
    let MacroInput { query, struct_name } = parse_macro_input!(input as MacroInput);
    
    // Extract the query string
    let query_string = query.value();

    // Use regex to find all captures in the query (e.g., @name, @message)
    let re = Regex::new(r"@(\w+)").unwrap();
    let captures: Vec<String> = re.captures_iter(&query_string)
        .map(|cap| cap[1].to_string())
        .collect();

    // Generate struct fields based on captures
    let fields = captures.iter().map(|field| {
        let ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            pub #ident: ::std::option::Option<::tree_sitter::Node<'tree>>,
        }
    }).collect::<Vec<_>>();

    // Generate identifiers for fields
    let capture_names: Vec<Ident> = captures.iter()
        .map(|field| Ident::new(field, proc_macro2::Span::call_site()))
        .collect();

    // Generate code to extract captures and populate the struct
    let expanded = quote! {
        #[derive(::std::fmt::Debug, ::std::default::Default)]
        pub struct #struct_name<'tree> {
            #(#fields)*
        }

        impl<'tree> crate::util::QueryExecutor<'tree> for #struct_name<'tree> {
            fn execute(node: ::tree_sitter::Node<'tree>, buffer: &crate::util::RawBuffer) -> ::std::vec::Vec<Self> {
                use crate::util::{create_query, ExtractText};
                
                let query = create_query(#query);

                let mut cursor = ::tree_sitter::QueryCursor::new();
                let mut captures = cursor.matches(&query, node, crate::util::RopeTextProvider::from(buffer));

                let mut result = ::std::vec::Vec::new();

                while let ::std::option::Option::Some(capture) = captures.next() {

                    let mut fields = Self::default();
                    #(
                        let idx = query.capture_index_for_name(stringify!(#capture_names)).unwrap();

                        if let ::std::option::Option::Some(node) = capture.nodes_for_capture_index(idx).next() {
                            fields.#capture_names = ::std::option::Option::Some(node); //.text(buffer).to_string());
                        }
                    )*

                    result.push(fields);
                }

                result
            }

        }
    };

    TokenStream::from(expanded)
}

struct MacroInput {
    query: LitStr,
    struct_name: Ident,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let query = input.parse()?;
        input.parse::<Token![,]>()?;
        let struct_name = input.parse()?;
        Ok(MacroInput { query, struct_name })
    }
} 