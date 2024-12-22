use attribute::{DenseElementsAttribute, FlatSymbolRefAttribute, FloatAttribute, IntegerAttribute};
use llvm_sys::core::{LLVMContextCreate, LLVMPrintModuleToString};
use melior::{
    dialect::{arith, func, DialectHandle, DialectRegistry},
    ir::{
        attribute::{StringAttribute, TypeAttribute},
        r#type::FunctionType,
        *,
    },
    pass::{conversion::create_to_llvm, transform::create_inliner, PassManager},
    utility::{register_all_dialects, register_all_llvm_translations},
    Context,
};
use mlir_sys::{mlirTranslateModuleToLLVMIR, MlirDialectHandle};
use operation::OperationBuilder;
use r#type::RankedTensorType;

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
    let mut module = Module::new(location);

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
        TypeAttribute::new(FunctionType::new(&context, &[], &[index_type]).into()),
        {
            let block = Block::new(&[]);
            let v0 = block.append_operation(arith::constant(
                &context,
                IntegerAttribute::new(index_type, 21).into(),
                location,
            ));
            let v0 = v0.result(0).unwrap();
            let ans = block.append_operation(func::call(
                &context,
                FlatSymbolRefAttribute::new(&context, "add"),
                &[v0.into(), v0.into()],
                &[index_type],
                location,
            ));
            let ans = ans.result(0).unwrap();

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

            block.append_operation(func::r#return(&[ans.into()], location));
            let region = Region::new();
            region.append_block(block);
            region
        },
        &[],
        location,
    ));

    assert!(module.as_operation().verify());
    dbg!(module.as_operation());

    // register_all_passes();
    let pass_manager = PassManager::new(&context);
    pass_manager.add_pass(create_inliner());
    // pass_manager.add_pass(create_canonicalizer());
    // pass_manager.add_pass(create_cse());

    pass_manager.run(&mut module).unwrap();

    assert!(module.as_operation().verify());
    println!("After some passes:");
    dbg!(module.as_operation());

    let pass_manager = PassManager::new(&context);
    pass_manager.add_pass(create_to_llvm());
    pass_manager.run(&mut module).unwrap();

    assert!(module.as_operation().verify());
    println!("to llvm:");
    dbg!(module.as_operation());

    // TO LLVM IR
    register_all_llvm_translations(&context);
    let llvm_context = unsafe { LLVMContextCreate() };
    let llvm_module =
        unsafe { mlirTranslateModuleToLLVMIR(module.as_operation().to_raw(), llvm_context as _) };

    let s = unsafe { LLVMPrintModuleToString(llvm_module as _) };

    let s = unsafe { std::ffi::CStr::from_ptr(s) };
    println!("LLVM IR:");
    println!("{}", s.to_str().unwrap());
}
