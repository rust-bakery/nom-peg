use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;


use super::parser::ParseTree;

// This compiles, and might be a good way to return an anonymous stucture with methods
// let a = {
//     struct Anon {}
//     impl Anon {
//         fn f(&self) -> u64 {
//             5
//         }
//     }
//     Anon {}
// };
//
// println!("{:?}", a.f());
//
// and this doesn't work, so Anon is hidden :)
// let b = Anon {};


impl ToTokens for ParseTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // `__input` passed implicitly
        tokens.append_all(match self {
            ParseTree::DefinitionList(definitions) => {
                quote! {
                    {
                        struct PEGParser {}
                        impl PEGParser {
                            #(
                                #definitions
                            )*
                        }
                        PEGParser {}

                        // struct PEGParser {
                        // #name: Box<Fn(CompleteStr) -> ::nom::IResult<CompleteStr, CompleteStr>>
                        // }
                        // PEGParser {
                        // #name: Box::new(|input| { do_parse!(input, #expr >> (#block)) })
                        // }
                        // |input: &str| do_parse!(input, #expr >> (#block))
                    }
                }
            }

            ParseTree::ParserDefinition(name, return_type, expr) => {
                let return_type = match return_type {
                    Some(return_type) => quote! { #return_type },
                    None => quote! { &'input str },
                };
                quote! {
                    fn #name<'input>(&self, input: &'input str) -> ::nom::IResult<&'input str, #return_type> {
                        do_parse!(input, __ret: #expr >> (__ret))
                    }
                    // named!(#name<CompleteStr, CompleteStr>, do_parse!(#expr >> (#block)));
                }
            }

            ParseTree::Capture(term, _ident) => {
                // nothing happens here, `Capture` is just an indicator wrapper
                // that is used in the `Sequence` code generation
                quote! {
                    #term
                }
            }

            ParseTree::NonTerminal(ident) => {
                quote! {
                    call!(|input| self.#ident(input))
                }
            }

            ParseTree::Sequence(seq, block) => {

                // check for captures in the sequence
                let capture_map: Vec<(bool, &Option<Ident>)> = seq.iter()
                    .map(|expr| {
                        match expr {
                            ParseTree::Capture(_, ident) => (true, ident),
                            _ => (false, &None),
                        }
                    })
                    .collect();

                let block_prelude = if !capture_map.iter().any(|x| x.0) {
                    // no captures, just use the original tuple
                    quote! { let result = __result; }

                } else {
                    // indices and names for all the named captures
                    let (indices, idents): (Vec<usize>, Vec<&Ident>) = capture_map.iter()
                        .enumerate()
                        .filter_map(|x| {
                            match x {
                                (index, (_, Some(ident))) => Some((index, ident)),
                                (_index, (_, &None)) => None,
                            }
                        })
                        .unzip();

                    // indices for the anonymous captures
                    let anon_indices: Vec<usize> = capture_map.iter()
                        .enumerate()
                        .filter_map(|x| {
                            match x {
                                (index, (true, None)) => Some(index),
                                (_index, (_, _)) => None,
                            }
                        })
                        .collect();

                    quote! {
                        let result = ( #( __result.#anon_indices ),* );
                        #(
                            let #idents = __result.#indices;
                        )*
                    }
                };

                let block = match block {
                    Some(block) => quote! { #block },
                    None => quote! { result },
                };

                quote! {
                    do_parse!(__result: tuple!(#(#seq),*) >> ( { #block_prelude #block } ))
                }
            }

            ParseTree::Empty => quote! {
                take!(0)
            },

            ParseTree::Terminal(term) => {
                quote! {
                    tag!(#term)
                }
            }

            ParseTree::Choice(choices) => {
                quote! {
                    alt!(#(#choices)|*)
                }
            }

            ParseTree::Many0(term) => {
                quote! {
                    many0!(#term)
                }
            }

            ParseTree::Many1(term) => {
                quote! {
                    many1!(#term)
                }
            }

            ParseTree::Optional(term) => {
                quote! {
                    opt!(#term)
                }
            }

            ParseTree::Peek(term) => {
                quote! {
                    peek!(#term)
                }
            }

            ParseTree::Not(term) => {
                quote! {
                    not!(#term)
                }
            }
        });
    }
}
