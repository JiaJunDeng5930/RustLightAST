use quote::ToTokens;
use rustlightast::{
    parse_rust_source, ForeignMember, ForeignSafety, Initializer, Item, RustCodeGenerator,
    TraitMember, Type, TypeBound,
};

fn assert_same_tokens(source: &str, printed: &str) {
    let tokens = |source: &str| {
        syn::parse_file(source)
            .unwrap()
            .to_token_stream()
            .to_string()
    };
    assert_eq!(tokens(printed), tokens(source), "{printed}");
}

#[test]
fn extern_members_expose_written_signatures_and_safety() {
    let source = r#"
        #[link(name = "c")]
        unsafe extern "C" {
            #![allow(unused)]
            pub safe fn length(#[allow(unused)] text: *const u8) -> usize;
            pub unsafe fn print(format: *const u8, ...) -> i32;
            pub safe static CODE: i32;
            pub unsafe static mut COUNTER: usize;
            type Handle;
            members!();
        }
        pub static mut POINTER: *const u8 = core::ptr::null();
    "#;
    let module = parse_rust_source(source, "ffi").unwrap();
    let [Item::Foreign(block), Item::Static(pointer)] = module.items.as_slice() else {
        panic!("expected foreign block and static");
    };
    assert!(block.unsafety);
    let [ForeignMember::Function(length), ForeignMember::Function(print), ForeignMember::Static(code), ForeignMember::Static(counter), ForeignMember::Type(handle), ForeignMember::Macro(mac)] =
        block.members.as_slice()
    else {
        panic!("expected structural extern members: {:?}", block.members)
    };
    assert_eq!(length.name, "length");
    assert!(length.safe);
    assert!(matches!(length.params[0].param.ty, Type::RawPointer(..)));
    assert_eq!(length.params[0].attrs[0].unqualified_name(), Some("allow"));
    assert!(print.qualifiers.unsafety);
    assert!(print.variadic.is_some());
    assert_eq!(code.safety, ForeignSafety::Safe);
    assert_eq!(counter.safety, ForeignSafety::Unsafe);
    assert!(counter.mutable);
    assert_eq!(handle.name, "Handle");
    assert_eq!(mac.invocation.path, ["members"]);
    assert!(pointer.mutable);
    assert!(matches!(pointer.value, Initializer::Expr(_)));

    assert_same_tokens(
        source,
        &RustCodeGenerator::new().generate_module_code(&module),
    );
}
