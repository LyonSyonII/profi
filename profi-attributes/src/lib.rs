use proc_macro::{TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn profile(_: TokenStream, items: TokenStream) -> TokenStream {
    let mut items = items.into_iter().collect::<Vec<_>>();
    
    let Some(TokenTree::Group(body)) = items.pop() else {
        return r#"compile_error!("Expected function body");"#.parse().unwrap();
    };
    let Some(TokenTree::Ident(name)) = items.get(1) else {
        return r#"compile_error!("Expected function name");"#.parse().unwrap();
    };
    
    let cr = match proc_macro_crate::crate_name("profi").unwrap() {
        proc_macro_crate::FoundCrate::Itself => std::borrow::Cow::Borrowed("profi"),
        proc_macro_crate::FoundCrate::Name(n) => std::borrow::Cow::Owned(n),
    };
    let profile = {
        let _crate = proc_macro::Ident::new(&cr, proc_macro::Span::call_site());
        let punct1 = proc_macro::Punct::new(':', proc_macro::Spacing::Joint);
        let punct2 = proc_macro::Punct::new(':', proc_macro::Spacing::Alone);
        let _macro = proc_macro::Ident::new("prof", proc_macro::Span::call_site());
        let punct3 = proc_macro::Punct::new('!', proc_macro::Spacing::Alone);
        let group = proc_macro::Group::new(proc_macro::Delimiter::Parenthesis, TokenStream::from_iter([
            proc_macro::TokenTree::Ident(name.to_owned())
        ]));
        let punct4 = proc_macro::Punct::new(';', proc_macro::Spacing::Alone);
        [
            proc_macro::TokenTree::Ident(_crate), 
            proc_macro::TokenTree::Punct(punct1), 
            proc_macro::TokenTree::Punct(punct2), 
            proc_macro::TokenTree::Ident(_macro), 
            proc_macro::TokenTree::Punct(punct3), 
            proc_macro::TokenTree::Group(group), 
            proc_macro::TokenTree::Punct(punct4),
            proc_macro::TokenTree::Group(body)
        ]
    };
    let tree = TokenTree::from(proc_macro::Group::new(proc_macro::Delimiter::Brace, TokenStream::from_iter(profile)));
    items.push(tree);
    
    TokenStream::from_iter(items)
}
