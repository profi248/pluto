extern crate proc_macro;

mod topics;

use proc_macro::TokenStream;

use syn::{ parse_macro_input, LitStr };

const WORD_COUNT: usize = 2048;

#[proc_macro]
pub fn wordlist(tokens: TokenStream) -> TokenStream {
    let file_path: LitStr = parse_macro_input!(tokens);

    let text = std::fs::read_to_string(file_path.value())
        .expect("Failed to open wordlist file");

    let words: Vec<&str> = text.split_whitespace().collect();

    if words.len() != WORD_COUNT {
        panic!("Word list does not match expected word count. Expected {}, got {}.", WORD_COUNT, words.len());
    }

    (quote::quote!{
        &[
            #(#words),*
        ]
    }).into()
}

#[proc_macro]
pub fn define_topics(tokens: TokenStream) -> TokenStream {
    let topics = parse_macro_input!(tokens as topics::RootTopic);

    topics::expand_root(topics).into()
}
