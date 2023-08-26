use oak_core::{Language, ParseSession, Parser, SourceText};
use oak_valkyrie::{ValkyrieLanguage, ValkyrieParser};

#[ignore = "Valkyrie ECS extension test is out of date"]
#[test]
fn test_ecs_extension() {
    // Test ECS extension with support enabled
    let language = ValkyrieLanguage::ecs_language();
    let parser = ValkyrieParser::new(&language);
    let source = SourceText::new(
        r#"
        component Position {
            x: f32,
            y: f32
        }

        component Player {
            name: String,
            health: i32,
            max_health: i32,
            events on_health_change: micro(current: i32, max: i32) -> (),
            events on_death: micro() -> ()
        }

        system MovementSystem {
            micro execute(world: World): Result<()> {
                # system logic
                return Ok()
            }
        }

        system PlayerSystem {
            micro execute(world: World): Result<()> {
                # system logic
                return Ok()
            }
        }
    "#,
    );

    let mut session = ParseSession::new(1024);
    let output = parser.parse(&source, &[], &mut session);
    assert!(output.result.is_ok(), "Parser should succeed with ECS extension enabled");

    // Test ECS extension with support disabled
    let language_disabled = ValkyrieLanguage::default();
    let parser_disabled = ValkyrieParser::new(&language_disabled);
    let output_disabled = parser_disabled.parse(&source, &[], &mut session);
    // This should fail because ECS extension is disabled
    assert!(output_disabled.result.is_err(), "Parser should fail with ECS extension disabled");
}
