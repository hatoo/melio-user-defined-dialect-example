use attribute::{DenseElementsAttribute, FlatSymbolRefAttribute, FloatAttribute, IntegerAttribute};
use melior::{
    dialect::{arith, func, DialectHandle, DialectRegistry},
    ir::{
        attribute::{StringAttribute, TypeAttribute},
        r#type::FunctionType,
        *,
    },
    utility::register_all_dialects,
    Context,
};
use mlir_sys::MlirDialectHandle;
use operation::OperationBuilder;
use r#type::RankedTensorType;

melior::dialect! {
    name: "toy",
    td_file: "mlir/Ops.td",
}

/*
pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
*/

unsafe extern "C" {
    fn mlirGetDialectHandle__toy__() -> MlirDialectHandle;
}

fn main() {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let toy = unsafe { DialectHandle::from_raw(mlirGetDialectHandle__toy__()) };
    toy.insert_dialect(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let module = Module::new(location);

    let index_type = Type::index(&context);

    module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "add"),
        TypeAttribute::new(
            FunctionType::new(&context, &[index_type, index_type], &[index_type]).into(),
        ),
        {
            let block = Block::new(&[(index_type, location), (index_type, location)]);

            let sum = block.append_operation(arith::addi(
                block.argument(0).unwrap().into(),
                block.argument(1).unwrap().into(),
                location,
            ));

            block.append_operation(func::r#return(&[sum.result(0).unwrap().into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        &[(
            Identifier::new(&context, "sym_visibility").into(),
            StringAttribute::new(&context, "private").into(),
        )],
        location,
    ));
    module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "main"),
        TypeAttribute::new(FunctionType::new(&context, &[], &[]).into()),
        {
            let block = Block::new(&[]);
            let v0 = block.append_operation(arith::constant(
                &context,
                IntegerAttribute::new(index_type, 21).into(),
                location,
            ));
            let v0 = v0.result(0).unwrap();
            block.append_operation(func::call(
                &context,
                FlatSymbolRefAttribute::new(&context, "add"),
                &[v0.into(), v0.into()],
                &[index_type],
                location,
            ));

            let double_type = Type::parse(&context, "f64").unwrap();
            let tensor = RankedTensorType::new(&[1], double_type, None);
            let toy_constant = OperationBuilder::new("toy.constant", location)
                .add_attributes(&[(
                    Identifier::new(&context, "value"),
                    DenseElementsAttribute::new(
                        tensor.into(),
                        &[FloatAttribute::new(&context, double_type, 3.14).into()],
                    )
                    .unwrap()
                    .into(),
                )])
                .add_results(&[tensor.into()])
                .build()
                .unwrap();

            block.append_operation(toy_constant);

            block.append_operation(func::r#return(&[], location));
            let region = Region::new();
            region.append_block(block);
            region
        },
        &[(
            Identifier::new(&context, "sym_visibility").into(),
            StringAttribute::new(&context, "private").into(),
        )],
        location,
    ));

    assert!(module.as_operation().verify());
    dbg!(module.as_operation());
}
