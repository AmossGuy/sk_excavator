use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{*, spanned::Spanned};

pub fn macro_main(input: DeriveInput) -> TokenStream {
	let display_body = match input.data {
		Data::Struct(struct_data) => struct_display_body(struct_data),
		Data::Enum(enum_data) => enum_display_body(enum_data),
		Data::Union(union_data) => {
			let span = union_data.union_token.span;
			quote_spanned!(span=> compile_error!("EditableData derive does not support unions"))
		},
	};
	
	let input_ident = input.ident;
	let input_ident_string = input_ident.to_string();
	quote! {
		#[automatically_derived]
		impl crate::formats::EditableData for #input_ident {
			fn struct_name(&self) -> &str {
				#input_ident_string
			}
			
			fn display(&mut self, mut renderer: impl crate::formats::EditableDataRenderer) {
				#display_body
			}
		}
	}
}

fn struct_display_body(struct_data: DataStruct) -> TokenStream {
	struct_data.fields.iter().zip(0..).map(|(field, i)| {
		let (field, field_name) = match &field.ident {
			Some(ident) => {
				(Member::Named(ident.clone()), ident.to_string())
			},
			None => {
				let index = Index { index: i, span: Span::call_site() };
				(Member::Unnamed(index), i.to_string())
			},
		};
		
		quote! {
			crate::formats::FieldDispatch::dispatch(&mut self.#field, &mut renderer, #field_name);
		}
	}).collect()
}

fn enum_display_body(enum_data: DataEnum) -> TokenStream {
	let variants = match enum_data.variants.iter().map(|variant| {
		let values = AttributeValues::parse(variant.attrs.iter())?;
		Ok((variant, values))
	}).collect::<syn::Result<Vec<_>>>() {
		Ok(stuff) => stuff,
		Err(e) => {
			return e.into_compile_error();
		},
	};
	
	let selected_arms = variants.iter().map(|(variant, _)| {
		let f_ignore = fields_ignore(&variant.fields);
		let variant_ident = &variant.ident;
		let variant_name = variant_ident.to_string();
		quote! {
			Self::#variant_ident #f_ignore => { #variant_name },
		}
	});
	
	let choices = variants.iter().map(|(variant, values)| {
		if values.skip {
			quote! {} // That's right, nothing
		} else {
			let f_ignore = fields_ignore(&variant.fields);
			let f_default = fields_default(&variant.fields);
			
			let variant_ident = &variant.ident;
			let variant_name = variant_ident.to_string();
			quote! {
				let is_selected = ::std::matches!(self, Self::#variant_ident #f_ignore);
				if contents.choice(#variant_name, is_selected) && !is_selected {
					*self = Self::#variant_ident #f_default;
				}
			}
		}
	});
	
	let recurse_arms = variants.iter().map(|(variant, values)| {
		if let Fields::Unnamed(unnamed) = &variant.fields && unnamed.unnamed.len() == 1 {
			let variant_ident = &variant.ident;
			let display_code = match values.skip {
				true => quote! {}, // That's right, nothing (again)
				false => quote! { crate::formats::EditableData::display(value, renderer); },
			};
			quote! {
				Self::#variant_ident(value) => { #display_code },
			}
		} else if let Fields::Unit = &variant.fields {
			let variant_ident = &variant.ident;
			quote! {
				Self::#variant_ident => {},
			}
		} else {
			let span = variant.span();
			quote_spanned! {span=>
				compile_error!("EditableData enum variants without `skip` must be either unit or one-element tuple")
			}
		}
	});
	
	quote! {
		let selected_text = match self {
			#(#selected_arms)*
		};
		
		renderer.dropdown("enum variant", selected_text, |mut contents| {
			use crate::formats::DropdownRenderer;
			#(#choices)*
		});
		
		match self {
			#(#recurse_arms)*
		}
	}
}

fn fields_ignore(fields: &Fields) -> TokenStream {
	match fields {
		Fields::Named(_) => quote! { {..} },
		Fields::Unnamed(_) => quote! { (..) },
		Fields::Unit => quote! {},
	}
}

fn fields_default(fields: &Fields) -> TokenStream {
	match fields {
		Fields::Named(named) => {
			let defaults = named.named.iter().map(|field| {
				let field_ident = field.ident.as_ref().expect("named field should have an identifier");
				quote! { #field_ident: ::std::default::Default::default() }
			});
			quote! { { #(#defaults),* } }
		},
		Fields::Unnamed(unnamed) => {
			let defaults = unnamed.unnamed.iter().map(|_field| {
				quote! { ::std::default::Default::default() }
			});
			quote! { (#(#defaults),*) }
		},
		Fields::Unit => quote! {},
	}
}

struct AttributeValues {
	skip: bool
}

impl AttributeValues {
	fn parse<'a>(attrs: impl Iterator<Item = &'a Attribute>) -> syn::Result<Self> {
		let mut this = Self {
			skip: false,
		};
		
		for attr in attrs {
			attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("skip") {
					this.skip = true;
					Ok(())
				} else {
					Err(meta.error("unrecognized `edit` attribute property"))
				}
			})?;
		}
		
		Ok(this)
	}
}
