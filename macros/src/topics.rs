use proc_macro2::TokenStream;

use syn::{
    Attribute, Ident, LitStr, Type,
    Token, token,
    braced,
    punctuated::Punctuated,
    parse::{ Parse, ParseStream },
};

use quote::{ quote, format_ident };

pub struct RootTopic {
    topics: Punctuated<Topic, Token![,]>
}

pub enum Topic {
    Leaf(LeafTopic),
    Nested(NestedTopic),
}

pub struct LeafTopic {
    attributes: Vec<Attribute>,
    name: Ident,
    topic: TopicString,
    message_type: Option<Type>
}

pub enum TopicString {
    Exact(LitStr),
    WithParams {
        format_string: LitStr,
        params: Vec<Ident>,
    }
}

pub struct NestedTopic {
    name: Ident,
    _brace_token: token::Brace,
    topics: Punctuated<Topic, Token![,]>
}

impl Parse for RootTopic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            topics: input.parse_terminated(Topic::parse)?
        })
    }
}

impl Parse for Topic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![ # ]) || input.peek2(Token![ -> ]) {
            Ok(Topic::Leaf(LeafTopic::parse(input)?))
        }
        else {
            Ok(Topic::Nested(NestedTopic::parse(input)?))
        }
    }
}

impl Parse for LeafTopic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const MESSAGE_ATTRIBUTE: &'static str = "message";

        let attributes = input.call(Attribute::parse_outer)?;
        let name = input.parse()?;

        input.parse::<Token![ -> ]>()?;

        let topic = input.parse()?;

        let message_type = input.peek(Token![ => ])
            .then(|| {
                input.parse::<Token![ => ]>()?;

                input.parse::<Type>()
            })
            .transpose()?;

        Ok(Self {
            attributes,
            name,
            topic,
            message_type,
        })
    }
}

fn parse_message_attribute(input: ParseStream) -> syn::Result<Type> {
    let _equals: Token![ = ] = input.parse()?;
    input.parse()
}

impl Parse for TopicString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal: LitStr = input.parse()?;

        let string = literal.value();

        if !string.contains('}') || !string.contains('{') {
            return Ok(Self::Exact(literal));
        }

        use regex::Regex;

        lazy_static::lazy_static! {
            static ref REGEX: Regex = Regex::new(r"\{([[:alnum:]]+)\}").unwrap();
        }

        let params: Vec<Ident> = REGEX.captures_iter(&string)
            .map(|c| Ident::new(c.get(1).unwrap().as_str(), literal.span()))
            .collect();

        Ok(Self::WithParams {
            format_string: literal,
            params,
        })
    }
}

impl Parse for NestedTopic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            _brace_token: braced!(content in input),
            topics: content.parse_terminated(Topic::parse)?,
        })
    }
}

pub fn expand_root(root: RootTopic) -> TokenStream {
    let mut enum_tokens = TokenStream::new();
    let mut impl_tokens = TokenStream::new();
    let mut macro_tokens = TokenStream::new();

    for topic in root.topics.iter() {
        match topic {
            Topic::Leaf(leaf) => {
                expand_leaf(leaf, &mut enum_tokens, &mut impl_tokens, &[], &mut macro_tokens);
            }
            Topic::Nested(nested) => {
                expand_nested(nested, &mut enum_tokens, &mut impl_tokens, &[], &mut macro_tokens);
            }
        }
    }

    quote! {
        macro_rules! topic {
            #macro_tokens
        }

        pub(crate) use topic;

        pub enum Topic {
            #enum_tokens
        }

        #impl_tokens
    }
}

fn expand_nested(nested: &NestedTopic, enum_tokens: &mut TokenStream, impl_tokens: &mut TokenStream, context: &[String], macro_tokens: &mut TokenStream) {
    let context = {
        let mut v = Vec::new();

        v.extend_from_slice(context);
        v.push(nested.name.to_string());

        v
    };


    let variant_name = nested.name.clone();
    let enum_name = format_ident!("{}Topic", context.join(""));

    enum_tokens.extend([quote! {
        #variant_name(#enum_name),
    }]);

    let mut inner_tokens = TokenStream::new();

    for topics in nested.topics.iter() {
        match topics {
            Topic::Leaf(leaf) => {
                expand_leaf(leaf, &mut inner_tokens, impl_tokens, &context, macro_tokens);
            }
            Topic::Nested(nested) => {
                expand_nested(nested, &mut inner_tokens, impl_tokens, &context, macro_tokens);
            }
        }
    }

    impl_tokens.extend([quote! {
        pub enum #enum_name {
            #inner_tokens
        }
    }]);
}

fn expand_leaf(leaf: &LeafTopic, enum_tokens: &mut TokenStream, impl_tokens: &mut TokenStream, context: &[String], macro_tokens: &mut TokenStream) {
    use std::str::FromStr;

    let variant_name = leaf.name.clone();
    let struct_name = format_ident!("{}{}Topic", context.join(""), leaf.name);
    let macro_path = TokenStream::from_str(&format!("{}::{}", context.join("::"), leaf.name)).unwrap();
    let attributes = &leaf.attributes;

    let (topic_string, params) = match &leaf.topic {
        TopicString::Exact(s) => (
            quote! { #s.to_owned() },
            TokenStream::new()
        ),
        TopicString::WithParams { format_string, params } => {
            let params = params.iter().map(|i| quote!(, #i: String )).collect();

            (quote!( format!(#format_string) ), params)
        }
    };

    let message_function = match &leaf.message_type {
        Some(ty) => quote! {
            pub fn message(&self) -> #ty {
                Default::default()
            }
        },
        None => TokenStream::new(),
    };

    impl_tokens.extend([quote! {
        #(#attributes)*
        #[derive(Default)]
        pub struct #struct_name;

        impl #struct_name {
            pub fn topic(&self #params) -> String {
                #topic_string
            }

            #message_function
        }
    }]);

    enum_tokens.extend([quote! {
        #variant_name,
    }]);

    macro_tokens.extend([quote! {
        (#macro_path) => { #struct_name };
    }]);
}
