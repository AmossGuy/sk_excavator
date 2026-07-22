use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Field, Fields, ItemStruct, Token, Type};
use syn::punctuated::{Pair, Punctuated};

pub fn unraw(input: ItemStruct) -> TokenStream {
	let mut new_struct = input.clone();
	new_struct.attrs.clear();
	
	let input_ident_name = input.ident.to_string();
	let Some(new_ident_name) = input_ident_name.strip_suffix("Raw") else {
		return quote! { compile_error!("struct name needs to end with `Raw`"); };
	};
	let new_ident = Ident::new(&new_ident_name, Span::call_site());
	new_struct.ident = new_ident.clone();
	
	let taken_fields = std::mem::replace(&mut new_struct.fields, Fields::Unit);
	new_struct.fields = parsify_buncha_fields(taken_fields);
	
	let number_of_fields = new_struct.fields.len();
	let field_name_arms = new_struct.fields.iter().enumerate().map(|(i, field)| {
		let name = field.ident.as_ref().map(|x| x.to_string()).unwrap_or_default();
		quote! { #i => Some(#name) }
	});
	let field_ref_arms = new_struct.fields.iter().enumerate().map(|(i, field)| {
		match &field.ident {
			Some(ident) => quote! { #i => Some(&self.#ident) },
			None => quote! { #i => Some(&self.#i) },
		}
	});
	let field_mut_arms = new_struct.fields.iter().enumerate().map(|(i, field)| {
		match &field.ident {
			Some(ident) => quote! { #i => Some(&mut self.#ident) },
			None => quote! { #i => Some(&mut self.#i) },
		}
	});
	
	quote! {
		#new_struct
		
		impl crate::formats::EditableStruct for #new_ident {
			fn struct_name(&self) -> &str {
				#new_ident_name
			}
			
			fn number_of_fields(&self) -> usize {
				#number_of_fields
			}
			
			fn field_name(&self, index: usize) -> Option<&str> {
				match index {
					#(#field_name_arms),*,
					_ => None,
				}
			}
			
			fn field_ref(&self, index: usize) -> Option<&dyn std::any::Any> {
				match index {
					#(#field_ref_arms),*,
					_ => None,
				}
			}
			
			fn field_mut(&mut self, index: usize) -> Option<&mut dyn std::any::Any> {
				match index {
					#(#field_mut_arms),*,
					_ => None,
				}
			}
		}
	}
}

fn parsify_buncha_fields(mut fields: Fields) -> Fields {
	match fields {
		Fields::Named(ref mut named) => parsify_punc_fields_thru(&mut named.named),
		Fields::Unnamed(ref mut unnamed) => parsify_punc_fields_thru(&mut unnamed.unnamed),
		Fields::Unit => {},
	}
	fields
}

fn parsify_punc_fields_thru(punc: &mut Punctuated<Field, Token![,]>) {
	*punc = parsify_punc_fields(punc);
}

fn parsify_punc_fields(punc: &Punctuated<Field, Token![,]>) -> Punctuated<Field, Token![,]> {
	punc.pairs().filter_map(|pair| {
		let (field, p) = pair.into_tuple();
		parsify_field(field).map(|field| {
			Pair::new(field, p.copied())
		})
	}).collect()
}

fn parsify_field(field: &Field) -> Option<Field> {
	let mut field = field.clone();
	let attrs = std::mem::take(&mut field.attrs);
	
	let mut skip = false;
	for attr in attrs {
		if attr.meta.path().is_ident(&Ident::new("unraw", Span::call_site())) {
			attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("skip") {
					skip = true;
					Ok(())
				} else {
					Err(meta.error("unrecognized unraw attribute"))
				}
			}).unwrap(); // bad error handling alert
		}
	}
	if skip { return None; }
	
	let ty = &field.ty;
	let parsified = quote! {
		<#ty as crate::parse_new::RawField>::Parsed
	};
	field.ty = Type::Verbatim(parsified);
	Some(field)
}
