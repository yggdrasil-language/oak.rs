use oak_core::{Builder, OakError, SourceText, parser::session::ParseSession};
use oak_hlsl::{
    HlslBuilder, HlslLanguage,
    ast::{HlslAssignOp, HlslBinaryOp, HlslExpression, HlslFunction, HlslGlobalVar, HlslItem, HlslQualifier, HlslRoot, HlslStatement, HlslUnaryOp},
};

/// Builds the AST and the diagnostics for `source`.
fn build(source: &str) -> (HlslRoot, Vec<OakError>) {
    let language = HlslLanguage::default();
    let builder = HlslBuilder::new(&language);
    let text = SourceText::new(source);
    let mut cache = ParseSession::<HlslLanguage>::default();

    let output = builder.build(&text, &[], &mut cache);
    (output.result.expect("the HLSL source should build into an AST"), output.diagnostics)
}

/// Finds a function by name.
fn function<'a>(root: &'a HlslRoot, name: &str) -> &'a HlslFunction {
    root.items
        .iter()
        .find_map(|item| match item {
            HlslItem::Function(function) if function.name == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing function `{name}`"))
}

/// Finds a module-scope variable by name.
fn global_var<'a>(root: &'a HlslRoot, name: &str) -> &'a HlslGlobalVar {
    root.items
        .iter()
        .find_map(|item| match item {
            HlslItem::GlobalVar(global) if global.name == name => Some(global),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing global variable `{name}`"))
}

fn type_args<'a>(ty: &'a oak_hlsl::ast::HlslType) -> Vec<&'a str> {
    ty.args.iter().map(|arg| arg.name.as_str()).collect()
}

#[test]
fn parses_compute_kernel() {
    let (root, diagnostics) = build(include_str!("compute.hlsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 3, "unexpected items: {:?}", root.items.len());

    let params = match &root.items[1] {
        HlslItem::Cbuffer(buffer) => buffer,
        other => panic!("expected the `Params` constant buffer, got {other:?}"),
    };
    assert_eq!(params.kind, "cbuffer");
    assert_eq!(params.name, "Params");
    let register = params.register.as_ref().expect("the constant buffer should have a register");
    assert_eq!(register.class.as_deref(), Some("b"));
    assert_eq!(register.index, Some(0));
    assert_eq!(params.members.len(), 2);
    assert_eq!(params.members[0].name, "tint");
    assert_eq!(params.members[0].ty.name, "float4");
    assert_eq!(params.members[1].name, "exposure");
    assert_eq!(params.members[1].ty.name, "float");

    let output = global_var(&root, "output");
    let ty = output.ty.as_ref().expect("the buffer should have a type");
    assert_eq!(ty.name, "RWStructuredBuffer");
    assert_eq!(type_args(ty), ["float4"]);
    let register = output.register.as_ref().expect("the buffer should have a register");
    assert_eq!(register.class.as_deref(), Some("u"));
    assert_eq!(register.index, Some(0));

    let kernel = function(&root, "cs_main");
    assert_eq!(kernel.attributes.len(), 1);
    assert_eq!(kernel.attributes[0].name, "numthreads");
    assert!(matches!(kernel.attributes[0].args.as_slice(), [HlslExpression::IntegerLiteral(64), HlslExpression::IntegerLiteral(1), HlslExpression::IntegerLiteral(1)]));
    assert_eq!(kernel.return_type.as_ref().map(|ty| ty.name.as_str()), Some("void"));
    assert_eq!(kernel.params.len(), 1);
    assert_eq!(kernel.params[0].name, "id");
    assert_eq!(kernel.params[0].ty.name, "uint3");
    assert_eq!(kernel.params[0].semantic.as_deref(), Some("SV_DispatchThreadID"));

    assert_eq!(kernel.body.len(), 4, "unexpected body: {:#?}", kernel.body);
    match &kernel.body[0] {
        HlslStatement::Var { name, ty, init: Some(HlslExpression::Member { base, field }), is_const, .. } => {
            assert_eq!(name, "index");
            assert_eq!(ty.as_ref().map(|ty| ty.name.as_str()), Some("uint"));
            assert!(!is_const);
            assert!(matches!(base.as_ref(), HlslExpression::Ident(base) if base == "id"));
            assert_eq!(field, "x");
        }
        other => panic!("expected the `index` declaration, got {other:?}"),
    }

    match &kernel.body[1] {
        HlslStatement::Var { name, init: Some(HlslExpression::Call { callee, args }), .. } => {
            assert_eq!(name, "value");
            assert_eq!(callee, "float4");
            assert!(matches!(
                args.as_slice(),
                [HlslExpression::FloatLiteral(first), HlslExpression::FloatLiteral(second), ..] if *first == 1.0 && *second == 0.5
            ));
        }
        other => panic!("expected the `value` declaration, got {other:?}"),
    }

    match &kernel.body[2] {
        HlslStatement::If { cond: HlslExpression::Paren(cond), then_body, else_body } => {
            assert!(matches!(
                cond.as_ref(),
                HlslExpression::Binary { op: HlslBinaryOp::Ge, lhs, rhs }
                    if matches!(lhs.as_ref(), HlslExpression::Ident(name) if name == "index")
                        && matches!(rhs.as_ref(), HlslExpression::IntegerLiteral(64))
            ));
            assert!(matches!(then_body.as_slice(), [HlslStatement::Return(None)]));
            assert!(else_body.is_empty());
        }
        other => panic!("expected an `if` statement, got {other:?}"),
    }

    match &kernel.body[3] {
        HlslStatement::Assign { target: HlslExpression::Index { base, index }, op: HlslAssignOp::Assign, value: HlslExpression::Binary { op: HlslBinaryOp::Mul, .. } } => {
            assert!(matches!(base.as_ref(), HlslExpression::Ident(name) if name == "output"));
            assert!(matches!(index.as_ref(), HlslExpression::Ident(name) if name == "index"));
        }
        other => panic!("expected the buffer write, got {other:?}"),
    }
}

#[test]
fn parses_loops_and_branches() {
    let (root, diagnostics) = build(include_str!("loops.hlsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let accumulate = function(&root, "accumulate");
    assert_eq!(accumulate.return_type.as_ref().map(|ty| ty.name.as_str()), Some("float"));
    assert_eq!(accumulate.body.len(), 6, "unexpected body: {:#?}", accumulate.body);

    assert!(matches!(&accumulate.body[0], HlslStatement::Var { name, .. } if name == "total"));

    let HlslStatement::For { init, cond, update, body } = &accumulate.body[1]
    else {
        panic!("expected a `for` loop, got {:?}", accumulate.body[1]);
    };
    match init.as_deref() {
        Some(HlslStatement::Var { name, ty, init: Some(HlslExpression::IntegerLiteral(0)), is_const: false, .. }) => {
            assert_eq!(name, "i");
            assert_eq!(ty.as_ref().map(|ty| ty.name.as_str()), Some("int"));
        }
        other => panic!("unexpected loop initializer: {other:?}"),
    }
    assert!(matches!(cond, Some(HlslExpression::Binary { op: HlslBinaryOp::Lt, .. })));
    assert!(matches!(update.as_deref(), Some(HlslStatement::Expr(HlslExpression::Unary { op: HlslUnaryOp::PostIncrement, .. }))));
    assert!(matches!(body.as_slice(), [HlslStatement::Assign { op: HlslAssignOp::AddAssign, value: HlslExpression::Binary { op: HlslBinaryOp::Mul, .. }, .. }]));

    assert!(matches!(&accumulate.body[2], HlslStatement::Var { name, .. } if name == "remaining"));

    match &accumulate.body[3] {
        HlslStatement::While { cond, body } => {
            assert!(matches!(cond, HlslExpression::Paren(cond) if matches!(cond.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::Gt, .. })));
            assert_eq!(body.len(), 3, "unexpected while body: {body:#?}");
            assert!(matches!(body[0], HlslStatement::Expr(HlslExpression::Unary { op: HlslUnaryOp::PostDecrement, .. })));
            match &body[1] {
                HlslStatement::If { then_body, else_body, .. } => {
                    assert!(matches!(then_body.as_slice(), [HlslStatement::Continue]));
                    assert!(matches!(else_body.as_slice(), [HlslStatement::If { then_body, .. }] if matches!(then_body.as_slice(), [HlslStatement::Break])));
                }
                other => panic!("expected the nested branch, got {other:?}"),
            }
            assert!(matches!(body[2], HlslStatement::Assign { op: HlslAssignOp::Assign, value: HlslExpression::Binary { op: HlslBinaryOp::Sub, .. }, .. }));
        }
        other => panic!("expected a `while` loop, got {other:?}"),
    }

    match &accumulate.body[4] {
        HlslStatement::DoWhile { body, cond } => {
            assert!(matches!(body.as_slice(), [HlslStatement::Assign { op: HlslAssignOp::MulAssign, .. }, HlslStatement::Expr(HlslExpression::Unary { op: HlslUnaryOp::PostIncrement, .. })]));
            assert!(matches!(cond, HlslExpression::Paren(cond) if matches!(cond.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::Lt, .. })));
        }
        other => panic!("expected a `do ... while` loop, got {other:?}"),
    }

    assert!(matches!(&accumulate.body[5], HlslStatement::Return(Some(HlslExpression::Ident(name))) if name == "total"));
}

#[test]
fn parses_structs_qualifiers_and_aliases() {
    let (root, diagnostics) = build(include_str!("structs.hlsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 8, "unexpected items: {:#?}", root.items);

    let input = match &root.items[0] {
        HlslItem::Struct(structure) => structure,
        other => panic!("expected a struct, got {other:?}"),
    };
    assert_eq!(input.name, "VSInput");
    assert_eq!(input.members.len(), 3);
    assert_eq!(input.members[0].name, "position");
    assert_eq!(input.members[0].ty.name, "float4");
    assert_eq!(input.members[0].semantic.as_deref(), Some("POSITION"));
    assert_eq!(input.members[1].semantic.as_deref(), Some("TEXCOORD0"));

    match &root.items[2] {
        HlslItem::TypeAlias(alias) => {
            assert_eq!(alias.name, "Color");
            assert_eq!(alias.ty.name, "float4");
        }
        other => panic!("expected a `using` alias, got {other:?}"),
    }
    match &root.items[3] {
        HlslItem::TypeAlias(alias) => {
            assert_eq!(alias.name, "Vector3");
            assert_eq!(alias.ty.name, "float3");
        }
        other => panic!("expected a `typedef` alias, got {other:?}"),
    }

    let world = global_var(&root, "gWorld");
    assert_eq!(world.qualifiers, vec![HlslQualifier::Uniform]);
    assert_eq!(world.ty.as_ref().map(|ty| ty.name.as_str()), Some("float4x4"));

    let texture = global_var(&root, "gTexture");
    assert_eq!(texture.ty.as_ref().map(|ty| ty.name.as_str()), Some("Texture2D"));
    let register = texture.register.as_ref().expect("the texture should have a register");
    assert_eq!(register.class.as_deref(), Some("t"));
    assert_eq!(register.index, Some(0));

    let sampler = global_var(&root, "gSampler");
    assert_eq!(sampler.register.as_ref().and_then(|register| register.class.as_deref()), Some("s"));

    let vertex = function(&root, "vs_main");
    assert_eq!(vertex.return_type.as_ref().map(|ty| ty.name.as_str()), Some("VSOutput"));
    assert_eq!(vertex.params.len(), 1);
    assert_eq!(vertex.params[0].ty.name, "VSInput");
    assert_eq!(vertex.body.len(), 5, "unexpected body: {:#?}", vertex.body);
    assert!(matches!(
        &vertex.body[1],
        HlslStatement::Var { init: Some(HlslExpression::Call { callee, args }), .. }
            if callee == "mul" && args.len() == 2
    ));
    assert!(matches!(
        &vertex.body[2],
        HlslStatement::Assign { target: HlslExpression::Member { field, .. }, .. } if field == "position"
    ));
}

#[test]
fn parses_expression_precedence() {
    let source = "float4 f(float3 a, float3 b) { float3 c = a + b * 2.0f; float3 d = (a + b) * 2.0f; uint m = 0xFFu; int n = (1 << 2) | 3 & 4; return float4(c, 1.0f); }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let f = function(&root, "f");
    match &f.body[0] {
        HlslStatement::Var { init: Some(HlslExpression::Binary { op: HlslBinaryOp::Add, rhs, .. }), .. } => {
            assert!(matches!(rhs.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::Mul, .. }));
        }
        other => panic!("expected `a + b * 2.0`, got {other:?}"),
    }
    match &f.body[1] {
        HlslStatement::Var { init: Some(HlslExpression::Binary { op: HlslBinaryOp::Mul, lhs, .. }), .. } => {
            assert!(matches!(lhs.as_ref(), HlslExpression::Paren(inner) if matches!(inner.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::Add, .. })));
        }
        other => panic!("expected `(a + b) * 2.0`, got {other:?}"),
    }
    assert!(matches!(&f.body[2], HlslStatement::Var { init: Some(HlslExpression::IntegerLiteral(255)), .. }));
    match &f.body[3] {
        HlslStatement::Var { init: Some(HlslExpression::Binary { op: HlslBinaryOp::BitOr, lhs, rhs }), .. } => {
            assert!(matches!(lhs.as_ref(), HlslExpression::Paren(inner) if matches!(inner.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::Shl, .. })));
            assert!(matches!(rhs.as_ref(), HlslExpression::Binary { op: HlslBinaryOp::BitAnd, .. }));
        }
        other => panic!("expected a bitwise expression, got {other:?}"),
    }
}

#[test]
fn reports_truncated_function() {
    let (root, diagnostics) = build("float4 main(");
    assert!(!diagnostics.is_empty(), "a truncated function must report diagnostics");
    assert_eq!(root.items.len(), 1);
    assert_eq!(function(&root, "main").name, "main");
}

#[test]
fn reports_unknown_attribute() {
    let (root, diagnostics) = build("[mystery(1)] void f() {}");
    assert!(diagnostics.iter().any(|error| format!("{error:?}").contains("unknown HLSL attribute")), "expected an unknown attribute diagnostic, got {diagnostics:?}");

    let f = function(&root, "f");
    assert_eq!(f.attributes.len(), 1);
    assert_eq!(f.attributes[0].name, "mystery");
    assert!(matches!(f.attributes[0].args.as_slice(), [HlslExpression::IntegerLiteral(1)]));
}

#[test]
fn reports_unknown_construct_and_recovers() {
    let (root, diagnostics) = build("@ @ void f() {}");
    assert_eq!(diagnostics.len(), 2, "each stray token must be reported: {diagnostics:?}");
    assert_eq!(root.items.len(), 1);
    assert_eq!(function(&root, "f").name, "f");
}
