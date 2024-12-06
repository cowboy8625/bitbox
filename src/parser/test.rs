use crate::lexer::lex;
use crate::parser;

pub fn snapshot_parsing(input: &str) -> String {
    let tokens = lex(input);
    let program = parser::Parser::new(tokens).parse();
    format!("{:#?}", program)
}

macro_rules! snapshot {
    ($name:tt, $path:tt) => {
        #[test]
        fn $name() {
            let contents = include_str!($path);
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path("testdata/output/");
            settings.bind(|| {
                insta::assert_snapshot!(snapshot_parsing(contents));
            });
        }
    };
}

snapshot!(binary, "../../snapshots/basic.bitbox");

