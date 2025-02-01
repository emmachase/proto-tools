use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Ident};
use syn::parse::{Parse, ParseStream, Result};
use regex::Regex;

/// Procedural macro to generate a struct with captures from a query and implement `QueryExecutor`.
#[proc_macro]
pub fn tree_sitter_query(input: TokenStream) -> TokenStream {
    // Parse the input tokens into our MacroInputs structure
    let MacroInputs { inputs } = parse_macro_input!(input as MacroInputs);
    
    let mut expanded = quote! {};
    
    for MacroInput { query, struct_name, .. } in inputs {
        // Parse the query string
        let query_string = query.value();
    
        // Extract the capture names from the query string
        let re = Regex::new(r"@(\w+)").unwrap();
        let captures: Vec<String> = re.captures_iter(&query_string)
            .map(|cap| cap[1].to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
    
        // Generate the fields for the struct
        let fields = captures.iter().map(|field| {
            let ident = Ident::new(field, proc_macro2::Span::call_site());
            quote! {
                pub #ident: ::std::option::Option<::tree_sitter::Node<'tree>>,
            }
        }).collect::<Vec<_>>();
    
        // Generate identifiers for the capture names
        let capture_names: Vec<Ident> = captures.iter()
            .map(|field| Ident::new(field, proc_macro2::Span::call_site()))
            .collect();
    
        expanded.extend(quote! {
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
        });
    }
    
    TokenStream::from(expanded)
}

struct MacroInput {
    struct_name: Ident,
    _paren_token: syn::token::Paren,
    query: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(MacroInput { 
            struct_name: input.parse()?, 
            _paren_token: syn::parenthesized!(content in input),
            query: content.parse()?,
        })
    }
}

struct MacroInputs {
    inputs: Vec<MacroInput>,
}

impl Parse for MacroInputs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inputs = Vec::new();
        while !input.is_empty() {
            inputs.push(input.parse()?);
        }
        Ok(MacroInputs { inputs })
    }
}
