use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::*;

pub fn macro_main(input: DeriveInput) -> TokenStream {
	let methods = match input.data {
		Data::Struct(struct_data) => struct_methods(&struct_data, &input.ident),
		Data::Enum(enum_data) => enum_methods(&enum_data, &input.ident),
		Data::Union(union_data) => {
			let span = union_data.union_token.span;
			return quote_spanned! {span=>
				compile_error!("EditableData derive does not support unions");
			};
		},
	};
	
	let Methods { field_count, field_name, field_ref, errors } = methods;
	let input_ident = input.ident;
	let input_ident_string = input_ident.to_string();
	
	let errors = errors.into_iter()
		.map(|e| e.into_compile_error())
		.collect::<Vec<_>>();
	
	quote! {
		#(#errors)*
		
		#[automatically_derived]
		impl crate::formats::common::EditableData for #input_ident {
			fn struct_name(&self) -> &str {
				#input_ident_string
			}
			
			fn field_count(&self) -> usize { #field_count }
			fn field_name(&self, index: usize) -> &str { #field_name }
			fn field_ref(&self, index: usize) -> crate::formats::common::FieldRef<'_> { #field_ref }
			
			/*
			fn variant_count(&self) -> usize {
				todo!()
			}
			
			fn variant_name(&self, index: usize) -> &str {
				todo!()
			}
			
			fn variant_current(&self) -> usize {
				todo!()
			}
			*/
		}
	}
}

struct Methods {
	field_count: TokenStream,
	field_name: TokenStream,
	field_ref: TokenStream,
	
	errors: Vec<syn::Error>,
}

fn struct_methods(struct_data: &DataStruct, struct_ident: &Ident) -> Methods {
	let mut errors = Vec::<syn::Error>::new();
	
	let fields = struct_data.fields.iter().enumerate().filter_map(|(i, field)| {
		let values = AttributeValues::parse(field.attrs.iter(), &mut errors);
		if values.skip {
			None
		} else {
			Some((i, field, values))
		}
	}).collect::<Vec<_>>();
	
	let struct_name = struct_ident.to_string();
	let field_count = fields.len();
	
	let field_name_arms = fields.iter().map(|(i, field, _values)| {
		let field_name = match &field.ident {
			Some(ident) => ident.to_string(),
			None => i.to_string(),
		};
		quote! {
			#i => #field_name,
		}
	});
	
	let field_ref_arms = fields.iter().map(|(i, field, _values)| {
		let access = match &field.ident {
			Some(ident) => quote! { self.#ident },
			None => quote! { self.#i },
		};
		quote! {
			#i => crate::formats::common::FieldRef::from(&#access),
		}
	});
	
	Methods {
		field_count: quote! {
			#field_count
		},
		field_name: quote! {
			match index {
				#(#field_name_arms)*
				_ => panic!("`{}::field_name` called with out-of-range index", #struct_name),
			}
		},
		field_ref: quote! {
			match index {
				#(#field_ref_arms)*
				_ => panic!("`{}::field_ref` called with out-of-range index", #struct_name),
			}
		},
		
		errors,
	}
}

fn enum_methods(enum_data: &DataEnum, struct_ident: &Ident) -> Methods {
	todo!("enum_methods")
}

struct AttributeValues {
	skip: bool,
}

impl AttributeValues {
	fn parse<'a>(attrs: impl Iterator<Item = &'a Attribute>, errors: &mut Vec<syn::Error>) -> Self {
		let mut this = Self {
			skip: false,
		};
		
		for attr in attrs {
			if attr.path().is_ident("edit") {
				let result = attr.parse_nested_meta(|meta| {
					if meta.path.is_ident("skip") {
						this.skip = true;
						Ok(())
					} else {
						Err(meta.error("unrecognized `edit` attribute property"))
					}
				});
				
				if let Err(e) = result {
					errors.push(e);
				}
			}
		}
		
		this
	}
}
