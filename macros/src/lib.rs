extern crate proc_macro;

mod topics;

use proc_macro::TokenStream;

use syn::{ parse_macro_input, LitStr };

const WORD_COUNT: usize = 2048;

#[proc_macro]
pub fn wordlist(tokens: TokenStream) -> TokenStream {
    let file_path: LitStr = parse_macro_input!(tokens);

    // todo: fix this using relative paths to the source span.
    // currently not doable without nightly.
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

/// Defines a hierarchy of topics.
///
/// Identifiers can either open a braced scope to declare nesting, or
/// use the `->` operator to declare the topic string literal. This supports
/// rust formatting, and generates string parameters based on the identifiers
/// in the format string literal.
///
/// Topics can be accessed via the generated `topics!` macro, where nested
/// topic identifiers are separated by `::` like enums. This saves the user
/// from needing to use all of the nested enums and variants which could look
/// messy.
///
/// On leaf topics, another operator `=> S` can be added to specify the type `S`
/// of the initial request structure to this topic. This generates a function
/// called `message(&self) -> S` on the given topic, which returns `Default::default()`.
///
/// # Example
/// ```
/// # use pluto_macros::define_topics;
/// # #[derive(Default)] pub struct AuthNodeInit;
/// define_topics! {
///     Coordinator {
///         Auth -> "coordinator/auth" => AuthNodeInit
///     },
///     Node {
///         Auth -> "node/{id}/auth"
///     }
/// }
/// ```
/// generates the following code:
/// ```
/// # #[derive(Default)] pub struct AuthNodeInit;
/// pub enum Topic {
///     Coordinator(CoordinatorTopic),
///     Node(NodeTopic),
/// }
/// #[derive(Default)]
/// pub struct CoordinatorAuthTopic;
/// impl CoordinatorAuthTopic {
///     pub fn topic(&self) -> String {
///         "coordinator/auth".to_owned()
///     }
///     pub fn message(&self) -> AuthNodeInit {
///         Default::default()
///     }
/// }
/// pub enum CoordinatorTopic {
///     Auth,
/// }
/// #[derive(Default)]
/// pub struct NodeAuthTopic;
/// impl NodeAuthTopic {
///     pub fn topic(&self, id: String) -> String {
///         format!("node/{id}/auth")
///     }
/// }
/// pub enum NodeTopic {
///     Auth,
/// }
///
/// macro_rules! topic {
///     (Coordinator::Auth) => { CoordinatorAuthTopic };
///     (Node::Auth) => { NodeAuthTopic };
/// }
/// ```
#[proc_macro]
pub fn define_topics(tokens: TokenStream) -> TokenStream {
    let topics = parse_macro_input!(tokens as topics::RootTopic);

    topics::expand_root(topics).into()
}
