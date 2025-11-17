//! Code Generator
//!
//! Generates executable code from validated patterns.
//! Produces dispatch tables, guard code, and receipt templates.

use crate::compiler::extractor::{DataType, ExtractedPattern, Guard, GuardType, Variable};
use crate::error::{WorkflowError, WorkflowResult};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use tracing::{debug, info, instrument};

/// Generated code structure
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Pattern dispatch table
    pub dispatch_table: DispatchTable,
    /// Generated guards
    pub guards: Vec<GeneratedGuard>,
    /// Receipt templates
    pub receipts: Vec<ReceiptTemplate>,
    /// Symbol table
    pub symbols: SymbolTable,
    /// Constants
    pub constants: Vec<Constant>,
    /// Metadata
    pub metadata: CodeMetadata,
}

/// Pattern dispatch table
#[derive(Debug, Clone)]
pub struct DispatchTable {
    /// Table entries
    pub entries: Vec<DispatchEntry>,
    /// Jump table for fast dispatch
    pub jump_table: Vec<u32>,
    /// Pattern index map
    pub pattern_map: HashMap<u8, usize>,
}

/// Dispatch table entry
#[derive(Debug, Clone)]
pub struct DispatchEntry {
    /// Pattern ID
    pub pattern_id: u8,
    /// Entry point offset
    pub entry_point: u32,
    /// Guard list offset
    pub guard_offset: u32,
    /// Variable table offset
    pub var_table_offset: u32,
    /// Receipt template offset
    pub receipt_offset: u32,
    /// Flags
    pub flags: u32,
}

/// Generated guard code
#[derive(Debug, Clone)]
pub struct GeneratedGuard {
    /// Guard ID
    pub id: String,
    /// Bytecode
    pub bytecode: Vec<u8>,
    /// Variable references
    pub var_refs: Vec<u16>,
    /// Constants used
    pub const_refs: Vec<u16>,
    /// Stack depth required
    pub stack_depth: u8,
}

/// Receipt template
#[derive(Debug, Clone)]
pub struct ReceiptTemplate {
    /// Pattern ID
    pub pattern_id: u8,
    /// Template format
    pub format: ReceiptFormat,
    /// Field definitions
    pub fields: Vec<ReceiptField>,
    /// Size in bytes
    pub size: u32,
}

/// Receipt format
#[derive(Debug, Clone)]
pub enum ReceiptFormat {
    Fixed,
    Variable,
    Compressed,
}

/// Receipt field
#[derive(Debug, Clone)]
pub struct ReceiptField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Offset in receipt
    pub offset: u16,
    /// Size in bytes
    pub size: u16,
}

/// Field type
#[derive(Debug, Clone)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Timestamp,
}

/// Symbol table
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Variable symbols
    pub variables: HashMap<String, Symbol>,
    /// Function symbols
    pub functions: HashMap<String, Symbol>,
    /// Type symbols
    pub types: HashMap<String, Symbol>,
    /// Next available ID
    next_id: u16,
}

/// Symbol entry
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol ID
    pub id: u16,
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub symbol_type: SymbolType,
    /// Offset/address
    pub offset: u32,
}

/// Symbol type
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(DataType),
    Function,
    Type,
}

/// Constant
#[derive(Debug, Clone)]
pub struct Constant {
    /// Constant ID
    pub id: u16,
    /// Value
    pub value: ConstantValue,
}

/// Constant value
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

/// Code metadata
#[derive(Debug, Clone)]
pub struct CodeMetadata {
    /// Total code size
    pub code_size: usize,
    /// Data size
    pub data_size: usize,
    /// Stack size required
    pub stack_size: usize,
    /// Optimization level
    pub optimization_level: u8,
}

/// Code generator
pub struct CodeGenerator {
    /// Current offset
    current_offset: u32,
    /// Label counter
    label_counter: u32,
    /// Optimization hints
    optimization_hints: Vec<OptimizationHint>,
}

/// Optimization hint
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    /// Hint type
    pub hint_type: HintType,
    /// Location
    pub location: u32,
    /// Data
    pub data: String,
}

/// Hint type
#[derive(Debug, Clone)]
pub enum HintType {
    Inline,
    NoInline,
    HotPath,
    ColdPath,
    Constant,
    Pure,
}

impl CodeGenerator {
    /// Create new code generator
    pub fn new() -> Self {
        Self {
            current_offset: 0,
            label_counter: 0,
            optimization_hints: Vec::new(),
        }
    }

    /// Generate code from patterns
    #[instrument(skip(self, patterns))]
    pub async fn generate(
        &mut self,
        patterns: &[ExtractedPattern],
    ) -> WorkflowResult<GeneratedCode> {
        info!("Generating code for {} patterns", patterns.len());

        // Initialize symbol table
        let mut symbols = SymbolTable::new();

        // Generate dispatch table
        let dispatch_table = self.generate_dispatch_table(patterns)?;

        // Generate guards
        let guards = self.generate_guards(patterns, &mut symbols)?;

        // Generate receipt templates
        let receipts = self.generate_receipts(patterns)?;

        // Collect constants
        let constants = self.collect_constants(patterns)?;

        // Calculate metadata
        let metadata = self.calculate_metadata(&dispatch_table, &guards, &receipts);

        Ok(GeneratedCode {
            dispatch_table,
            guards,
            receipts,
            symbols,
            constants,
            metadata,
        })
    }

    /// Generate dispatch table
    fn generate_dispatch_table(
        &mut self,
        patterns: &[ExtractedPattern],
    ) -> WorkflowResult<DispatchTable> {
        debug!("Generating dispatch table");

        let mut entries = Vec::new();
        let mut jump_table = vec![0u32; 44]; // 43 patterns + 1 for 0-index
        let mut pattern_map = HashMap::new();

        for (index, pattern) in patterns.iter().enumerate() {
            let entry_point = self.current_offset;
            self.current_offset += 64; // Each entry is 64 bytes (cache line)

            let guard_offset = if !pattern.guards.is_empty() {
                let offset = self.current_offset;
                self.current_offset += (pattern.guards.len() as u32) * 32;
                offset
            } else {
                0
            };

            let var_table_offset = if !pattern.variables.is_empty() {
                let offset = self.current_offset;
                self.current_offset += (pattern.variables.len() as u32) * 24;
                offset
            } else {
                0
            };

            let receipt_offset = self.current_offset;
            self.current_offset += 128; // Receipt template size

            let flags = self.calculate_pattern_flags(pattern);

            let entry = DispatchEntry {
                pattern_id: pattern.pattern_id,
                entry_point,
                guard_offset,
                var_table_offset,
                receipt_offset,
                flags,
            };

            entries.push(entry.clone());
            jump_table[pattern.pattern_id as usize] = entry_point;
            pattern_map.insert(pattern.pattern_id, index);
        }

        Ok(DispatchTable {
            entries,
            jump_table,
            pattern_map,
        })
    }

    /// Calculate pattern flags
    fn calculate_pattern_flags(&self, pattern: &ExtractedPattern) -> u32 {
        let mut flags = 0u32;

        // Has guards
        if !pattern.guards.is_empty() {
            flags |= 1 << 0;
        }

        // Has timeout
        if pattern.timeout_ms.is_some() {
            flags |= 1 << 1;
        }

        // Is loop
        if pattern.pattern_type == crate::compiler::extractor::PatternType::ArbitraryLoop {
            flags |= 1 << 2;
        }

        // Has multiple instances
        if pattern.pattern_type == crate::compiler::extractor::PatternType::MultipleInstance {
            flags |= 1 << 3;
        }

        // Has event handlers
        if !pattern.event_handlers.is_empty() {
            flags |= 1 << 4;
        }

        // Critical path (hot)
        if pattern.tick_budget <= 4 {
            flags |= 1 << 5;
            self.add_hint(HintType::HotPath, self.current_offset, "Critical path");
        }

        flags
    }

    /// Generate guards
    fn generate_guards(
        &mut self,
        patterns: &[ExtractedPattern],
        symbols: &mut SymbolTable,
    ) -> WorkflowResult<Vec<GeneratedGuard>> {
        debug!("Generating guards");

        let mut generated_guards = Vec::new();

        for pattern in patterns {
            for guard in &pattern.guards {
                let generated = self.generate_single_guard(guard, symbols)?;
                generated_guards.push(generated);
            }
        }

        info!("Generated {} guards", generated_guards.len());
        Ok(generated_guards)
    }

    /// Generate single guard
    fn generate_single_guard(
        &mut self,
        guard: &Guard,
        symbols: &mut SymbolTable,
    ) -> WorkflowResult<GeneratedGuard> {
        // Parse expression into bytecode
        let mut bytecode = Vec::new();
        let mut var_refs = Vec::new();
        let mut const_refs = Vec::new();

        // Simple expression compiler
        let tokens = self.tokenize_expression(&guard.expression)?;
        let ast = self.parse_expression(&tokens)?;
        let (code, vars, consts, stack) = self.compile_expression(&ast, symbols)?;

        bytecode = code;
        var_refs = vars;
        const_refs = consts;

        // Add optimization hints for pure guards
        if guard.guard_type == GuardType::Invariant {
            self.add_hint(HintType::Pure, self.current_offset, &guard.id);
        }

        Ok(GeneratedGuard {
            id: guard.id.clone(),
            bytecode,
            var_refs,
            const_refs,
            stack_depth: stack,
        })
    }

    /// Tokenize expression
    fn tokenize_expression(&self, expr: &str) -> WorkflowResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = expr.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
                    chars.next();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    chars.next();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    chars.next();
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Equal);
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '&' => {
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(Token::And);
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(Token::Or);
                    }
                }
                '+' => {
                    tokens.push(Token::Plus);
                    chars.next();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    chars.next();
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    chars.next();
                }
                '/' => {
                    tokens.push(Token::Divide);
                    chars.next();
                }
                '0'..='9' => {
                    let num = self.read_number(&mut chars);
                    tokens.push(Token::Number(num));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.read_identifier(&mut chars);
                    if ident == "true" {
                        tokens.push(Token::Boolean(true));
                    } else if ident == "false" {
                        tokens.push(Token::Boolean(false));
                    } else {
                        tokens.push(Token::Identifier(ident));
                    }
                }
                '"' => {
                    chars.next();
                    let string = self.read_string(&mut chars)?;
                    tokens.push(Token::String(string));
                }
                _ => {
                    chars.next(); // Skip unknown characters
                }
            }
        }

        Ok(tokens)
    }

    /// Read number from chars
    fn read_number(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> f64 {
        let mut num = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                num.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        num.parse().unwrap_or(0.0)
    }

    /// Read identifier
    fn read_identifier(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut ident = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        ident
    }

    /// Read string
    fn read_string(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> WorkflowResult<String> {
        let mut string = String::new();
        while let Some(ch) = chars.next() {
            if ch == '"' {
                return Ok(string);
            } else if ch == '\\' {
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '"' => string.push('"'),
                        '\\' => string.push('\\'),
                        _ => string.push(escaped),
                    }
                }
            } else {
                string.push(ch);
            }
        }
        Err(WorkflowError::Parse("Unterminated string".to_string()))
    }

    /// Parse expression into AST
    fn parse_expression(&self, tokens: &[Token]) -> WorkflowResult<AstNode> {
        // Simple recursive descent parser
        let mut parser = Parser::new(tokens);
        parser.parse_or()
    }

    /// Compile expression to bytecode
    fn compile_expression(
        &mut self,
        ast: &AstNode,
        symbols: &mut SymbolTable,
    ) -> WorkflowResult<(Vec<u8>, Vec<u16>, Vec<u16>, u8)> {
        let mut compiler = ExpressionCompiler::new(symbols);
        compiler.compile(ast)
    }

    /// Generate receipt templates
    fn generate_receipts(
        &mut self,
        patterns: &[ExtractedPattern],
    ) -> WorkflowResult<Vec<ReceiptTemplate>> {
        debug!("Generating receipt templates");

        let mut receipts = Vec::new();

        for pattern in patterns {
            let receipt = self.generate_receipt_for_pattern(pattern)?;
            receipts.push(receipt);
        }

        info!("Generated {} receipt templates", receipts.len());
        Ok(receipts)
    }

    /// Generate receipt for pattern
    fn generate_receipt_for_pattern(
        &mut self,
        pattern: &ExtractedPattern,
    ) -> WorkflowResult<ReceiptTemplate> {
        let mut fields = Vec::new();
        let mut offset = 0u16;

        // Standard fields
        fields.push(ReceiptField {
            name: "pattern_id".to_string(),
            field_type: FieldType::U8,
            offset,
            size: 1,
        });
        offset += 1;

        fields.push(ReceiptField {
            name: "timestamp".to_string(),
            field_type: FieldType::Timestamp,
            offset,
            size: 8,
        });
        offset += 8;

        fields.push(ReceiptField {
            name: "execution_ticks".to_string(),
            field_type: FieldType::U8,
            offset,
            size: 1,
        });
        offset += 1;

        // Variable fields
        for var in &pattern.variables {
            let (field_type, size) = match var.data_type {
                DataType::String => (FieldType::String, 32),
                DataType::Integer => (FieldType::I64, 8),
                DataType::Float => (FieldType::F64, 8),
                DataType::Boolean => (FieldType::U8, 1),
                DataType::DateTime => (FieldType::Timestamp, 8),
                DataType::Duration => (FieldType::U64, 8),
                DataType::Object(_) => (FieldType::Bytes, 64),
            };

            fields.push(ReceiptField {
                name: var.name.clone(),
                field_type,
                offset,
                size,
            });
            offset += size;
        }

        // Guard results
        if !pattern.guards.is_empty() {
            fields.push(ReceiptField {
                name: "guard_results".to_string(),
                field_type: FieldType::Bytes,
                offset,
                size: (pattern.guards.len() as u16 + 7) / 8, // Bit vector
            });
            offset += (pattern.guards.len() as u16 + 7) / 8;
        }

        let format = if offset <= 64 {
            ReceiptFormat::Fixed
        } else if offset <= 256 {
            ReceiptFormat::Variable
        } else {
            ReceiptFormat::Compressed
        };

        Ok(ReceiptTemplate {
            pattern_id: pattern.pattern_id,
            format,
            fields,
            size: offset as u32,
        })
    }

    /// Collect constants
    fn collect_constants(&self, patterns: &[ExtractedPattern]) -> WorkflowResult<Vec<Constant>> {
        let mut constants = Vec::new();
        let mut const_id = 0u16;

        for pattern in patterns {
            for guard in &pattern.guards {
                // Extract constants from guard expressions
                if guard.expression.contains("true") {
                    constants.push(Constant {
                        id: const_id,
                        value: ConstantValue::Boolean(true),
                    });
                    const_id += 1;
                }
                if guard.expression.contains("false") {
                    constants.push(Constant {
                        id: const_id,
                        value: ConstantValue::Boolean(false),
                    });
                    const_id += 1;
                }
            }

            // Extract from initial values
            for var in &pattern.variables {
                if let Some(ref init) = var.initial_value {
                    let value = match var.data_type {
                        DataType::Integer => {
                            if let Ok(i) = init.parse::<i64>() {
                                Some(ConstantValue::Integer(i))
                            } else {
                                None
                            }
                        }
                        DataType::Float => {
                            if let Ok(f) = init.parse::<f64>() {
                                Some(ConstantValue::Float(f))
                            } else {
                                None
                            }
                        }
                        DataType::Boolean => {
                            if let Ok(b) = init.parse::<bool>() {
                                Some(ConstantValue::Boolean(b))
                            } else {
                                None
                            }
                        }
                        DataType::String => Some(ConstantValue::String(init.clone())),
                        _ => None,
                    };

                    if let Some(val) = value {
                        constants.push(Constant {
                            id: const_id,
                            value: val,
                        });
                        const_id += 1;
                    }
                }
            }
        }

        Ok(constants)
    }

    /// Calculate metadata
    fn calculate_metadata(
        &self,
        dispatch: &DispatchTable,
        guards: &[GeneratedGuard],
        receipts: &[ReceiptTemplate],
    ) -> CodeMetadata {
        let code_size =
            dispatch.entries.len() * 64 + guards.iter().map(|g| g.bytecode.len()).sum::<usize>();

        let data_size = receipts.iter().map(|r| r.size as usize).sum::<usize>();

        let stack_size = guards
            .iter()
            .map(|g| g.stack_depth as usize)
            .max()
            .unwrap_or(0)
            * 8; // 8 bytes per stack slot

        CodeMetadata {
            code_size,
            data_size,
            stack_size,
            optimization_level: 1,
        }
    }

    /// Add optimization hint
    fn add_hint(&self, hint_type: HintType, location: u32, data: &str) {
        // Hints are collected but not stored in this version
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            next_id: 0,
        }
    }

    fn add_variable(&mut self, name: String, data_type: DataType) -> u16 {
        let id = self.next_id;
        self.next_id += 1;

        self.variables.insert(
            name.clone(),
            Symbol {
                id,
                name: name.clone(),
                symbol_type: SymbolType::Variable(data_type),
                offset: id as u32 * 8, // Simple offset calculation
            },
        );

        id
    }
}

// Token types for expression parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Not,
    Assign,
    LeftParen,
    RightParen,
}

// AST node for expressions
#[derive(Debug, Clone)]
enum AstNode {
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<AstNode>,
    },
}

#[derive(Debug, Clone)]
enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
enum UnaryOperator {
    Not,
    Negate,
}

// Simple parser
struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_or(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_and()?;

        while self.current() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = AstNode::BinaryOp {
                op: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_equality()?;

        while self.current() == Some(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = AstNode::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_comparison()?;

        while let Some(token) = self.current() {
            let op = match token {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_term()?;

        while let Some(token) = self.current() {
            let op = match token {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.current() {
            let op = match token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> WorkflowResult<AstNode> {
        let mut left = self.parse_unary()?;

        while let Some(token) = self.current() {
            let op = match token {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> WorkflowResult<AstNode> {
        match self.current() {
            Some(Token::Not) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> WorkflowResult<AstNode> {
        match self.current() {
            Some(Token::Number(n)) => {
                let num = *n;
                self.advance();
                Ok(AstNode::Number(num))
            }
            Some(Token::String(s)) => {
                let string = s.clone();
                self.advance();
                Ok(AstNode::String(string))
            }
            Some(Token::Boolean(b)) => {
                let bool_val = *b;
                self.advance();
                Ok(AstNode::Boolean(bool_val))
            }
            Some(Token::Identifier(name)) => {
                let var_name = name.clone();
                self.advance();
                Ok(AstNode::Variable(var_name))
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_or()?;
                if self.current() != Some(&Token::RightParen) {
                    return Err(WorkflowError::Parse("Expected )".to_string()));
                }
                self.advance();
                Ok(expr)
            }
            _ => Err(WorkflowError::Parse("Unexpected token".to_string())),
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}

// Expression compiler
struct ExpressionCompiler<'a> {
    symbols: &'a mut SymbolTable,
    bytecode: Vec<u8>,
    var_refs: Vec<u16>,
    const_refs: Vec<u16>,
    max_stack: u8,
    current_stack: u8,
}

impl<'a> ExpressionCompiler<'a> {
    fn new(symbols: &'a mut SymbolTable) -> Self {
        Self {
            symbols,
            bytecode: Vec::new(),
            var_refs: Vec::new(),
            const_refs: Vec::new(),
            max_stack: 0,
            current_stack: 0,
        }
    }

    fn compile(&mut self, ast: &AstNode) -> WorkflowResult<(Vec<u8>, Vec<u16>, Vec<u16>, u8)> {
        self.compile_node(ast)?;
        self.emit_return();

        Ok((
            self.bytecode.clone(),
            self.var_refs.clone(),
            self.const_refs.clone(),
            self.max_stack,
        ))
    }

    fn compile_node(&mut self, node: &AstNode) -> WorkflowResult<()> {
        match node {
            AstNode::Number(n) => {
                self.emit_const_float(*n);
            }
            AstNode::Boolean(b) => {
                self.emit_const_bool(*b);
            }
            AstNode::String(s) => {
                // String constants would need special handling
                self.emit_const_string(s);
            }
            AstNode::Variable(name) => {
                let var_id = self.symbols.add_variable(name.clone(), DataType::String);
                self.var_refs.push(var_id);
                self.emit_load_var(var_id);
            }
            AstNode::BinaryOp { op, left, right } => {
                self.compile_node(left)?;
                self.compile_node(right)?;
                self.emit_binary_op(op);
            }
            AstNode::UnaryOp { op, operand } => {
                self.compile_node(operand)?;
                self.emit_unary_op(op);
            }
        }
        Ok(())
    }

    fn emit_const_float(&mut self, value: f64) {
        self.bytecode.push(0x01); // CONST_FLOAT opcode
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self.push_stack(1);
    }

    fn emit_const_bool(&mut self, value: bool) {
        self.bytecode.push(0x02); // CONST_BOOL opcode
        self.bytecode.push(if value { 1 } else { 0 });
        self.push_stack(1);
    }

    fn emit_const_string(&mut self, _value: &str) {
        // String constants would need special handling
        self.bytecode.push(0x03); // CONST_STRING opcode
        self.push_stack(1);
    }

    fn emit_load_var(&mut self, var_id: u16) {
        self.bytecode.push(0x10); // LOAD_VAR opcode
        self.bytecode.extend_from_slice(&var_id.to_le_bytes());
        self.push_stack(1);
    }

    fn emit_binary_op(&mut self, op: &BinaryOperator) {
        let opcode = match op {
            BinaryOperator::Add => 0x20,
            BinaryOperator::Subtract => 0x21,
            BinaryOperator::Multiply => 0x22,
            BinaryOperator::Divide => 0x23,
            BinaryOperator::Greater => 0x24,
            BinaryOperator::GreaterEqual => 0x25,
            BinaryOperator::Less => 0x26,
            BinaryOperator::LessEqual => 0x27,
            BinaryOperator::Equal => 0x28,
            BinaryOperator::NotEqual => 0x29,
            BinaryOperator::And => 0x2A,
            BinaryOperator::Or => 0x2B,
        };
        self.bytecode.push(opcode);
        self.pop_stack(1); // Two operands consumed, one result produced
    }

    fn emit_unary_op(&mut self, op: &UnaryOperator) {
        let opcode = match op {
            UnaryOperator::Not => 0x30,
            UnaryOperator::Negate => 0x31,
        };
        self.bytecode.push(opcode);
        // Stack depth unchanged (one consumed, one produced)
    }

    fn emit_return(&mut self) {
        self.bytecode.push(0xFF); // RETURN opcode
    }

    fn push_stack(&mut self, count: u8) {
        self.current_stack += count;
        if self.current_stack > self.max_stack {
            self.max_stack = self.current_stack;
        }
    }

    fn pop_stack(&mut self, count: u8) {
        self.current_stack = self.current_stack.saturating_sub(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generator_creation() {
        let generator = CodeGenerator::new();
        assert_eq!(generator.current_offset, 0);
    }

    #[tokio::test]
    async fn test_symbol_table() {
        let mut symbols = SymbolTable::new();
        let id = symbols.add_variable("test".to_string(), DataType::Integer);
        assert_eq!(id, 0);
        assert!(symbols.variables.contains_key("test"));
    }

    #[tokio::test]
    async fn test_tokenization() {
        let generator = CodeGenerator::new();
        let tokens = generator.tokenize_expression("x > 5 && y < 10").unwrap();
        assert!(!tokens.is_empty());
    }

    #[tokio::test]
    async fn test_simple_expression_parsing() {
        let generator = CodeGenerator::new();
        let tokens = generator.tokenize_expression("true").unwrap();
        let ast = generator.parse_expression(&tokens).unwrap();
        match ast {
            AstNode::Boolean(true) => {}
            _ => panic!("Expected boolean true"),
        }
    }
}
