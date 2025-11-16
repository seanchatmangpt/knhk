//! Workflow DSL parser
//!
//! Parses the workflow! macro input into an AST.

use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, token, Ident, LitInt, Result, Token};

/// Workflow definition AST
#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub name: Ident,
    pub patterns: Vec<Ident>,
    pub states: Vec<StateTransition>,
    pub constraints: WorkflowConstraints,
}

/// State transition in workflow
#[derive(Debug, Clone)]
pub enum StateTransition {
    /// Single state to single state: A -> B
    Simple {
        from: Ident,
        to: Ident,
    },
    /// Single state to multiple states: A -> [B, C]
    Split {
        from: Ident,
        to: Vec<Ident>,
    },
    /// Multiple states to single state: [A, B] -> C
    Join {
        from: Vec<Ident>,
        to: Ident,
    },
    /// Multiple states to multiple states: [A, B] -> [C, D]
    Complex {
        from: Vec<Ident>,
        to: Vec<Ident>,
    },
}

/// Workflow constraints
#[derive(Debug, Clone, Default)]
pub struct WorkflowConstraints {
    pub max_duration: Option<u64>,
    pub max_concurrency: Option<usize>,
    pub patterns_used: Option<Vec<usize>>,
}

impl Parse for WorkflowDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut patterns = Vec::new();
        let mut states = Vec::new();
        let mut constraints = WorkflowConstraints::default();

        // Parse key-value pairs
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                "patterns" => {
                    let content;
                    bracketed!(content in input);
                    while !content.is_empty() {
                        patterns.push(content.parse()?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                "states" => {
                    let content;
                    braced!(content in input);
                    while !content.is_empty() {
                        states.push(parse_state_transition(&content)?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                "constraints" => {
                    constraints = parse_constraints(input)?;
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown workflow field: {}", key),
                    ))
                }
            }
        }

        let name = name.ok_or_else(|| input.error("Missing 'name' field"))?;

        Ok(WorkflowDefinition {
            name,
            patterns,
            states,
            constraints,
        })
    }
}

fn parse_state_transition(input: ParseStream) -> Result<StateTransition> {
    // Parse left side (from states)
    let from: Vec<Ident> = if input.peek(token::Bracket) {
        let content;
        bracketed!(content in input);
        let mut states = Vec::new();
        while !content.is_empty() {
            states.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        states
    } else {
        vec![input.parse()?]
    };

    // Parse arrow
    input.parse::<Token![->]>()?;

    // Parse right side (to states)
    let to: Vec<Ident> = if input.peek(token::Bracket) {
        let content;
        bracketed!(content in input);
        let mut states = Vec::new();
        while !content.is_empty() {
            states.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        states
    } else {
        vec![input.parse()?]
    };

    // Determine transition type
    Ok(match (from.len(), to.len()) {
        (1, 1) => StateTransition::Simple {
            from: from[0].clone(),
            to: to[0].clone(),
        },
        (1, _) => StateTransition::Split {
            from: from[0].clone(),
            to,
        },
        (_, 1) => StateTransition::Join {
            from,
            to: to[0].clone(),
        },
        (_, _) => StateTransition::Complex { from, to },
    })
}

fn parse_constraints(input: ParseStream) -> Result<WorkflowConstraints> {
    let content;
    braced!(content in input);

    let mut constraints = WorkflowConstraints::default();

    while !content.is_empty() {
        let key: Ident = content.parse()?;
        content.parse::<Token![:]>()?;

        match key.to_string().as_str() {
            "max_duration" => {
                let lit: LitInt = content.parse()?;
                constraints.max_duration = Some(lit.base10_parse()?);
            }
            "max_concurrency" => {
                let lit: LitInt = content.parse()?;
                constraints.max_concurrency = Some(lit.base10_parse()?);
            }
            "patterns_used" => {
                let array_content;
                bracketed!(array_content in content);
                let mut patterns = Vec::new();
                while !array_content.is_empty() {
                    let lit: LitInt = array_content.parse()?;
                    patterns.push(lit.base10_parse()?);
                    if array_content.peek(Token![,]) {
                        array_content.parse::<Token![,]>()?;
                    }
                }
                constraints.patterns_used = Some(patterns);
            }
            _ => {
                return Err(syn::Error::new(
                    key.span(),
                    format!("Unknown constraint: {}", key),
                ))
            }
        }

        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }

    Ok(constraints)
}
