mod proc_unraw_struct;
use syn::{parse_macro_input, ItemStruct};

use proc_macro::TokenStream;

#[proc_macro_derive(ProcUnrawStruct, attributes(unraw))]
pub fn proc_unraw_struct(item: TokenStream) -> TokenStream {
	let item = parse_macro_input!(item as ItemStruct);
	let expanded = proc_unraw_struct::unraw(item);
	TokenStream::from(expanded)
}
