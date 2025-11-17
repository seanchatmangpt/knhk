//! Binary Serializer
//!
//! Serializes linked descriptors into binary format for kernel loading.
//! Uses a custom, compact binary format with zero-copy deserialization support.

use crate::compiler::linker::{LinkedDescriptor, LinkedSymbolType, RelocationType};
use crate::error::{WorkflowError, WorkflowResult};
use std::io::Write;
use tracing::{debug, info, instrument};

/// Descriptor binary format version
const FORMAT_VERSION: u32 = 0x01000000; // 1.0.0.0

/// Magic number for descriptor files
const MAGIC_NUMBER: [u8; 4] = [0x4B, 0x4E, 0x48, 0x4B]; // "KNHK"

/// Binary serializer
pub struct BinarySerializer {
    /// Enable compression
    compression: bool,
}

impl BinarySerializer {
    /// Create new serializer
    pub fn new() -> Self {
        Self {
            compression: false, // Disabled for now, can use lz4 later
        }
    }

    /// Serialize descriptor to binary format
    #[instrument(skip(self, descriptor))]
    pub async fn serialize(&self, descriptor: &LinkedDescriptor) -> WorkflowResult<Vec<u8>> {
        info!("Serializing descriptor to binary format");

        let mut buffer =
            Vec::with_capacity(descriptor.code_segment.size + descriptor.data_segment.size + 4096);

        // Write header
        self.write_header(&mut buffer, descriptor)?;

        // Write metadata
        self.write_metadata(&mut buffer, descriptor)?;

        // Write code segment
        self.write_code_segment(&mut buffer, descriptor)?;

        // Write data segment
        self.write_data_segment(&mut buffer, descriptor)?;

        // Write symbol table
        self.write_symbol_table(&mut buffer, descriptor)?;

        // Write relocation table
        self.write_relocation_table(&mut buffer, descriptor)?;

        // Write entry points
        self.write_entry_points(&mut buffer, descriptor)?;

        // Optionally compress
        if self.compression {
            buffer = self.compress_buffer(buffer)?;
        }

        info!("Serialized descriptor: {} bytes", buffer.len());
        Ok(buffer)
    }

    /// Write descriptor header
    fn write_header(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        // Magic number (4 bytes)
        buffer.extend_from_slice(&MAGIC_NUMBER);

        // Format version (4 bytes)
        buffer.extend_from_slice(&FORMAT_VERSION.to_le_bytes());

        // Header size (4 bytes)
        buffer.extend_from_slice(&64u32.to_le_bytes());

        // Pattern count (4 bytes)
        buffer.extend_from_slice(&(descriptor.pattern_count as u32).to_le_bytes());

        // Code segment offset (8 bytes)
        buffer.extend_from_slice(&256u64.to_le_bytes()); // After headers

        // Code segment size (8 bytes)
        buffer.extend_from_slice(&(descriptor.code_segment.size as u64).to_le_bytes());

        // Data segment offset (8 bytes)
        let data_offset = 256u64 + descriptor.code_segment.size as u64;
        buffer.extend_from_slice(&data_offset.to_le_bytes());

        // Data segment size (8 bytes)
        buffer.extend_from_slice(&(descriptor.data_segment.size as u64).to_le_bytes());

        // Symbol table offset (8 bytes)
        let symbol_offset = data_offset + descriptor.data_segment.size as u64;
        buffer.extend_from_slice(&symbol_offset.to_le_bytes());

        // Symbol count (4 bytes)
        buffer.extend_from_slice(&(descriptor.symbol_table.symbols.len() as u32).to_le_bytes());

        // Flags (4 bytes)
        let mut flags = 0u32;
        if self.compression {
            flags |= 1 << 0;
        }
        buffer.extend_from_slice(&flags.to_le_bytes());

        Ok(())
    }

    /// Write metadata section
    fn write_metadata(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        // Timestamp (8 bytes)
        buffer.extend_from_slice(&descriptor.metadata.timestamp.to_le_bytes());

        // Checksum (4 bytes)
        buffer.extend_from_slice(&descriptor.metadata.checksum.to_le_bytes());

        // Linker version string length (4 bytes)
        let version_bytes = descriptor.metadata.linker_version.as_bytes();
        buffer.extend_from_slice(&(version_bytes.len() as u32).to_le_bytes());

        // Linker version string
        buffer.extend_from_slice(version_bytes);

        // Pad to align
        while buffer.len() % 16 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    /// Write code segment
    fn write_code_segment(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        debug!(
            "Writing code segment: {} bytes",
            descriptor.code_segment.size
        );

        // Ensure we're at the expected offset (256)
        while buffer.len() < 256 {
            buffer.push(0);
        }

        // Write raw bytecode
        buffer.extend_from_slice(&descriptor.code_segment.bytecode);

        // Align to cache line
        while buffer.len() % descriptor.code_segment.alignment != 0 {
            buffer.push(0x00); // NOP padding
        }

        Ok(())
    }

    /// Write data segment
    fn write_data_segment(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        debug!(
            "Writing data segment: {} bytes",
            descriptor.data_segment.size
        );

        // Data section
        buffer.extend_from_slice(&descriptor.data_segment.data);

        // Constants section
        buffer.extend_from_slice(&descriptor.data_segment.constants);

        // String pool
        buffer.extend_from_slice(&descriptor.data_segment.strings);

        // Align to 8 bytes
        while buffer.len() % 8 != 0 {
            buffer.push(0);
        }

        Ok(())
    }

    /// Write symbol table
    fn write_symbol_table(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        debug!(
            "Writing symbol table: {} symbols",
            descriptor.symbol_table.symbols.len()
        );

        // Symbol count (redundant but useful for verification)
        buffer.extend_from_slice(&(descriptor.symbol_table.symbols.len() as u32).to_le_bytes());

        for symbol in &descriptor.symbol_table.symbols {
            // Symbol type (1 byte)
            let type_byte = match symbol.symbol_type {
                LinkedSymbolType::Pattern => 0u8,
                LinkedSymbolType::Guard => 1u8,
                LinkedSymbolType::Variable => 2u8,
                LinkedSymbolType::Constant => 3u8,
                LinkedSymbolType::String => 4u8,
                LinkedSymbolType::Receipt => 5u8,
            };
            buffer.push(type_byte);

            // Flags (4 bytes)
            buffer.extend_from_slice(&symbol.flags.to_le_bytes());

            // Address (4 bytes)
            buffer.extend_from_slice(&symbol.address.to_le_bytes());

            // Size (4 bytes)
            buffer.extend_from_slice(&symbol.size.to_le_bytes());

            // Name length (2 bytes)
            let name_bytes = symbol.name.as_bytes();
            buffer.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());

            // Name
            buffer.extend_from_slice(name_bytes);

            // Align to 4 bytes
            while buffer.len() % 4 != 0 {
                buffer.push(0);
            }
        }

        Ok(())
    }

    /// Write relocation table
    fn write_relocation_table(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        debug!(
            "Writing relocation table: {} entries",
            descriptor.relocations.len()
        );

        // Relocation count
        buffer.extend_from_slice(&(descriptor.relocations.len() as u32).to_le_bytes());

        for reloc in &descriptor.relocations {
            // Offset (4 bytes)
            buffer.extend_from_slice(&reloc.offset.to_le_bytes());

            // Symbol index (2 bytes)
            buffer.extend_from_slice(&(reloc.symbol as u16).to_le_bytes());

            // Relocation type (1 byte)
            let type_byte = match reloc.reloc_type {
                RelocationType::Absolute => 0u8,
                RelocationType::Relative => 1u8,
                RelocationType::PatternRef => 2u8,
                RelocationType::GuardRef => 3u8,
                RelocationType::VariableRef => 4u8,
            };
            buffer.push(type_byte);

            // Padding (1 byte)
            buffer.push(0);

            // Addend (4 bytes)
            buffer.extend_from_slice(&reloc.addend.to_le_bytes());
        }

        Ok(())
    }

    /// Write entry points
    fn write_entry_points(
        &self,
        buffer: &mut Vec<u8>,
        descriptor: &LinkedDescriptor,
    ) -> WorkflowResult<()> {
        debug!(
            "Writing entry points: {} entries",
            descriptor.entry_points.len()
        );

        // Entry point count
        buffer.extend_from_slice(&(descriptor.entry_points.len() as u32).to_le_bytes());

        // Write sorted entry points for deterministic output
        let mut sorted_entries: Vec<_> = descriptor.entry_points.iter().collect();
        sorted_entries.sort_by_key(|(pattern_id, _)| *pattern_id);

        for (pattern_id, address) in sorted_entries {
            // Pattern ID (1 byte)
            buffer.push(*pattern_id);

            // Padding (3 bytes)
            buffer.extend_from_slice(&[0, 0, 0]);

            // Address (4 bytes)
            buffer.extend_from_slice(&address.to_le_bytes());
        }

        Ok(())
    }

    /// Compress buffer (placeholder for future implementation)
    fn compress_buffer(&self, buffer: Vec<u8>) -> WorkflowResult<Vec<u8>> {
        // Future: Use lz4 or zstd compression
        Ok(buffer)
    }

    /// Deserialize descriptor from binary (for verification)
    pub fn deserialize(&self, data: &[u8]) -> WorkflowResult<DescriptorInfo> {
        if data.len() < 64 {
            return Err(WorkflowError::Parse("Descriptor too small".to_string()));
        }

        // Verify magic number
        if &data[0..4] != MAGIC_NUMBER {
            return Err(WorkflowError::Parse("Invalid magic number".to_string()));
        }

        // Read version
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version != FORMAT_VERSION {
            return Err(WorkflowError::Parse(format!(
                "Unsupported format version: {:08x}",
                version
            )));
        }

        // Read header fields
        let header_size = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let pattern_count = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let code_offset = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let code_size = u64::from_le_bytes(data[24..32].try_into().unwrap());
        let data_offset = u64::from_le_bytes(data[32..40].try_into().unwrap());
        let data_size = u64::from_le_bytes(data[40..48].try_into().unwrap());
        let symbol_offset = u64::from_le_bytes(data[48..56].try_into().unwrap());
        let symbol_count = u32::from_le_bytes(data[56..60].try_into().unwrap());
        let flags = u32::from_le_bytes(data[60..64].try_into().unwrap());

        Ok(DescriptorInfo {
            version,
            header_size,
            pattern_count,
            code_offset,
            code_size,
            data_offset,
            data_size,
            symbol_offset,
            symbol_count,
            flags,
        })
    }
}

/// Descriptor information (for deserialization)
#[derive(Debug)]
pub struct DescriptorInfo {
    pub version: u32,
    pub header_size: u32,
    pub pattern_count: u32,
    pub code_offset: u64,
    pub code_size: u64,
    pub data_offset: u64,
    pub data_size: u64,
    pub symbol_offset: u64,
    pub symbol_count: u32,
    pub flags: u32,
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::linker::{
        CodeSegment, DataSegment, LinkMetadata, LinkedDescriptor, LinkedSymbolTable,
    };
    use std::collections::HashMap;

    fn create_test_descriptor() -> LinkedDescriptor {
        LinkedDescriptor {
            pattern_count: 1,
            code_segment: CodeSegment {
                bytecode: vec![0x01, 0x02, 0x03, 0x04],
                size: 4,
                alignment: 64,
            },
            data_segment: DataSegment {
                data: vec![0x10, 0x11],
                constants: vec![0x20],
                strings: vec![0x30],
                size: 4,
            },
            symbol_table: LinkedSymbolTable {
                symbols: Vec::new(),
                name_map: HashMap::new(),
            },
            relocations: Vec::new(),
            entry_points: HashMap::from([(1, 0x100)]),
            metadata: LinkMetadata {
                timestamp: 1234567890,
                linker_version: "1.0.0".to_string(),
                total_size: 8,
                checksum: 0xDEADBEEF,
            },
        }
    }

    #[tokio::test]
    async fn test_serializer_creation() {
        let serializer = BinarySerializer::new();
        assert!(!serializer.compression);
    }

    #[tokio::test]
    async fn test_serialization() {
        let serializer = BinarySerializer::new();
        let descriptor = create_test_descriptor();

        let binary = serializer.serialize(&descriptor).await.unwrap();
        assert!(binary.len() > 64); // At least header size

        // Verify magic number
        assert_eq!(&binary[0..4], &MAGIC_NUMBER);
    }

    #[tokio::test]
    async fn test_deserialization() {
        let serializer = BinarySerializer::new();
        let descriptor = create_test_descriptor();

        let binary = serializer.serialize(&descriptor).await.unwrap();
        let info = serializer.deserialize(&binary).unwrap();

        assert_eq!(info.version, FORMAT_VERSION);
        assert_eq!(info.pattern_count, 1);
    }

    #[tokio::test]
    async fn test_round_trip() {
        let serializer = BinarySerializer::new();
        let descriptor = create_test_descriptor();

        // Serialize
        let binary = serializer.serialize(&descriptor).await.unwrap();

        // Deserialize and verify
        let info = serializer.deserialize(&binary).unwrap();
        assert_eq!(info.pattern_count, descriptor.pattern_count as u32);
        assert_eq!(info.code_size, descriptor.code_segment.size as u64);
        assert_eq!(info.data_size, descriptor.data_segment.size as u64);
    }
}
