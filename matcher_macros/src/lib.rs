use proc_macro::TokenStream;
use quote::{format_ident, quote};
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

/// Derive macro for DebugWithName that uses children's DebugWithName implementations
#[proc_macro_derive(DebugWithName)]
pub fn derive_debug_with_name(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a DeriveInput
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        syn::Data::Struct(data) => {
            // Add a check for DebugWithName trait bound
            let field_types = data.fields.iter().map(|field| &field.ty);
            let trait_check = quote! {
                const _: () = {
                    trait AssertDebugWithName {
                        fn assert_debug_with_name() where Self: DebugWithName {}
                    }
                    impl<T: DebugWithName> AssertDebugWithName for T {}
                    
                    fn assert_all() {
                        #(<#field_types as AssertDebugWithName>::assert_debug_with_name();)*
                    }
                };
            };

            let fields = data.fields.iter().map(|field| {
                let field_name = &field.ident;
                quote! {
                    format!("{}: {}", stringify!(#field_name), self.#field_name.debug_with_name(db))
                }
            });

            quote! {
                #trait_check

                impl DebugWithName for #name {
                    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
                        let fields = vec![#(#fields),*];
                        format!("{} {{ {} }}", stringify!(#name), fields.join(", "))
                    }
                }

                impl DebugWithName for &#name {
                    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
                        (*self).debug_with_name(db)
                    }
                }
            }
        },
        syn::Data::Enum(data) => {

            let variants = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unit => {
                        quote! {
                            #name::#variant_name => stringify!(#variant_name).to_string()
                        }
                    },
                    syn::Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let field_prints = fields.named.iter().map(|f| {
                            let fname = &f.ident;
                            quote! {
                                format!("{}: {}", stringify!(#fname), #fname.debug_with_name(db))
                            }
                        });
                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                let fields = vec![#(#field_prints),*];
                                format!("{} {{ {} }}", stringify!(#variant_name), fields.join(", "))
                            }
                        }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let field_names = (0..fields.unnamed.len())
                            .map(|i| format_ident!("field{}", i));
                        let field_prints = (0..fields.unnamed.len()).map(|i| {
                            let fname = format_ident!("field{}", i);
                            quote! {
                                #fname.debug_with_name(db)
                            }
                        });
                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                let fields = vec![#(#field_prints),*];
                                format!("{}({})", stringify!(#variant_name), fields.join(", "))
                            }
                        }
                    }
                }
            });

            quote! {
                impl DebugWithName for #name {
                    fn debug_with_name(&self, db: &ProtoDatabase) -> String {
                        match self {
                            #(#variants),*
                        }
                    }
                }
            }
        },
        syn::Data::Union(_) => {
            let err = syn::Error::new_spanned(
                name.clone(),
                "DebugWithName cannot be derived for unions"
            );
            return TokenStream::from(err.to_compile_error());
        }
    };

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
