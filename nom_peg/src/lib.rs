extern crate proc_macro;
extern crate proc_macro2;
use proc_macro2::TokenStream;

extern crate quote;
use quote::{quote, ToTokens, TokenStreamExt};

extern crate syn;
use syn::{
    parenthesized, parse::Parse, parse::ParseStream, parse_macro_input, token::Paren, Block, Ident,
    LitStr, Token,
};

// PEG paper: http://bford.info/pub/lang/peg.pdf

#[derive(Debug)]
enum ParseTree {
    Empty,
    Terminal(String),
    NonTerminal(Ident),
    // Grouping(Box<ParseTree>),
    Sequence(Vec<ParseTree>),
    Choice(Vec<ParseTree>),
    Many0(Box<ParseTree>),
    Many1(Box<ParseTree>),
    Optional(Box<ParseTree>),
    Peek(Box<ParseTree>),
    Not(Box<ParseTree>),

    Function(Ident, Box<ParseTree>, Option<Block>),
    FunctionList(Vec<ParseTree>)
}


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
            ParseTree::FunctionList(functions) => {
                quote! {
                    {
                        struct PEGParser {}
                        impl PEGParser {
                            // fn #name<'a>(&self, input: &'a str) -> ::nom::IResult<&'a str, &'a str> {
                            //     do_parse!(input, #expr >> (#block))
                            // }
                            #(
                                #functions
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
                    // named!(#name<CompleteStr, CompleteStr>, do_parse!(#expr >> (#block)));
                }
            }

            ParseTree::Function(name, expr, block) => {
                let block = match block {
                    Some(block) => quote! { #block },
                    None => quote! { parser_result },
                };

                quote! {
                    fn #name<'a>(&self, input: &'a str) -> ::nom::IResult<&'a str, &'a str> {
                        do_parse!(input, #expr >> (#block))
                    }
                    // named!(#name<CompleteStr, CompleteStr>, do_parse!(#expr >> (#block)));
                }
            }

            ParseTree::NonTerminal(ident) => {
                quote! {
                    call!(|input| self.#ident(input))
                }
            }

            ParseTree::Empty => quote! {
                take!(0)
            },

            ParseTree::Sequence(seq) => {
                quote! {
                    do_parse!(#(#seq >> )* ())
                    // tuple!(#(#seq),*)
                }
                // #( { #seq } );*
            }

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

            _ => quote! {
                unimplemented!()
            },
        });
    }
}

enum Prefix {
    Peek,
    Not,
}

enum Postfix {
    Optional,
    Many0,
    Many1,
}

fn parse_prefix(input: ParseStream) -> Option<Prefix> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![&]) {
        // Peek
        input.parse::<Token![&]>().unwrap(); // just skip past this
        Some(Prefix::Peek)
    } else if lookahead.peek(Token![!]) {
        // Not
        input.parse::<Token![!]>().unwrap(); // just skip past this
        Some(Prefix::Not)
    } else {
        // No prefix found
        None
    }
}

fn parse_postfix(input: ParseStream) -> Option<Postfix> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![?]) {
        // Optional
        input.parse::<Token![?]>().unwrap(); // just skip past this
        Some(Postfix::Optional)
    } else if lookahead.peek(Token![*]) {
        // Many0
        input.parse::<Token![*]>().unwrap(); // just skip past this
        Some(Postfix::Many0)
    } else if lookahead.peek(Token![+]) {
        // Many1
        input.parse::<Token![+]>().unwrap(); // just skip past this
        Some(Postfix::Many1)
    } else {
        // No postfix found
        None
    }
}

fn parse_element(input: ParseStream) -> syn::Result<ParseTree> {
    let prefix = parse_prefix(input);

    let lookahead = input.lookahead1();
    let mut parsed = if lookahead.peek(Ident) {
        // if there's an '=' sign following it's the start of a new nonterminal
        if input.peek2(Token![=]) {
            Err(input.error("Reached start of new nonterminal."))
        } else {
            // Non-Terminal / Indentifier
            Ok(ParseTree::NonTerminal(input.parse::<Ident>()?))
        }
    } else if lookahead.peek(LitStr) {
        // Terminal
        Ok(ParseTree::Terminal(input.parse::<LitStr>()?.value()))
    } else if lookahead.peek(Paren) {
        // Grouping
        // Get content of parens
        let content;
        parenthesized!(content in input);
        // and parse the content
        // Ok(ParseTree::Grouping(Box::new(content.parse::<ParseTree>()?)))
        Ok(parse_expression(&content)?)
    } else {
        Err(lookahead.error())
    };

    let postfix = parse_postfix(input);

    // process postfix
    parsed = parsed.and_then(|p| {
        Ok(match postfix {
            Some(Postfix::Optional) => ParseTree::Optional(Box::new(p)),
            Some(Postfix::Many0) => ParseTree::Many0(Box::new(p)),
            Some(Postfix::Many1) => ParseTree::Many1(Box::new(p)),
            None => p,
        })
    });

    // process prefix
    parsed.and_then(|p| {
        Ok(match prefix {
            Some(Prefix::Peek) => ParseTree::Peek(Box::new(p)),
            Some(Prefix::Not) => ParseTree::Not(Box::new(p)),
            None => p,
        })
    })
}

fn parse_sequence(input: ParseStream) -> syn::Result<ParseTree> {
    let mut expressions: Vec<ParseTree> = Vec::with_capacity(4);
    while !input.is_empty() {
        match parse_element(input) {
            Ok(e) => expressions.push(e),
            Err(_) => break,
        }
    }
    match expressions.len() {
        0 => Ok(ParseTree::Empty),
        1 => Ok(expressions.remove(0)),
        _ => Ok(ParseTree::Sequence(expressions)),
    }
}

fn parse_expression(input: ParseStream) -> syn::Result<ParseTree> {
    let mut expressions: Vec<ParseTree> = Vec::with_capacity(4);

    expressions.push(parse_sequence(input)?);
    while !input.is_empty() && input.peek(Token![|]) {
        input.parse::<Token![|]>()?; // just skip past this
        expressions.push(parse_sequence(input)?);
    }

    match expressions.len() {
        0 => Ok(ParseTree::Empty),
        1 => Ok(expressions.remove(0)),
        _ => Ok(ParseTree::Choice(expressions)),
    }
}

fn parse_nonterminal(input: ParseStream) -> syn::Result<ParseTree> {
    // parse name
    let name = input.parse::<Ident>()?;
    input.parse::<Token![=]>()?; // just skip past this

    // parse expression
    let expression = parse_expression(input)?;

    // Parse transformation code
    let block = if input.peek(Token![=>]) {
        input.parse::<Token![=>]>()?; // just skip past this
        Some(input.parse::<Block>()?)
    } else {
        None
    };

    // Final ast node
    Ok(ParseTree::Function(name, Box::new(expression), block))
}

impl Parse for ParseTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nonterminals: Vec<ParseTree> = Vec::with_capacity(4);

        nonterminals.push(parse_nonterminal(input)?);
        while !input.is_empty() {
            nonterminals.push(parse_nonterminal(input)?);
        }
        Ok(ParseTree::FunctionList(nonterminals))
    }
}

#[proc_macro]
pub fn peg_grammar(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parse_tree = parse_macro_input!(tokens as ParseTree);
    eprintln!("!! input: {:?}", parse_tree);

    parse_tree.into_token_stream().into()
}
