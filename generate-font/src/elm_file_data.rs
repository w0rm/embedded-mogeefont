/// Parse Elm file to get the following data:
/// - emHeight and spaceWidth (number constants)
/// - defaultBearings (tuple)
/// - bearingsDict (Dict.fromList)
/// - leftKerningClass (Dict.fromList)
/// - rightKerningClass (Dict.fromList)
/// - kerningPairs (Dict.fromList)
/// - kerningOverrides (Dict.fromList)
use std::path::Path;

pub struct ElmFileData {
    pub em_height: u32,
    pub space_width: u32,
    pub default_bearings: (i32, i32),
    pub bearings: Vec<(String, i32, i32)>,
    pub left_kerning_class: Vec<(usize, Vec<String>)>,
    pub right_kerning_class: Vec<(usize, Vec<String>)>,

    pub kering_pairs: Vec<(usize, usize, i32)>,
    pub kerning_overrides: Vec<(String, String, i32)>,
}

/// emHeight =
///     11
const NUMBER_CONSTANT: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (number_constant_expr) @number_literal
    (#match? @lower_case_identifier "^(emHeight|spaceWidth)$")
)"#;

/// defaultBearings =
///     ( 0, 1 )
const DEFAULT_BEARINGS: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (tuple_expr
        expr: (number_constant_expr (number_literal) @left_bearing)
        expr: (number_constant_expr (number_literal) @right_bearing)
    )
    (#eq? @lower_case_identifier "defaultBearings")
)"#;

/// bearingsDict =
///     Dict.fromList
///         [ ( "j", ( -2, 1 ) )
///         , ( "jj", ( -2, 1 ) )
///         ]
const BEARINGS: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (function_call_expr
        arg: (list_expr
            exprList: (tuple_expr
                expr: (string_constant_expr (regular_string_part) @name)
                expr: (tuple_expr
                    expr: (number_constant_expr) @left_bearing
                    expr: (number_constant_expr) @right_bearing
                )
            )
        )
    )
    (#eq? @lower_case_identifier "bearingsDict")
)"#;

/// leftKerningClass =
///     invertDict
///         [ ( 1, [ "A", "B", "C", "D" ] )
///         , ( 2, [ [ "F", "P" ] )
///         ]
const KERNING_CLASS: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (function_call_expr
        arg: (list_expr
            exprList: (tuple_expr
                expr: (number_constant_expr) @class
                expr: (list_expr) @chars
            )
        )
    )
    (#match? @lower_case_identifier "^(leftKerningClass|rightKerningClass)$")
)"#;

/// kerningDict =
///     Dict.fromList
///         [ ( ( 1, 14 ), -1 )
///         , ( ( 2, 2 ), -1 )
///         ]
const KERNING_PAIRS: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (function_call_expr
        arg: (list_expr
            exprList: (tuple_expr
                expr: (tuple_expr
                    expr: (number_constant_expr) @left_class
                    expr: (number_constant_expr) @right_class
                )
                expr: (number_constant_expr) @kerning
            )
        )
    )
    (#eq? @lower_case_identifier "kerningDict")
)"#;

/// kerningOverrides =
///     Dict.fromList
///         [ ( ( "\\", "\\" ), -2 )
///         , ( ( "C", "f" ), -1 )
///         ]
const KERNING_OVERRIDES: &str = r#"
(value_declaration
    functionDeclarationLeft: (function_declaration_left) @lower_case_identifier
    body: (function_call_expr
        arg: (list_expr
            exprList: (tuple_expr
                expr: (tuple_expr
                    expr: (string_constant_expr (regular_string_part) @left_char)
                    expr: (string_constant_expr (regular_string_part) @right_char)
                )
                expr: (number_constant_expr) @kerning
            )
        )
    )
    (#eq? @lower_case_identifier "kerningOverrides")
)"#;

impl<P> From<P> for ElmFileData
where
    P: AsRef<Path>,
{
    fn from(elm_file: P) -> Self {
        let elm_code = std::fs::read(elm_file).unwrap();
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_elm::language();
        parser.set_language(language).unwrap();
        let tree = parser.parse(&elm_code, None).unwrap();
        let mut cursor = tree_sitter::QueryCursor::new();

        // emHeight and spaceWidth
        let query =
            tree_sitter::Query::new(language, NUMBER_CONSTANT).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut em_height = 0;
        let mut space_width = 0;
        for m in matches {
            let value = m.captures[1]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            match m.captures[0].node.utf8_text(&elm_code) {
                Ok("emHeight") => em_height = value,
                Ok("spaceWidth") => space_width = value,
                _ => {}
            }
        }

        // Default bearings
        let query =
            tree_sitter::Query::new(language, DEFAULT_BEARINGS).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut left_default_bearing: i32 = 0;
        let mut right_default_bearing: i32 = 0;
        let captures = matches.into_iter().next().unwrap().captures;
        left_default_bearing = captures[1]
            .node
            .utf8_text(&elm_code)
            .unwrap()
            .parse()
            .unwrap();
        right_default_bearing = captures[2]
            .node
            .utf8_text(&elm_code)
            .unwrap()
            .parse()
            .unwrap();

        // Bearings
        let query = tree_sitter::Query::new(language, BEARINGS).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut bearings: Vec<(String, i32, i32)> = Vec::new();
        for m in matches {
            let char = m.captures[1].node.utf8_text(&elm_code).unwrap();
            let left_bearing = m.captures[2]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            let right_bearing = m.captures[3]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            bearings.push((
                unescape::unescape(char).unwrap().to_string(),
                left_bearing,
                right_bearing,
            ));
        }

        // Kerning class
        let query =
            tree_sitter::Query::new(language, KERNING_CLASS).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut left_kerning_class: Vec<(usize, Vec<String>)> = Vec::new();
        let mut right_kerning_class: Vec<(usize, Vec<String>)> = Vec::new();
        for m in matches {
            let class = m.captures[1]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            // exprList: (string_constant_expr (regular_string_part)+ @chars)
            let chars = m.captures[2]
                .node
                .children_by_field_name("exprList", &mut m.captures[2].node.walk())
                .map(|expr| {
                    let str = expr
                        .children(&mut expr.walk())
                        .nth(1)
                        .unwrap()
                        .utf8_text(&elm_code)
                        .unwrap();
                    unescape::unescape(str).unwrap().to_string()
                })
                .collect::<Vec<String>>();

            match m.captures[0].node.utf8_text(&elm_code) {
                Ok("leftKerningClass") => left_kerning_class.push((class, chars)),
                Ok("rightKerningClass") => right_kerning_class.push((class, chars)),
                _ => {}
            }
        }

        // Kerning pairs
        let query =
            tree_sitter::Query::new(language, KERNING_PAIRS).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut kering_pairs: Vec<(usize, usize, i32)> = Vec::new();
        for m in matches {
            let left_class = m.captures[1]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            let right_class = m.captures[2]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            let kerning = m.captures[3]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            kering_pairs.push((left_class, right_class, kerning));
        }

        // Kerning overrides
        let query =
            tree_sitter::Query::new(language, KERNING_OVERRIDES).expect("Failed to create query");
        let matches = cursor.matches(&query, tree.root_node(), elm_code.as_slice());
        let mut kerning_overrides: Vec<(String, String, i32)> = Vec::new();
        for m in matches {
            let left_char = m.captures[1].node.utf8_text(&elm_code).unwrap().to_string();
            let right_char = m.captures[2].node.utf8_text(&elm_code).unwrap().to_string();
            let kerning = m.captures[3]
                .node
                .utf8_text(&elm_code)
                .unwrap()
                .parse()
                .unwrap();
            kerning_overrides.push((
                unescape::unescape(&left_char).unwrap().to_string(),
                unescape::unescape(&right_char).unwrap().to_string(),
                kerning,
            ));
        }

        ElmFileData {
            em_height,
            space_width,
            default_bearings: (left_default_bearing, right_default_bearing),
            bearings,
            left_kerning_class,
            right_kerning_class,
            kering_pairs,
            kerning_overrides,
        }
    }
}
