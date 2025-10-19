mod texture_atlas;

use darling::export::syn::parse_macro_input;

#[proc_macro]
pub fn texture_atlas(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as texture_atlas::AtlasOpts);
    input.generate().into()
}
