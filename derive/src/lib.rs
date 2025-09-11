use proc_macro::TokenStream;
// use syn::parse_macro_input;

#[proc_macro_derive(Mnemonic, attributes(Instruction))]
pub fn mnemonic_macro(input: TokenStream) -> TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    "fn test() -> u32 { 42 }".parse().unwrap()
}
