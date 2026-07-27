mod derive_editable_data;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EditableData, attributes(edit))]
pub fn derive_editable_data(item: TokenStream) -> TokenStream {
	let item = parse_macro_input!(item as DeriveInput);
	let expanded = derive_editable_data::macro_main(item);
	TokenStream::from(expanded)
}
