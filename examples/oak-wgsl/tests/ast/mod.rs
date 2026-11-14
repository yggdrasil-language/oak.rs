use oak_core::{Builder, OakError, SourceText, parser::session::ParseSession};
use oak_wgsl::{
    WgslBuilder, WgslLanguage,
    ast::{WgslAccessMode, WgslAddressSpace, WgslAssignOp, WgslBinaryOp, WgslExpression, WgslFunction, WgslItem, WgslRoot, WgslStatement, WgslType, WgslUnaryOp},
};

/// Builds the AST and the diagnostics for `source`.
fn build(source: &str) -> (WgslRoot, Vec<OakError>) {
    let language = WgslLanguage::default();
    let builder = WgslBuilder::new(&language);
    let text = SourceText::new(source);
    let mut cache = ParseSession::<WgslLanguage>::default();

    let output = builder.build(&text, &[], &mut cache);
    (output.result.expect("the WGSL source should build into an AST"), output.diagnostics)
}

/// Finds a function by name.
fn function<'a>(root: &'a WgslRoot, name: &str) -> &'a WgslFunction {
    root.items
        .iter()
        .find_map(|item| match item {
            WgslItem::Function(function) if function.name == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing function `{name}`"))
}

fn type_args(ty: &WgslType) -> Vec<&str> {
    ty.args.iter().map(|arg| arg.name.as_str()).collect()
}

#[test]
fn parses_compute_fill() {
    let (root, diagnostics) = build(include_str!("compute_fill.wgsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 2);

    let global = match &root.items[0] {
        WgslItem::GlobalVar(global) => global,
        other => panic!("expected a global variable, got {other:?}"),
    };
    assert_eq!(global.name, "out");
    assert_eq!(global.address_space, Some(WgslAddressSpace::Storage));
    assert_eq!(global.access, Some(WgslAccessMode::ReadWrite));
    assert_eq!(global.attributes.len(), 2);
    assert_eq!(global.attributes[0].name, "group");
    assert!(matches!(global.attributes[0].args.as_slice(), [WgslExpression::IntegerLiteral(0)]));
    assert_eq!(global.attributes[1].name, "binding");
    assert!(matches!(global.attributes[1].args.as_slice(), [WgslExpression::IntegerLiteral(0)]));

    let ty = global.ty.as_ref().expect("the variable should have a type");
    assert_eq!(ty.name, "array");
    assert_eq!(type_args(ty), ["f32"]);

    let fill = function(&root, "fill");
    assert_eq!(fill.attributes.len(), 2);
    assert_eq!(fill.attributes[0].name, "compute");
    assert_eq!(fill.attributes[1].name, "workgroup_size");
    assert!(matches!(fill.attributes[1].args.as_slice(), [WgslExpression::IntegerLiteral(64), WgslExpression::IntegerLiteral(1), WgslExpression::IntegerLiteral(1)]));
    assert_eq!(fill.params.len(), 1);
    assert_eq!(fill.params[0].name, "gid");
    assert_eq!(fill.params[0].attributes[0].name, "builtin");
    assert!(matches!(
        fill.params[0].attributes[0].args.as_slice(),
        [WgslExpression::Ident(name)] if name == "global_invocation_id"
    ));
    assert_eq!(fill.params[0].ty.name, "vec3");
    assert_eq!(type_args(&fill.params[0].ty), ["u32"]);

    assert_eq!(fill.body.len(), 2);
    match &fill.body[0] {
        WgslStatement::Var { name, ty, init: Some(WgslExpression::Member { base, field }), is_const } => {
            assert_eq!(name, "index");
            assert!(ty.is_none());
            assert!(*is_const, "`let` declarations are immutable");
            assert!(matches!(base.as_ref(), WgslExpression::Ident(base) if base == "gid"));
            assert_eq!(field, "x");
        }
        other => panic!("expected a `let` statement, got {other:?}"),
    }

    match &fill.body[1] {
        WgslStatement::Assign { target: WgslExpression::Index { base, index }, op, value } => {
            assert_eq!(*op, WgslAssignOp::Assign);
            assert!(matches!(base.as_ref(), WgslExpression::Ident(base) if base == "out"));
            assert!(matches!(index.as_ref(), WgslExpression::Ident(index) if index == "index"));
            assert!(matches!(
                value,
                WgslExpression::Call { callee, args }
                    if callee == "f32" && matches!(args.as_slice(), [WgslExpression::Ident(argument)] if argument == "index")
            ));
        }
        other => panic!("expected an assignment, got {other:?}"),
    }
}

#[test]
fn parses_matmul_control_flow_and_precedence() {
    let (root, diagnostics) = build(include_str!("matmul.wgsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let global_names: Vec<_> = root
        .items
        .iter()
        .filter_map(|item| match item {
            WgslItem::GlobalVar(global) => Some((global.name.as_str(), global.access)),
            _ => None,
        })
        .collect();
    assert_eq!(global_names, vec![("a", Some(WgslAccessMode::Read)), ("b", Some(WgslAccessMode::Read)), ("result", Some(WgslAccessMode::ReadWrite))]);

    let index_function = function(&root, "index");
    assert_eq!(index_function.return_type.as_ref().map(|ty| ty.name.as_str()), Some("u32"));
    assert!(matches!(index_function.body.as_slice(), [WgslStatement::Return(Some(WgslExpression::Binary { op: WgslBinaryOp::Add, .. }))]));

    let matmul = function(&root, "matmul");
    assert_eq!(matmul.body.len(), 7, "unexpected body: {:#?}", matmul.body);

    // `1.0 + 2.0 * 3.0` groups multiplication first.
    match &matmul.body[3] {
        WgslStatement::Var { name, init: Some(WgslExpression::Binary { op: WgslBinaryOp::Add, lhs, rhs }), .. } => {
            assert_eq!(name, "scaled");
            assert!(matches!(lhs.as_ref(), WgslExpression::FloatLiteral(value) if *value == 1.0));
            assert!(matches!(
                rhs.as_ref(),
                WgslExpression::Binary { op: WgslBinaryOp::Mul, lhs, rhs }
                    if matches!(lhs.as_ref(), WgslExpression::FloatLiteral(value) if *value == 2.0)
                        && matches!(rhs.as_ref(), WgslExpression::FloatLiteral(value) if *value == 3.0)
            ));
        }
        other => panic!("expected the `scaled` declaration, got {other:?}"),
    }

    let WgslStatement::For { init, cond, update, body } = &matmul.body[4]
    else {
        panic!("expected a `for` loop, got {:?}", matmul.body[4]);
    };

    match init.as_deref() {
        Some(WgslStatement::Var { name, ty, init: Some(WgslExpression::IntegerLiteral(0)), is_const }) => {
            assert_eq!(name, "k");
            assert!(ty.is_none());
            assert!(!*is_const);
        }
        other => panic!("unexpected loop initializer: {other:?}"),
    }

    match cond {
        Some(WgslExpression::Binary { op: WgslBinaryOp::Lt, lhs, rhs }) => {
            assert!(matches!(lhs.as_ref(), WgslExpression::Ident(name) if name == "k"));
            assert!(matches!(rhs.as_ref(), WgslExpression::Ident(name) if name == "N"));
        }
        other => panic!("unexpected loop condition: {other:?}"),
    }

    match update.as_deref() {
        Some(WgslStatement::Assign { target: WgslExpression::Ident(target), op: WgslAssignOp::Assign, value: WgslExpression::Binary { op: WgslBinaryOp::Add, lhs, rhs } }) => {
            assert_eq!(target, "k");
            assert!(matches!(lhs.as_ref(), WgslExpression::Ident(name) if name == "k"));
            assert!(matches!(rhs.as_ref(), WgslExpression::IntegerLiteral(1)));
        }
        other => panic!("unexpected loop update: {other:?}"),
    }

    match body.as_slice() {
        [WgslStatement::Assign { value: WgslExpression::Binary { op: WgslBinaryOp::Add, lhs, rhs }, .. }] => {
            assert!(matches!(lhs.as_ref(), WgslExpression::Ident(name) if name == "sum"));
            match rhs.as_ref() {
                WgslExpression::Binary { op: WgslBinaryOp::Mul, lhs, rhs } => {
                    assert!(matches!(
                        lhs.as_ref(),
                        WgslExpression::Index { base, index }
                            if matches!(base.as_ref(), WgslExpression::Ident(name) if name == "a")
                                && matches!(
                                    index.as_ref(),
                                    WgslExpression::Call { callee, args }
                                        if callee == "index"
                                            && matches!(args.as_slice(), [WgslExpression::Ident(row), WgslExpression::Ident(k)] if row == "row" && k == "k")
                                )
                    ));
                    assert!(matches!(
                        rhs.as_ref(),
                        WgslExpression::Index { base, index }
                            if matches!(base.as_ref(), WgslExpression::Ident(name) if name == "b")
                                && matches!(
                                    index.as_ref(),
                                    WgslExpression::Call { callee, args }
                                        if callee == "index"
                                            && matches!(args.as_slice(), [WgslExpression::Ident(k), WgslExpression::Ident(col)] if k == "k" && col == "col")
                                )
                    ));
                }
                other => panic!("expected a multiplication, got {other:?}"),
            }
        }
        other => panic!("unexpected loop body: {other:?}"),
    }

    match &matmul.body[5] {
        WgslStatement::If { cond: WgslExpression::Paren(cond), then_body, else_body } => {
            assert!(matches!(
                cond.as_ref(),
                WgslExpression::Binary { op: WgslBinaryOp::Gt, lhs, rhs }
                    if matches!(lhs.as_ref(), WgslExpression::Ident(name) if name == "sum")
                        && matches!(rhs.as_ref(), WgslExpression::FloatLiteral(value) if *value == 100.0)
            ));
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_empty());
        }
        other => panic!("expected an `if` statement, got {other:?}"),
    }

    match &matmul.body[6] {
        WgslStatement::Assign { target: WgslExpression::Index { base, index }, op: WgslAssignOp::Assign, value: WgslExpression::Ident(value) } => {
            assert!(matches!(base.as_ref(), WgslExpression::Ident(name) if name == "result"));
            assert!(matches!(index.as_ref(), WgslExpression::Call { callee, .. } if callee == "index"));
            assert_eq!(value, "sum");
        }
        other => panic!("expected the result assignment, got {other:?}"),
    }
}

#[test]
fn parses_vertex_and_fragment() {
    let (root, diagnostics) = build(include_str!("vertex_fragment.wgsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 3);

    let structure = match &root.items[0] {
        WgslItem::Struct(structure) => structure,
        other => panic!("expected a struct, got {other:?}"),
    };
    assert_eq!(structure.name, "VertexOutput");
    assert_eq!(structure.members.len(), 2);

    assert_eq!(structure.members[0].name, "position");
    assert_eq!(structure.members[0].attributes[0].name, "builtin");
    assert!(matches!(
        structure.members[0].attributes[0].args.as_slice(),
        [WgslExpression::Ident(name)] if name == "position"
    ));
    assert_eq!(structure.members[0].ty.name, "vec4");
    assert_eq!(type_args(&structure.members[0].ty), ["f32"]);

    assert_eq!(structure.members[1].name, "color");
    assert_eq!(structure.members[1].attributes[0].name, "location");
    assert!(matches!(structure.members[1].attributes[0].args.as_slice(), [WgslExpression::IntegerLiteral(0)]));
    assert_eq!(structure.members[1].ty.name, "vec3");

    let vertex = function(&root, "vs_main");
    assert_eq!(vertex.attributes[0].name, "vertex");
    assert_eq!(vertex.params[0].name, "vertex_index");
    assert_eq!(vertex.return_type.as_ref().map(|ty| ty.name.as_str()), Some("VertexOutput"));
    assert_eq!(vertex.body.len(), 4);
    assert!(matches!(
        &vertex.body[0],
        WgslStatement::Var { name, ty: Some(ty), init: None, is_const: false }
            if name == "out" && ty.name == "VertexOutput"
    ));
    assert!(matches!(
        &vertex.body[1],
        WgslStatement::Assign { target: WgslExpression::Member { field, .. }, value: WgslExpression::Call { callee, args }, .. }
            if field == "position" && callee == "vec4<f32>" && args.len() == 4
    ));
    assert!(matches!(
        &vertex.body[2],
        WgslStatement::Assign { target: WgslExpression::Member { field, .. }, value: WgslExpression::Call { callee, args }, .. }
            if field == "color" && callee == "vec3<f32>" && args.len() == 3
    ));
    assert!(matches!(&vertex.body[3], WgslStatement::Return(Some(WgslExpression::Ident(name))) if name == "out"));

    let fragment = function(&root, "fs_main");
    assert_eq!(fragment.attributes[0].name, "fragment");
    assert_eq!(fragment.return_type.as_ref().map(|ty| ty.name.as_str()), Some("vec4"));
    assert_eq!(type_args(fragment.return_type.as_ref().expect("return type")), ["f32"]);
    assert!(matches!(
        fragment.body.as_slice(),
        [WgslStatement::Return(Some(WgslExpression::Call { callee, args }))]
            if callee == "vec4<f32>"
                && matches!(
                    args.as_slice(),
                    [WgslExpression::Member { field, .. }, WgslExpression::FloatLiteral(value)] if field == "color" && *value == 1.0
                )
    ));
}

#[test]
fn parses_nested_generics_and_shifts() {
    let source = "\
fn shifts(x: u32) -> u32 {
    var buffer: array<array<f32, 4>, 4>;
    let right = x >> 2u;
    let left = x << 2u;
    let negated = -1;
    let inverted = ~x;
    return right | left;
}
";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let shifts = function(&root, "shifts");
    match &shifts.body[0] {
        WgslStatement::Var { ty: Some(ty), .. } => {
            assert_eq!(ty.name, "array");
            assert_eq!(ty.args.len(), 2);
            assert_eq!(ty.args[0].name, "array");
            assert_eq!(type_args(&ty.args[0]), ["f32", "4"]);
            assert_eq!(ty.args[1].name, "4");
        }
        other => panic!("expected a typed variable, got {other:?}"),
    }

    assert!(matches!(&shifts.body[1], WgslStatement::Var { init: Some(WgslExpression::Binary { op: WgslBinaryOp::Shr, .. }), .. }));
    assert!(matches!(&shifts.body[2], WgslStatement::Var { init: Some(WgslExpression::Binary { op: WgslBinaryOp::Shl, .. }), .. }));
    assert!(matches!(&shifts.body[3], WgslStatement::Var { init: Some(WgslExpression::Unary { op: WgslUnaryOp::Neg, .. }), .. }));
    assert!(matches!(&shifts.body[4], WgslStatement::Var { init: Some(WgslExpression::Unary { op: WgslUnaryOp::BitNot, .. }), .. }));
    assert!(matches!(&shifts.body[5], WgslStatement::Return(Some(WgslExpression::Binary { op: WgslBinaryOp::BitOr, .. }))));
}

#[test]
fn parses_module_directives_and_alias() {
    let source = "\
enable f16;
requires readonly_and_readwrite_storage_textures;
alias Vec4 = vec4<f32>;
fn f() {}
";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 2, "directives do not produce items: {:?}", root.items);
    match &root.items[0] {
        WgslItem::TypeAlias(alias) => {
            assert_eq!(alias.name, "Vec4");
            assert_eq!(alias.ty.name, "vec4");
            assert_eq!(type_args(&alias.ty), ["f32"]);
        }
        other => panic!("expected a type alias, got {other:?}"),
    }
    assert!(matches!(&root.items[1], WgslItem::Function(function) if function.name == "f"));
}

#[test]
fn parses_statement_forms() {
    let source = "\
fn control(x: u32) -> u32 {
    var total = 0u;
    total += x;
    total <<= 1u;
    total >>= 2u;
    const limit = 4u;
    while (total < limit) {
        total = total + 1u;
    }
    loop {
        break;
    }
    for (var i = 0u; i < 2u; i = i + 1u) {
        if (i == 0u) {
            continue;
        } else {
            discard;
        }
    }
    {
        var nested = 1u;
    }
    x;
    return total;
}
";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let control = function(&root, "control");
    assert_eq!(control.body.len(), 11, "unexpected body: {:#?}", control.body);

    assert!(matches!(&control.body[0], WgslStatement::Var { name, is_const: false, .. } if name == "total"));
    assert!(matches!(&control.body[1], WgslStatement::Assign { op: WgslAssignOp::AddAssign, .. }));
    assert!(matches!(&control.body[2], WgslStatement::Assign { op: WgslAssignOp::ShlAssign, .. }));
    assert!(matches!(&control.body[3], WgslStatement::Assign { op: WgslAssignOp::ShrAssign, .. }));
    assert!(matches!(&control.body[4], WgslStatement::Var { name, is_const: true, .. } if name == "limit"));

    match &control.body[5] {
        WgslStatement::While { cond: WgslExpression::Paren(cond), body } => {
            assert!(matches!(cond.as_ref(), WgslExpression::Binary { op: WgslBinaryOp::Lt, .. }));
            assert!(matches!(body.as_slice(), [WgslStatement::Assign { .. }]));
        }
        other => panic!("expected a `while` loop, got {other:?}"),
    }

    assert!(matches!(&control.body[6], WgslStatement::Loop { body } if matches!(body.as_slice(), [WgslStatement::Break])));

    match &control.body[7] {
        WgslStatement::For { body, .. } => match body.as_slice() {
            [WgslStatement::If { then_body, else_body, .. }] => {
                assert!(matches!(then_body.as_slice(), [WgslStatement::Continue]));
                assert!(matches!(else_body.as_slice(), [WgslStatement::Discard]));
            }
            other => panic!("unexpected `for` body: {other:?}"),
        },
        other => panic!("expected a `for` loop, got {other:?}"),
    }

    assert!(matches!(&control.body[8], WgslStatement::Block(statements) if matches!(statements.as_slice(), [WgslStatement::Var { name, .. }] if name == "nested")));
    assert!(matches!(&control.body[9], WgslStatement::Expr(WgslExpression::Ident(name)) if name == "x"));
    assert!(matches!(&control.body[10], WgslStatement::Return(Some(WgslExpression::Ident(name))) if name == "total"));
}

#[test]
fn parses_type_forms() {
    let source = "fn typed(a: f32, b: i32, c: u32, d: bool, e: vec3<f32>, f: mat4x4<f32>, g: array<f32>, h: Custom, i: array<f32, 64>) {}";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let typed = function(&root, "typed");
    let types: Vec<_> = typed.params.iter().map(|param| (param.name.as_str(), param.ty.name.as_str(), type_args(&param.ty))).collect();
    assert_eq!(
        types,
        vec![("a", "f32", vec![]), ("b", "i32", vec![]), ("c", "u32", vec![]), ("d", "bool", vec![]), ("e", "vec3", vec!["f32"]), ("f", "mat4x4", vec!["f32"]), ("g", "array", vec!["f32"]), ("h", "Custom", vec![]), ("i", "array", vec!["f32", "64"]),]
    );
}

#[test]
fn reports_truncated_function() {
    let (root, diagnostics) = build("fn broken(");
    assert!(!diagnostics.is_empty(), "a truncated function must report diagnostics");
    assert_eq!(root.items.len(), 1);
    assert_eq!(function(&root, "broken").name, "broken");
}

#[test]
fn reports_unknown_attribute() {
    let (root, diagnostics) = build("@mystery(1) fn f() {}");
    assert!(diagnostics.iter().any(|error| format!("{error:?}").contains("unknown WGSL attribute")), "expected an unknown attribute diagnostic, got {diagnostics:?}");
    let f = function(&root, "f");
    assert_eq!(f.attributes.len(), 1);
    assert_eq!(f.attributes[0].name, "mystery");
}

#[test]
fn reports_missing_right_paren() {
    let (_, diagnostics) = build("@group(0 fn f() { let x = 1; }");
    assert!(!diagnostics.is_empty(), "a missing `)` must report diagnostics");
}
