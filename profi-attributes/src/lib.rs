use proc_macro::{TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn profile(_: TokenStream, items: TokenStream) -> TokenStream {
    let mut items = items.into_iter().collect::<Vec<_>>();

    let Some(TokenTree::Group(body)) = items.pop() else {
        return r#"compile_error!("Expected function body");"#.parse().unwrap();
    };

    let cr = match proc_macro_crate::crate_name("profi").unwrap() {
        proc_macro_crate::FoundCrate::Itself => std::borrow::Cow::Borrowed("profi"),
        proc_macro_crate::FoundCrate::Name(n) => std::borrow::Cow::Owned(n),
    };
    let profile = {
        use proc_macro::{Delimiter as D, Group, Ident, Punct, Spacing as S, Span};
        
        [
            TokenTree::Punct(Punct::new(':', S::Joint)),
            TokenTree::Punct(Punct::new(':', S::Alone)),
            TokenTree::Ident(Ident::new(&cr, Span::call_site())),
            TokenTree::Punct(Punct::new(':', S::Joint)),
            TokenTree::Punct(Punct::new(':', S::Alone)),
            TokenTree::Ident(Ident::new("prof", Span::call_site())),
            TokenTree::Punct(Punct::new('!', S::Alone)),
            TokenTree::Group(Group::new(D::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new(';', S::Alone)),
            TokenTree::Group(body),
        ]
    };
    let tree = TokenTree::from(proc_macro::Group::new(
        proc_macro::Delimiter::Brace,
        TokenStream::from_iter(profile),
    ));
    items.push(tree);

    TokenStream::from_iter(items)
}

#[proc_macro_attribute]
pub fn main(_: TokenStream, items: TokenStream) -> TokenStream {
    let mut items = items.into_iter().collect::<Vec<_>>();

    let Some(TokenTree::Group(body)) = items.pop() else {
        return r#"compile_error!("Expected function body");"#.parse().unwrap();
    };

    let cr = match proc_macro_crate::crate_name("profi").unwrap() {
        proc_macro_crate::FoundCrate::Itself => std::borrow::Cow::Borrowed("profi"),
        proc_macro_crate::FoundCrate::Name(n) => std::borrow::Cow::Owned(n),
    };
    let profile = {
        use proc_macro::{Delimiter as D, Group, Ident, Punct, Spacing as S, Span};
        
        [
            TokenTree::Punct(Punct::new(':', S::Joint)),
            TokenTree::Punct(Punct::new(':', S::Alone)),
            TokenTree::Ident(Ident::new(&cr, Span::call_site())),
            TokenTree::Punct(Punct::new(':', S::Joint)),
            TokenTree::Punct(Punct::new(':', S::Alone)),
            TokenTree::Ident(Ident::new("print_on_exit", Span::call_site())),
            TokenTree::Punct(Punct::new('!', S::Alone)),
            TokenTree::Group(Group::new(D::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new(';', S::Alone)),
            TokenTree::Group(body),
        ]
    };
    let tree = TokenTree::from(proc_macro::Group::new(
        proc_macro::Delimiter::Brace,
        TokenStream::from_iter(profile),
    ));
    items.push(tree);

    TokenStream::from_iter(items)
}
