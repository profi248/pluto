extern crate proc_macro;

mod topics;
mod reject;

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
/// in the format string literal. This can be obtained via the `topic() -> String`
/// method on a leaf topic struct.
///
/// On leaf topics, another operator `=> S` can be added to specify the type `S`
/// of the initial request structure to this topic. This generates a function
/// called `message(&self) -> S` on the given topic, which returns `Default::default()`.
///
/// Leaf topic structs can be accessed via the generated `topics!` macro, where nested
/// topic identifiers are separated by `::` like enums. This saves the user
/// from needing to use all of the nested enums and variants which could look
/// messy.
///
/// [`Debug`], [`Default`] and [`Clone`] are automatically derived for each leaf topic struct.
///
/// Each leaf topic struct implements [`FromStr`](std::str::FromStr) which returns
/// `Ok(Self)` if the input string matches the topic string. This includes
/// additional arguments, where it matches alphanumeric characters in place of the
/// arguments. **NOTE:** that this uses the `regex` and `lazy_static` crates, so these
/// will need to be imported alongside this macro.
///
/// Each leaf topic struct `S` also implements `From<S> for Topic` (and hence `Into<Topic>`)
/// which converts it to a full path in terms of the root `Topic` enum.
///
/// The root `Topic` enum implements a method `from_topic(String) -> Option<Topic>`
/// that returns the topic for which the string argument matches, if it exists.
/// Note that this returns the path of the topic, not the leaf topic struct, due to
/// static type limitations.
///
/// Each nested topic enum implements [`Debug`], [`Hash`], [`Eq`], [`PartialEq`] and [`Clone`] by default.
/// Additional attributes can be added to individual leaf topic structs by adding them above
/// the leaf topic in the macro syntax.
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
/// generates code which includes the following:
/// ```
/// # use std::str::FromStr;
/// # #[derive(Default)] pub struct AuthNodeInit;
/// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
/// pub enum Topic {
///     Coordinator(CoordinatorTopic),
///     Node(NodeTopic),
/// }
///
/// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
/// pub enum CoordinatorTopic {
///     Auth,
/// }
///
/// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
/// pub enum NodeTopic {
///     Auth,
/// }
///
/// impl Topic {
///     pub fn from_topic(topic: String) -> Option<Topic> {
///         /* ... */
/// #       None
///     }
/// }
///
/// #[derive(Default, Debug, Clone)]
/// pub struct CoordinatorAuthTopic;
/// impl CoordinatorAuthTopic {
///     pub fn topic(&self) -> String {
///         "coordinator/auth".to_owned()
///     }
///     pub fn message(&self) -> AuthNodeInit {
///         Default::default()
///     }
/// }
/// impl From<CoordinatorAuthTopic> for Topic {
///     fn from(t: CoordinatorAuthTopic) -> Topic {
///         Topic::Coordinator(CoordinatorTopic::Auth)
///     }
/// }
/// impl FromStr for CoordinatorAuthTopic {
///     type Err = ();
///     /* ... */
///#    fn from_str(s: &str) -> Result<Self, Self::Err> {
///#         Err(())
///#     }
/// }
///
/// #[derive(Default, Debug, Clone)]
/// pub struct NodeAuthTopic;
/// impl NodeAuthTopic {
///     pub fn topic(&self, id: String) -> String {
///         format!("node/{id}/auth")
///     }
/// }
/// impl From<NodeAuthTopic> for Topic {
///     fn from(t: NodeAuthTopic) -> Topic {
///         Topic::Node(NodeTopic::Auth)
///     }
/// }
/// impl FromStr for NodeAuthTopic {
///     type Err = ();
///     /* ... */
///#    fn from_str(s: &str) -> Result<Self, Self::Err> {
///#         Err(())
///#     }
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

#[proc_macro_attribute]
pub fn reject(_attribute: TokenStream, function: TokenStream) -> TokenStream {
    let function = parse_macro_input!(function as reject::RejectFunction);

    function.expand()
}