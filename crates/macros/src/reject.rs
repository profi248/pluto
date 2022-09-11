use proc_macro::TokenStream;

use syn::{
    Token, parenthesized,
    parse::{ Parse, ParseStream },
    punctuated::Punctuated,
    Attribute, Visibility, Ident, Type, Block,
};

#[derive(Clone)]
pub struct FnArg {
    pub mutability: Option<Token![mut]>,
    pub ident: Ident,
    pub ty: Type,
}

impl Parse for FnArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mutability = input.peek(Token![mut])
            .then(|| input.parse())
            .transpose()?;

        let ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let ty = input.parse()?;

        Ok(Self {
            mutability,
            ident,
            ty,
        })
    }
}

impl quote::ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            mutability,
            ident,
            ty,
        } = self;

        tokens.extend([quote::quote! {
            #mutability #ident: #ty
        }])
    }
}

pub struct RejectFunction {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub identifier: Ident,
    pub arguments: Punctuated<FnArg, Token![,]>,
    pub return_type: Type,
    pub inner: Block,
}

impl Parse for RejectFunction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let visibility = input.parse()?;
        let _async_keyword: Token![async] = input.parse()?;
        let _fn_keyword: Token![fn] = input.parse()?;
        let identifier = input.parse()?;

        let arguments;
        let _parentheses = parenthesized!(arguments in input);

        let _right_arrow: Token![->] = input.parse()?;
        let return_type = input.parse()?;

        let inner = input.parse()?;

        Ok(Self {
            attributes,
            visibility,
            identifier,
            arguments: arguments.parse_terminated(FnArg::parse)?,
            return_type,
            inner
        })
    }
}

impl RejectFunction {
    pub fn expand(self) -> TokenStream {
        let Self {
            attributes,
            visibility,
            identifier,
            arguments,
            return_type,
            inner,
        } = self;

        let call: Vec<Ident> = arguments.iter()
            .map(|a| a.ident.clone())
            .collect();

        let arguments: Vec<FnArg> = arguments.iter().cloned().collect();

        quote::quote!{
            #(#attributes)*
            #visibility async fn #identifier ( #(#arguments),* ) -> std::result::Result<warp::reply::Response, std::convert::Infallible> {
                use warp::Reply;

                async fn _inner( #(#arguments),* ) -> #return_type {
                    #inner
                }

                Ok(match _inner( #(#call),*).await {
                    Ok(r) => r.into_response(),
                    Err(r) => r.into_response(),
                })
            }
        }.into()
    }
}
