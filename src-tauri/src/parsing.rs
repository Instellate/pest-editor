use pest::error::{ErrorVariant, LineColLocation};
use pest::iterators::Pair;
use pest_vm::Vm;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ParserState {
    vm: std::sync::Mutex<Option<pest_vm::Vm>>,
}

#[derive(Serialize, Deserialize, specta::Type)]
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

#[specta::specta]
#[tauri::command]
pub fn update_pest_grammar(
    state: tauri::State<'_, ParserState>,
    grammar: String,
) -> Result<Vec<String>, Vec<PestGrammarError>> {
    let (_, rules) = match pest_meta::parse_and_optimize(&grammar) {
        Ok(r) => r,
        Err(e) => {
            let mut formatted_errors = vec![];
            for error in e {
                formatted_errors.push(PestGrammarError::from(
                    error.renamed_rules(pest_meta::parser::rename_meta_rule),
                ));
            }

            return Err(formatted_errors);
        }
    };

    let names = rules.iter().map(|r| r.name.clone()).collect();

    let mut vm = state.vm.lock().unwrap();
    *vm = Some(Vm::new(rules));

    Ok(names)
}

#[derive(specta::Type, Deserialize, Serialize)]
pub struct TokenTree {
    label: String,
    children: Vec<TokenTree>,
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
                label: format!("{}: {}", rule_name, value.as_str()),
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

    Ok(Some(pairs.into_iter().next().unwrap().into()))
}
