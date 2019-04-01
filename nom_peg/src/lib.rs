extern crate proc_macro;
extern crate proc_macro2;

extern crate syn;
use syn::parse_macro_input;

extern crate quote;
use quote::ToTokens;


// PEG paper: http://bford.info/pub/lang/peg.pdf

mod parser;
mod codegen;

// TODO: rewrite AST enum to facilitate `impl Parse` for each variant
#[proc_macro]
pub fn grammar(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parse_tree = parse_macro_input!(tokens as parser::ParseTree);
    // eprintln!("!! input: {:?}", parse_tree);

    parse_tree.into_token_stream().into()
}
