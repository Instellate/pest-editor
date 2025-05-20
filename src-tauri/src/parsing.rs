use std::collections::HashMap;

use pest::error::{ErrorVariant, LineColLocation};
use pest::iterators::Pair;
use pest_vm::Vm;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ParserState {
    pub vm: std::sync::Mutex<Option<pest_vm::Vm>>,
    pub references: std::sync::Mutex<HashMap<String, Vec<Location>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Location {
    start_line: i32,
    start_col: i32,
    end_line: i32,
    end_col: i32,
}

impl From<LineColLocation> for Location {
    fn from(value: LineColLocation) -> Self {
        match value {
            LineColLocation::Pos((l, c)) => Self {
                start_line: l.try_into().unwrap(),
                start_col: c.try_into().unwrap(),
                end_line: l.try_into().unwrap(),
                end_col: c.try_into().unwrap(),
            },
            LineColLocation::Span((l1, c1), (l2, c2)) => Self {
                start_line: l1.try_into().unwrap(),
                start_col: c1.try_into().unwrap(),
                end_line: l2.try_into().unwrap(),
                end_col: c2.try_into().unwrap(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct PestGrammarError {
    message: String,
    location: Location,
}

impl<T> From<pest::error::Error<T>> for PestGrammarError {
    fn from(value: pest::error::Error<T>) -> Self {
        let message = match value.variant {
            ErrorVariant::CustomError { message } => message,

            _ => unreachable!(),
        };

        let location = Location::from(value.line_col);

        Self { message, location }
    }
}

impl From<pest::Span<'_>> for Location {
    fn from(value: pest::Span<'_>) -> Self {
        let (start_line, start_col) = value.start_pos().line_col();
        let (end_line, end_col) = value.end_pos().line_col();

        Self {
            start_line: start_line.try_into().unwrap(),
            start_col: start_col.try_into().unwrap(),
            end_line: end_line.try_into().unwrap(),
            end_col: end_col.try_into().unwrap(),
        }
    }
}

fn update_references(
    state: tauri::State<'_, ParserState>,
    rules: pest::iterators::Pairs<'_, pest_meta::parser::Rule>,
) {
    for rule in rules {
        if rule.as_rule() == pest_meta::parser::Rule::identifier {
            let mut map = state.references.lock().unwrap();

            if let Some(span_vec) = map.get_mut(rule.as_str()) {
                span_vec.push(rule.as_span().into());
            } else {
                map.insert(rule.as_str().to_string(), vec![rule.as_span().into()]);
            }
        } else {
            let inner = rule.into_inner();
            update_references(state.clone(), inner);
        }
    }
}

#[specta::specta]
#[tauri::command]
pub fn update_pest_grammar(
    state: tauri::State<'_, ParserState>,
    grammar: String,
) -> Result<Vec<String>, Vec<PestGrammarError>> {
    let pairs = match pest_meta::parser::parse(pest_meta::parser::Rule::grammar_rules, &grammar) {
        Ok(r) => r,
        Err(e) => {
            return Err(vec![e
                .renamed_rules(pest_meta::parser::rename_meta_rule)
                .into()]);
        }
    };

    if let Err(e) = pest_meta::validator::validate_pairs(pairs.clone()) {
        return Err(e
            .into_iter()
            .map(|err| {
                err.renamed_rules(pest_meta::parser::rename_meta_rule)
                    .into()
            })
            .collect());
    };

    let ast = match pest_meta::parser::consume_rules(pairs.clone()) {
        Ok(a) => a,
        Err(e) => {
            return Err(e
                .into_iter()
                .map(|err| {
                    err.renamed_rules(pest_meta::parser::rename_meta_rule)
                        .into()
                })
                .collect())
        }
    };

    *state.references.lock().unwrap() = Default::default();
    update_references(state.clone(), pairs);

    let rules = pest_meta::optimizer::optimize(ast);
    let names = rules.iter().map(|r| r.name.clone()).collect();

    let mut vm = state.vm.lock().unwrap();
    *vm = Some(Vm::new(rules));

    Ok(names)
}

#[derive(Debug, specta::Type, Deserialize, Serialize)]
pub struct TokenTree {
    label: String,
    children: Vec<TokenTree>,
}

impl TokenTree {
    pub fn new(label: String, children: Vec<TokenTree>) -> Self {
        Self { label, children }
    }
}

impl<'a> From<Pair<'a, &'a str>> for TokenTree {
    fn from(value: Pair<'a, &'a str>) -> Self {
        let rule_name = value.as_rule();

        let inner = value.clone().into_inner();
        if inner.len() > 0 {
            TokenTree {
                label: rule_name.into(),
                children: inner.map(|p| p.into()).collect(),
            }
        } else {
            TokenTree {
                label: format!("{}\n{}", rule_name, value.as_str()),
                children: vec![],
            }
        }
    }
}

#[specta::specta]
#[tauri::command]
pub fn parse_input(
    state: tauri::State<'_, ParserState>,
    input: String,
    rule: String,
) -> Result<Option<TokenTree>, PestGrammarError> {
    let vm = state.vm.lock().unwrap();

    let Some(vm) = vm.as_ref() else {
        return Ok(None);
    };

    let pairs = match vm.parse(&rule, &input) {
        Ok(p) => p,
        Err(e) => {
            let error = e.renamed_rules(|r| r.to_string());
            return Err(PestGrammarError::from(error));
        }
    };

    let mut children = vec![];
    for pair in pairs {
        children.push(pair.into());
    }

    Ok(Some(TokenTree::new(rule, children)))
}

#[specta::specta]
#[tauri::command]
pub fn find_rule_references(
    state: tauri::State<'_, ParserState>,
    rule_name: String,
) -> Option<Vec<Location>> {
    let map = state.references.lock().unwrap();

    map.get(&rule_name).cloned()
}

#[specta::specta]
#[tauri::command]
pub fn get_all_rules(state: tauri::State<'_, ParserState>) -> Vec<String> {
    let map = state.references.lock().unwrap();

    map.keys().cloned().collect()
}
