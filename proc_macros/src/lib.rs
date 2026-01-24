use proc_macro::TokenStream;

mod expand;
mod parse;

#[proc_macro]
pub fn sector_layout(input: TokenStream) -> TokenStream {
    let layout = syn::parse_macro_input!(input as parse::SectorLayout);
    expand::expand_layout(layout).into()
}

#[proc_macro]
pub fn cmd_try_froms(input: TokenStream) -> TokenStream {
    let cmd_try_froms = syn::parse_macro_input!(input as parse::CmdTryFroms);
    expand::expand_cmd_try_froms(cmd_try_froms).into()
}
