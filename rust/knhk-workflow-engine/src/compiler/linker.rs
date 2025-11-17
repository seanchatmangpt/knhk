//! Pattern Linker
//!
//! Links compiled patterns, resolves references, and builds the final descriptor.

use crate::compiler::code_generator::GeneratedCode;
use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// Linked descriptor
#[derive(Debug, Clone)]
pub struct LinkedDescriptor {
    /// Pattern count
    pub pattern_count: usize,
    /// Code segment
    pub code_segment: CodeSegment,
    /// Data segment
    pub data_segment: DataSegment,
    /// Symbol table
    pub symbol_table: LinkedSymbolTable,
    /// Relocation table
    pub relocations: Vec<Relocation>,
    /// Entry points
    pub entry_points: HashMap<u8, u32>,
    /// Metadata
    pub metadata: LinkMetadata,
}

/// Code segment
#[derive(Debug, Clone)]
pub struct CodeSegment {
    /// Raw bytecode
    pub bytecode: Vec<u8>,
    /// Segment size
    pub size: usize,
    /// Alignment
    pub alignment: usize,
}

/// Data segment
#[derive(Debug, Clone)]
pub struct DataSegment {
    /// Static data
    pub data: Vec<u8>,
    /// Constants
    pub constants: Vec<u8>,
    /// String pool
    pub strings: Vec<u8>,
    /// Size
    pub size: usize,
}

/// Linked symbol table
#[derive(Debug, Clone)]
pub struct LinkedSymbolTable {
    /// Symbol entries
    pub symbols: Vec<LinkedSymbol>,
    /// Name to index map
    pub name_map: HashMap<String, usize>,
}

/// Linked symbol
#[derive(Debug, Clone)]
pub struct LinkedSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub symbol_type: LinkedSymbolType,
    /// Address/offset
    pub address: u32,
    /// Size
    pub size: u32,
    /// Flags
    pub flags: u32,
}

/// Symbol type after linking
#[derive(Debug, Clone)]
pub enum LinkedSymbolType {
    Pattern,
    Guard,
    Variable,
    Constant,
    String,
    Receipt,
}

/// Relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    /// Offset in code segment
    pub offset: u32,
    /// Symbol index
    pub symbol: usize,
    /// Relocation type
    pub reloc_type: RelocationType,
    /// Addend
    pub addend: i32,
}

/// Relocation type
#[derive(Debug, Clone)]
pub enum RelocationType {
    Absolute,
    Relative,
    PatternRef,
    GuardRef,
    VariableRef,
}

/// Link metadata
#[derive(Debug, Clone)]
pub struct LinkMetadata {
    /// Link timestamp
    pub timestamp: u64,
    /// Linker version
    pub linker_version: String,
    /// Total size
    pub total_size: usize,
    /// Checksum
    pub checksum: u32,
}

/// Pattern linker
pub struct Linker {
    /// Current code offset
    code_offset: u32,
    /// Current data offset
    data_offset: u32,
    /// Symbol index
    symbol_index: usize,
    /// Address map
    address_map: HashMap<String, u32>,
}

impl Linker {
    /// Create new linker
    pub fn new() -> Self {
        Self {
            code_offset: 0,
            data_offset: 0,
            symbol_index: 0,
            address_map: HashMap::new(),
        }
    }

    /// Link generated code into descriptor
    #[instrument(skip(self, code))]
    pub async fn link(&mut self, code: GeneratedCode) -> WorkflowResult<LinkedDescriptor> {
        info!("Linking patterns into descriptor");

        // Phase 1: Layout segments
        let (code_segment, data_segment) = self.layout_segments(&code)?;

        // Phase 2: Build symbol table
        let symbol_table = self.build_symbol_table(&code)?;

        // Phase 3: Resolve references
        let relocations = self.resolve_references(&code)?;

        // Phase 4: Build entry points
        let entry_points = self.build_entry_points(&code)?;

        // Phase 5: Calculate checksums
        let metadata = self.calculate_metadata(&code_segment, &data_segment)?;

        let pattern_count = code.dispatch_table.entries.len();

        info!(
            "Linked {} patterns, {} symbols, {} relocations",
            pattern_count,
            symbol_table.symbols.len(),
            relocations.len()
        );

        Ok(LinkedDescriptor {
            pattern_count,
            code_segment,
            data_segment,
            symbol_table,
            relocations,
            entry_points,
            metadata,
        })
    }

    /// Layout memory segments
    fn layout_segments(
        &mut self,
        code: &GeneratedCode,
    ) -> WorkflowResult<(CodeSegment, DataSegment)> {
        debug!("Laying out memory segments");

        // Layout code segment
        let mut bytecode = Vec::new();

        // Add dispatch table
        for entry in &code.dispatch_table.entries {
            self.emit_dispatch_entry(&mut bytecode, entry)?;
        }

        // Add guard code
        for guard in &code.guards {
            let guard_offset = self.code_offset;
            self.address_map.insert(guard.id.clone(), guard_offset);

            // Align to 16 bytes for better performance
            while bytecode.len() % 16 != 0 {
                bytecode.push(0x00); // NOP padding
            }

            bytecode.extend_from_slice(&guard.bytecode);
            self.code_offset = bytecode.len() as u32;
        }

        let code_segment = CodeSegment {
            size: bytecode.len(),
            alignment: 64, // Cache line aligned
            bytecode,
        };

        // Layout data segment
        let mut data = Vec::new();
        let mut constants = Vec::new();
        let mut strings = Vec::new();

        // Add constants
        for constant in &code.constants {
            let const_offset = self.data_offset;
            self.address_map
                .insert(format!("const_{}", constant.id), const_offset);

            match &constant.value {
                crate::compiler::code_generator::ConstantValue::Integer(i) => {
                    constants.extend_from_slice(&i.to_le_bytes());
                }
                crate::compiler::code_generator::ConstantValue::Float(f) => {
                    constants.extend_from_slice(&f.to_le_bytes());
                }
                crate::compiler::code_generator::ConstantValue::String(s) => {
                    let str_offset = strings.len();
                    strings.extend_from_slice(s.as_bytes());
                    strings.push(0); // Null terminator
                    constants.extend_from_slice(&(str_offset as u32).to_le_bytes());
                }
                crate::compiler::code_generator::ConstantValue::Boolean(b) => {
                    constants.push(if *b { 1 } else { 0 });
                }
            }

            self.data_offset = constants.len() as u32;
        }

        // Add receipt templates
        for receipt in &code.receipts {
            let receipt_offset = data.len() as u32;
            self.address_map
                .insert(format!("receipt_{}", receipt.pattern_id), receipt_offset);

            // Serialize receipt template
            data.push(receipt.pattern_id);
            data.push(receipt.fields.len() as u8);
            data.extend_from_slice(&(receipt.size).to_le_bytes());

            for field in &receipt.fields {
                // Field name index (would be in string pool)
                let name_offset = strings.len();
                strings.extend_from_slice(field.name.as_bytes());
                strings.push(0);

                data.extend_from_slice(&(name_offset as u16).to_le_bytes());
                data.extend_from_slice(&field.offset.to_le_bytes());
                data.extend_from_slice(&field.size.to_le_bytes());
            }
        }

        let data_segment = DataSegment {
            size: data.len() + constants.len() + strings.len(),
            data,
            constants,
            strings,
        };

        Ok((code_segment, data_segment))
    }

    /// Emit dispatch table entry
    fn emit_dispatch_entry(
        &mut self,
        bytecode: &mut Vec<u8>,
        entry: &crate::compiler::code_generator::DispatchEntry,
    ) -> WorkflowResult<()> {
        // Pattern ID (1 byte)
        bytecode.push(entry.pattern_id);

        // Flags (4 bytes)
        bytecode.extend_from_slice(&entry.flags.to_le_bytes());

        // Entry point (4 bytes)
        bytecode.extend_from_slice(&entry.entry_point.to_le_bytes());

        // Guard offset (4 bytes)
        bytecode.extend_from_slice(&entry.guard_offset.to_le_bytes());

        // Variable table offset (4 bytes)
        bytecode.extend_from_slice(&entry.var_table_offset.to_le_bytes());

        // Receipt offset (4 bytes)
        bytecode.extend_from_slice(&entry.receipt_offset.to_le_bytes());

        // Padding to 64 bytes (cache line)
        while bytecode.len() % 64 != 0 {
            bytecode.push(0);
        }

        Ok(())
    }

    /// Build symbol table
    fn build_symbol_table(&mut self, code: &GeneratedCode) -> WorkflowResult<LinkedSymbolTable> {
        debug!("Building symbol table");

        let mut symbols = Vec::new();
        let mut name_map = HashMap::new();

        // Add pattern symbols
        for entry in &code.dispatch_table.entries {
            let name = format!("pattern_{}", entry.pattern_id);
            let symbol = LinkedSymbol {
                name: name.clone(),
                symbol_type: LinkedSymbolType::Pattern,
                address: entry.entry_point,
                size: 64, // Dispatch entry size
                flags: entry.flags,
            };

            name_map.insert(name, symbols.len());
            symbols.push(symbol);
        }

        // Add guard symbols
        for guard in &code.guards {
            let symbol = LinkedSymbol {
                name: guard.id.clone(),
                symbol_type: LinkedSymbolType::Guard,
                address: *self.address_map.get(&guard.id).unwrap_or(&0),
                size: guard.bytecode.len() as u32,
                flags: 0,
            };

            name_map.insert(guard.id.clone(), symbols.len());
            symbols.push(symbol);
        }

        // Add variable symbols
        for (name, symbol) in &code.symbols.variables {
            let linked_symbol = LinkedSymbol {
                name: name.clone(),
                symbol_type: LinkedSymbolType::Variable,
                address: symbol.offset,
                size: 8, // Variable size
                flags: 0,
            };

            name_map.insert(name.clone(), symbols.len());
            symbols.push(linked_symbol);
        }

        // Add constant symbols
        for constant in &code.constants {
            let name = format!("const_{}", constant.id);
            let symbol = LinkedSymbol {
                name: name.clone(),
                symbol_type: LinkedSymbolType::Constant,
                address: *self.address_map.get(&name).unwrap_or(&0),
                size: match constant.value {
                    crate::compiler::code_generator::ConstantValue::Integer(_) => 8,
                    crate::compiler::code_generator::ConstantValue::Float(_) => 8,
                    crate::compiler::code_generator::ConstantValue::String(ref s) => s.len() as u32,
                    crate::compiler::code_generator::ConstantValue::Boolean(_) => 1,
                },
                flags: 0,
            };

            name_map.insert(name, symbols.len());
            symbols.push(symbol);
        }

        Ok(LinkedSymbolTable { symbols, name_map })
    }

    /// Resolve references
    fn resolve_references(&mut self, code: &GeneratedCode) -> WorkflowResult<Vec<Relocation>> {
        debug!("Resolving references");

        let mut relocations = Vec::new();

        // Resolve pattern references in dispatch table
        for entry in &code.dispatch_table.entries {
            if entry.guard_offset > 0 {
                relocations.push(Relocation {
                    offset: entry.entry_point + 12, // Offset to guard_offset field
                    symbol: entry.pattern_id as usize,
                    reloc_type: RelocationType::GuardRef,
                    addend: 0,
                });
            }

            if entry.var_table_offset > 0 {
                relocations.push(Relocation {
                    offset: entry.entry_point + 16, // Offset to var_table_offset field
                    symbol: entry.pattern_id as usize,
                    reloc_type: RelocationType::VariableRef,
                    addend: 0,
                });
            }
        }

        // Resolve variable references in guards
        for (guard_idx, guard) in code.guards.iter().enumerate() {
            for var_ref in &guard.var_refs {
                // Find variable offset in bytecode
                // This is simplified - real implementation would parse bytecode
                relocations.push(Relocation {
                    offset: guard_idx as u32 * 100, // Placeholder
                    symbol: *var_ref as usize,
                    reloc_type: RelocationType::VariableRef,
                    addend: 0,
                });
            }
        }

        info!("Resolved {} references", relocations.len());
        Ok(relocations)
    }

    /// Build entry points
    fn build_entry_points(&self, code: &GeneratedCode) -> WorkflowResult<HashMap<u8, u32>> {
        let mut entry_points = HashMap::new();

        for entry in &code.dispatch_table.entries {
            entry_points.insert(entry.pattern_id, entry.entry_point);
        }

        Ok(entry_points)
    }

    /// Calculate metadata
    fn calculate_metadata(
        &self,
        code_segment: &CodeSegment,
        data_segment: &DataSegment,
    ) -> WorkflowResult<LinkMetadata> {
        let total_size = code_segment.size + data_segment.size;

        // Calculate CRC32 checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&code_segment.bytecode);
        hasher.update(&data_segment.data);
        hasher.update(&data_segment.constants);
        hasher.update(&data_segment.strings);
        let checksum = hasher.finalize();

        Ok(LinkMetadata {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            linker_version: env!("CARGO_PKG_VERSION").to_string(),
            total_size,
            checksum,
        })
    }

    /// Apply relocations
    pub fn apply_relocations(
        &self,
        bytecode: &mut [u8],
        relocations: &[Relocation],
        symbol_table: &LinkedSymbolTable,
    ) -> WorkflowResult<()> {
        for reloc in relocations {
            if reloc.symbol >= symbol_table.symbols.len() {
                return Err(WorkflowError::Internal(format!(
                    "Invalid symbol index: {}",
                    reloc.symbol
                )));
            }

            let symbol = &symbol_table.symbols[reloc.symbol];
            let target_address = symbol.address as i32 + reloc.addend;

            match reloc.reloc_type {
                RelocationType::Absolute => {
                    // Write absolute address
                    if reloc.offset as usize + 4 <= bytecode.len() {
                        bytecode[reloc.offset as usize..reloc.offset as usize + 4]
                            .copy_from_slice(&target_address.to_le_bytes());
                    }
                }
                RelocationType::Relative => {
                    // Calculate relative offset
                    let rel_offset = target_address - reloc.offset as i32;
                    if reloc.offset as usize + 4 <= bytecode.len() {
                        bytecode[reloc.offset as usize..reloc.offset as usize + 4]
                            .copy_from_slice(&rel_offset.to_le_bytes());
                    }
                }
                _ => {
                    // Other relocation types
                }
            }
        }

        Ok(())
    }
}

impl Default for Linker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_linker_creation() {
        let linker = Linker::new();
        assert_eq!(linker.code_offset, 0);
        assert_eq!(linker.data_offset, 0);
    }

    #[tokio::test]
    async fn test_symbol_creation() {
        let symbol = LinkedSymbol {
            name: "test".to_string(),
            symbol_type: LinkedSymbolType::Pattern,
            address: 0x100,
            size: 64,
            flags: 0,
        };

        assert_eq!(symbol.name, "test");
        assert_eq!(symbol.address, 0x100);
    }

    #[tokio::test]
    async fn test_relocation() {
        let reloc = Relocation {
            offset: 0x200,
            symbol: 0,
            reloc_type: RelocationType::Absolute,
            addend: 0,
        };

        assert_eq!(reloc.offset, 0x200);
    }
}
