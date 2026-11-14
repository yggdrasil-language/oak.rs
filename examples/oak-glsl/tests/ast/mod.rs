use oak_core::{Builder, OakError, SourceText, parser::session::ParseSession};
use oak_glsl::{
    GlslBuilder, GlslLanguage,
    ast::{GlslAssignOp, GlslBinaryOp, GlslExpression, GlslFunction, GlslGlobalVar, GlslItem, GlslQualifier, GlslRoot, GlslStatement, GlslUnaryOp},
};

/// Builds the AST and the diagnostics for `source`.
fn build(source: &str) -> (GlslRoot, Vec<OakError>) {
    let language = GlslLanguage::default();
    let builder = GlslBuilder::new(&language);
    let text = SourceText::new(source);
    let mut cache = ParseSession::<GlslLanguage>::default();

    let output = builder.build(&text, &[], &mut cache);
    (output.result.expect("the GLSL source should build into an AST"), output.diagnostics)
}

/// Finds a function by name.
fn function<'a>(root: &'a GlslRoot, name: &str) -> &'a GlslFunction {
    root.items
        .iter()
        .find_map(|item| match item {
            GlslItem::Function(function) if function.name == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing function `{name}`"))
}

/// Finds a module-scope declaration by name.
fn global_var<'a>(root: &'a GlslRoot, name: &str) -> &'a GlslGlobalVar {
    root.items
        .iter()
        .find_map(|item| match item {
            GlslItem::GlobalVar(global) if global.name == name => Some(global),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing global declaration `{name}`"))
}

/// Returns the `(name, value)` pairs of layout qualifiers.
fn layout_names(global: &GlslGlobalVar) -> Vec<(&str, Option<i64>)> {
    global
        .layout
        .iter()
        .map(|entry| {
            let value = match &entry.value {
                Some(GlslExpression::IntegerLiteral(value)) => Some(*value),
                _ => None,
            };
            (entry.name.as_str(), value)
        })
        .collect()
}

#[test]
fn parses_compute_layouts_and_blocks() {
    let (root, diagnostics) = build(include_str!("compute.glsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 4, "unexpected items: {:#?}", root.items);

    let workgroup = global_var(&root, "");
    assert_eq!(workgroup.qualifiers, vec![GlslQualifier::In]);
    assert_eq!(layout_names(workgroup), vec![("local_size_x", Some(64)), ("local_size_y", Some(1)), ("local_size_z", Some(1))]);
    assert!(workgroup.ty.is_none(), "a layout-only declaration has no type");

    let output = global_var(&root, "Output");
    assert_eq!(output.qualifiers, vec![GlslQualifier::Buffer]);
    assert_eq!(layout_names(output), vec![("std430", None), ("binding", Some(0))]);
    assert_eq!(output.members.len(), 1);
    assert_eq!(output.members[0].name, "value");
    assert_eq!(output.members[0].ty.name, "float");

    let params = global_var(&root, "Params");
    assert_eq!(params.qualifiers, vec![GlslQualifier::Uniform]);
    assert_eq!(layout_names(params), vec![("std140", None), ("binding", Some(1))]);
    assert_eq!(params.members[0].ty.name, "vec4");

    let main = function(&root, "main");
    assert_eq!(main.return_type.as_ref().map(|ty| ty.name.as_str()), Some("void"));
    assert_eq!(main.body.len(), 4, "unexpected body: {:#?}", main.body);

    match &main.body[0] {
        GlslStatement::Var { name, ty, init: Some(GlslExpression::Member { base, field }), .. } => {
            assert_eq!(name, "index");
            assert_eq!(ty.as_ref().map(|ty| ty.name.as_str()), Some("uint"));
            assert!(matches!(base.as_ref(), GlslExpression::Ident(base) if base == "gl_GlobalInvocationID"));
            assert_eq!(field, "x");
        }
        other => panic!("expected the `index` declaration, got {other:?}"),
    }

    match &main.body[1] {
        GlslStatement::Var { init: Some(GlslExpression::Binary { op: GlslBinaryOp::Mul, lhs, rhs }), .. } => {
            assert!(matches!(lhs.as_ref(), GlslExpression::Call { callee, args } if callee == "float" && args.len() == 1));
            assert!(matches!(rhs.as_ref(), GlslExpression::FloatLiteral(value) if *value == 0.5));
        }
        other => panic!("expected the `scaled` declaration, got {other:?}"),
    }

    match &main.body[2] {
        GlslStatement::If { cond: GlslExpression::Paren(cond), then_body, else_body } => {
            assert!(matches!(
                cond.as_ref(),
                GlslExpression::Binary { op: GlslBinaryOp::Ge, lhs, rhs }
                    if matches!(lhs.as_ref(), GlslExpression::Ident(name) if name == "index")
                        && matches!(rhs.as_ref(), GlslExpression::IntegerLiteral(64))
            ));
            assert!(matches!(then_body.as_slice(), [GlslStatement::Return(None)]));
            assert!(else_body.is_empty());
        }
        other => panic!("expected an `if` statement, got {other:?}"),
    }

    assert!(matches!(
        &main.body[3],
        GlslStatement::Assign { target: GlslExpression::Ident(target), op: GlslAssignOp::Assign, value: GlslExpression::Binary { op: GlslBinaryOp::Mul, .. } }
            if target == "value"
    ));
}

#[test]
fn parses_loops_and_branches() {
    let (root, diagnostics) = build(include_str!("loops.glsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let accumulate = function(&root, "accumulate");
    assert_eq!(accumulate.return_type.as_ref().map(|ty| ty.name.as_str()), Some("float"));
    assert_eq!(accumulate.body.len(), 6, "unexpected body: {:#?}", accumulate.body);

    let GlslStatement::For { init, cond, update, body } = &accumulate.body[1]
    else {
        panic!("expected a `for` loop, got {:?}", accumulate.body[1]);
    };
    assert!(matches!(
        init.as_deref(),
        Some(GlslStatement::Var { name, ty, init: Some(GlslExpression::IntegerLiteral(0)), .. }) if name == "i" && ty.as_ref().map(|ty| ty.name.as_str()) == Some("int")
    ));
    assert!(matches!(cond, Some(GlslExpression::Binary { op: GlslBinaryOp::Lt, .. })));
    assert!(matches!(update.as_deref(), Some(GlslStatement::Expr(GlslExpression::Unary { op: GlslUnaryOp::PreIncrement, .. }))));
    assert!(matches!(body.as_slice(), [GlslStatement::Assign { op: GlslAssignOp::AddAssign, value: GlslExpression::Binary { op: GlslBinaryOp::Mul, .. }, .. }]));

    match &accumulate.body[3] {
        GlslStatement::While { cond, body } => {
            assert!(matches!(cond, GlslExpression::Paren(cond) if matches!(cond.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Gt, .. })));
            assert_eq!(body.len(), 3, "unexpected while body: {body:#?}");
            assert!(matches!(body[0], GlslStatement::Expr(GlslExpression::Unary { op: GlslUnaryOp::PostDecrement, .. })));
            match &body[1] {
                GlslStatement::If { then_body, else_body, .. } => {
                    assert!(matches!(then_body.as_slice(), [GlslStatement::Continue]));
                    assert!(matches!(else_body.as_slice(), [GlslStatement::If { then_body, .. }] if matches!(then_body.as_slice(), [GlslStatement::Break])));
                }
                other => panic!("expected the nested branch, got {other:?}"),
            }
        }
        other => panic!("expected a `while` loop, got {other:?}"),
    }

    match &accumulate.body[4] {
        GlslStatement::DoWhile { body, cond } => {
            assert!(matches!(body.as_slice(), [GlslStatement::Assign { op: GlslAssignOp::MulAssign, .. }, GlslStatement::Expr(GlslExpression::Unary { op: GlslUnaryOp::PostIncrement, .. })]));
            assert!(matches!(cond, GlslExpression::Paren(cond) if matches!(cond.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Lt, .. })));
        }
        other => panic!("expected a `do ... while` loop, got {other:?}"),
    }

    assert!(matches!(&accumulate.body[5], GlslStatement::Return(Some(GlslExpression::Ident(name))) if name == "total"));
}

#[test]
fn parses_structs_layouts_and_uniforms() {
    let (root, diagnostics) = build(include_str!("structs.glsl"));
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    assert_eq!(root.items.len(), 8, "unexpected items: {:#?}", root.items.len());

    let light = match &root.items[0] {
        GlslItem::Struct(structure) => structure,
        other => panic!("expected a struct, got {other:?}"),
    };
    assert_eq!(light.name, "Light");
    assert_eq!(light.members.len(), 2);
    assert_eq!(light.members[0].name, "position");
    assert_eq!(light.members[0].ty.name, "vec3");
    assert_eq!(light.members[1].name, "intensity");
    assert_eq!(light.members[1].ty.name, "float");

    let normal = global_var(&root, "vNormal");
    assert_eq!(normal.qualifiers, vec![GlslQualifier::In]);
    assert_eq!(layout_names(normal), vec![("location", Some(0))]);
    assert_eq!(normal.ty.as_ref().map(|ty| ty.name.as_str()), Some("vec3"));

    let color = global_var(&root, "fragColor");
    assert_eq!(color.qualifiers, vec![GlslQualifier::Out]);
    assert_eq!(layout_names(color), vec![("location", Some(0))]);
    assert_eq!(color.ty.as_ref().map(|ty| ty.name.as_str()), Some("vec4"));

    let model = global_var(&root, "uModel");
    assert_eq!(model.qualifiers, vec![GlslQualifier::Uniform]);
    assert_eq!(model.ty.as_ref().map(|ty| ty.name.as_str()), Some("mat4"));

    let texture = global_var(&root, "uTexture");
    assert_eq!(texture.ty.as_ref().map(|ty| ty.name.as_str()), Some("sampler2D"));

    match &root.items[5] {
        GlslItem::Const(constant) => {
            assert_eq!(constant.name, "PI");
            assert!(matches!(constant.init, GlslExpression::FloatLiteral(value) if (value - 3.14159).abs() < 1e-9));
        }
        other => panic!("expected the `PI` constant, got {other:?}"),
    }

    let shade = function(&root, "shade");
    assert_eq!(shade.params.len(), 2);
    assert_eq!(shade.params[0].ty.name, "Light");
    assert_eq!(shade.params[1].ty.name, "vec3");
    assert_eq!(shade.body.len(), 3);

    let main = function(&root, "main");
    assert_eq!(main.body.len(), 6, "unexpected body: {:#?}", main.body);
    assert!(matches!(&main.body[0], GlslStatement::Var { name, ty: Some(ty), .. } if name == "light" && ty.name == "Light"));
    assert!(matches!(&main.body[1], GlslStatement::Assign { target: GlslExpression::Member { field, .. }, .. } if field == "position"));
    assert!(matches!(
        &main.body[4],
        GlslStatement::Assign { target: GlslExpression::Ident(name), value: GlslExpression::Call { callee, args }, .. }
            if name == "fragColor" && callee == "texture" && args.len() == 2
    ));
    match &main.body[5] {
        GlslStatement::If { cond: GlslExpression::Paren(cond), then_body, .. } => {
            assert!(matches!(cond.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Lt, lhs, .. } if matches!(lhs.as_ref(), GlslExpression::Member { field, .. } if field == "x")));
            assert!(matches!(then_body.as_slice(), [GlslStatement::Discard]));
        }
        other => panic!("expected the `discard` branch, got {other:?}"),
    }
}

#[test]
fn parses_expression_precedence() {
    let source = "void main() { float c = 1.0 + 2.0 * 3.0; float d = (1.0 + 2.0) * 3.0; uint m = 0xFFu; int n = (1 << 2) | 3 & 4; }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let main = function(&root, "main");
    match &main.body[0] {
        GlslStatement::Var { init: Some(GlslExpression::Binary { op: GlslBinaryOp::Add, rhs, .. }), .. } => {
            assert!(matches!(rhs.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Mul, .. }));
        }
        other => panic!("expected `1.0 + 2.0 * 3.0`, got {other:?}"),
    }
    match &main.body[1] {
        GlslStatement::Var { init: Some(GlslExpression::Binary { op: GlslBinaryOp::Mul, lhs, .. }), .. } => {
            assert!(matches!(lhs.as_ref(), GlslExpression::Paren(inner) if matches!(inner.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Add, .. })));
        }
        other => panic!("expected `(1.0 + 2.0) * 3.0`, got {other:?}"),
    }
    assert!(matches!(&main.body[2], GlslStatement::Var { init: Some(GlslExpression::IntegerLiteral(255)), .. }));
    match &main.body[3] {
        GlslStatement::Var { init: Some(GlslExpression::Binary { op: GlslBinaryOp::BitOr, lhs, rhs }), .. } => {
            assert!(matches!(lhs.as_ref(), GlslExpression::Paren(inner) if matches!(inner.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::Shl, .. })));
            assert!(matches!(rhs.as_ref(), GlslExpression::Binary { op: GlslBinaryOp::BitAnd, .. }));
        }
        other => panic!("expected a bitwise expression, got {other:?}"),
    }
}

#[test]
fn reports_truncated_function() {
    let (root, diagnostics) = build("void main(");
    assert!(!diagnostics.is_empty(), "a truncated function must report diagnostics");
    assert_eq!(root.items.len(), 1);
    assert_eq!(function(&root, "main").name, "main");
}

#[test]
fn reports_unknown_construct_and_recovers() {
    let (root, diagnostics) = build("@ @ void main() { }");
    assert_eq!(diagnostics.len(), 2, "each stray token must be reported: {diagnostics:?}");
    assert_eq!(root.items.len(), 1);
    assert_eq!(function(&root, "main").name, "main");
}
