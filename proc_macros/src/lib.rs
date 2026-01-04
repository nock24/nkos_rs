use proc_macro::TokenStream;

mod parse;
mod expand;

#[proc_macro]
pub fn sector_layout(input: TokenStream) -> TokenStream {
    let layout = syn::parse_macro_input!(input as parse::SectorLayout);
    expand::expand_layout(layout).into()
}
