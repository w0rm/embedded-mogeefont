use std::{collections::BTreeMap, fmt::Debug, path::Path, str::FromStr};
use tree_sitter::{Node, Parser, Query, QueryCapture, QueryCursor};

/// Parse Elm file to get the following data:
/// - emHeight and spaceWidth (number constants)
/// - defaultBearings (tuple)
/// - bearingsDict (Dict.fromList)
/// - leftKerningClass (Dict.fromList)
/// - rightKerningClass (Dict.fromList)
/// - kerningPairs (Dict.fromList)
/// - kerningOverrides (Dict.fromList)
pub struct ElmFileData {
    pub em_height: u32,
    pub space_width: u32,
    pub default_bearings: (i8, i8),
    pub bearings: BTreeMap<String, (i8, i8)>,
    pub left_kerning_class: Vec<(u8, Vec<String>)>,
    pub right_kerning_class: Vec<(u8, Vec<String>)>,
    pub kering_pairs: Vec<(u8, u8, i8)>,
    pub kerning_overrides: Vec<(String, String, i8)>,
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

/// ["a", "b", "c"]
const STRING_LIST: &str = r#"
(list_expr
    exprList: (string_constant_expr (regular_string_part) @string)
)"#;

impl TryFrom<&Path> for ElmFileData {
    type Error = std::io::Error;

    fn try_from(elm_file: &Path) -> std::io::Result<Self> {
        let elm_code = std::fs::read(elm_file)?;
        let elm_code = elm_code.as_slice();
        let language = tree_sitter_elm::language();
        let tree = {
            let mut parser = Parser::new();
            parser.set_language(language).unwrap();
            parser.parse(&elm_code, None).unwrap()
        };
        let root_node = tree.root_node();
        let mut cursor = QueryCursor::new();

        // emHeight and spaceWidth
        let query = Query::new(language, NUMBER_CONSTANT).expect("Failed to create query");
        let matches = cursor.matches(&query, root_node, elm_code);
        let mut em_height = 0;
        let mut space_width = 0;
        for m in matches {
            let value = m.captures[1].parse(elm_code);
            match m.captures[0].node.utf8_text(elm_code) {
                Ok("emHeight") => em_height = value,
                Ok("spaceWidth") => space_width = value,
                _ => {}
            }
        }

        // Default bearings
        let query = Query::new(language, DEFAULT_BEARINGS).expect("Failed to create query");
        let matches = cursor.matches(&query, root_node, elm_code);
        let captures = matches.into_iter().next().unwrap().captures;
        let left_default_bearing = captures[1].parse(elm_code);
        let right_default_bearing = captures[2].parse(elm_code);

        // Bearings
        let query = Query::new(language, BEARINGS).expect("Failed to create query");
        let bearings = cursor
            .matches(&query, root_node, elm_code)
            .map(|m| {
                let char = m.captures[1].to_string(elm_code);
                let left_bearing = m.captures[2].parse(elm_code);
                let right_bearing = m.captures[3].parse(elm_code);
                (char, (left_bearing, right_bearing))
            })
            .collect();

        // Kerning class
        let query = Query::new(language, KERNING_CLASS).expect("Failed to create query");
        let mut left_kerning_class = Vec::new();
        let mut right_kerning_class = Vec::new();
        for m in cursor.matches(&query, root_node, elm_code) {
            let class = m.captures[1].parse(elm_code);
            let arr_query = Query::new(language, STRING_LIST).expect("Failed to create query");
            let chars = QueryCursor::new()
                .matches(&arr_query, m.captures[2].node, elm_code)
                .map(|m| m.captures[0].to_string(elm_code))
                .collect::<Vec<String>>();
            match m.captures[0].node.utf8_text(elm_code) {
                Ok("leftKerningClass") => left_kerning_class.push((class, chars)),
                Ok("rightKerningClass") => right_kerning_class.push((class, chars)),
                _ => {}
            }
        }

        // Kerning pairs
        let query = Query::new(language, KERNING_PAIRS).expect("Failed to create query");
        let kering_pairs = cursor
            .matches(&query, root_node, elm_code)
            .map(|m| {
                let left_class = m.captures[1].parse(elm_code);
                let right_class = m.captures[2].parse(elm_code);
                let kerning = m.captures[3].parse(elm_code);
                (left_class, right_class, kerning)
            })
            .collect();

        // Kerning overrides
        let query = Query::new(language, KERNING_OVERRIDES).expect("Failed to create query");
        let kerning_overrides = cursor
            .matches(&query, root_node, elm_code)
            .map(|m| {
                let left_char = m.captures[1].to_string(elm_code);
                let right_char = m.captures[2].to_string(elm_code);
                let kerning = m.captures[3].parse(elm_code);
                (left_char, right_char, kerning)
            })
            .collect();

        Ok(ElmFileData {
            em_height,
            space_width,
            default_bearings: (left_default_bearing, right_default_bearing),
            bearings,
            left_kerning_class,
            right_kerning_class,
            kering_pairs,
            kerning_overrides,
        })
    }
}

trait Reader<'a> {
    fn to_string(&self, source: &'a [u8]) -> String;

    fn parse<F: FromStr>(&self, source: &'a [u8]) -> F
    where
        <F as FromStr>::Err: Debug;
}

impl<'a> Reader<'a> for Node<'a> {
    fn to_string(&self, source: &[u8]) -> String {
        unescape::unescape(self.utf8_text(source).unwrap()).unwrap()
    }
    fn parse<F: FromStr>(&self, source: &[u8]) -> F
    where
        <F as FromStr>::Err: Debug,
    {
        self.to_string(source).parse().unwrap()
    }
}

impl<'a> Reader<'a> for QueryCapture<'a> {
    fn to_string(&self, source: &[u8]) -> String {
        self.node.to_string(source)
    }
    fn parse<F: FromStr>(&self, source: &[u8]) -> F
    where
        <F as FromStr>::Err: Debug,
    {
        self.node.parse(source)
    }
}
