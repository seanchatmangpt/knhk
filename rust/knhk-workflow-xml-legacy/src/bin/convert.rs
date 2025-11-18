//! CLI tool for converting XML YAWL workflows to TTL format

use clap::{Parser, Subcommand};
use knhk_workflow_xml_legacy::XmlToTtlConverter;
use std::path::PathBuf;

type LegacyResult<T> = Result<T, knhk_workflow_xml_legacy::LegacyError>;

#[derive(Parser)]
#[command(name = "yawl-xml-to-ttl")]
#[command(about = "Convert XML YAWL workflows to TTL/Turtle format", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input XML YAWL file (if no subcommand)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output TTL file (stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Validate TTL output
    #[arg(short, long)]
    validate: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a directory of XML files
    Dir {
        /// Input directory containing XML files
        #[arg(long)]
        dir: PathBuf,

        /// Output directory for TTL files
        #[arg(long)]
        output: PathBuf,

        /// Recursive directory traversal
        #[arg(short, long)]
        recursive: bool,
    },

    /// Validate a TTL file
    Validate {
        /// TTL file to validate
        file: PathBuf,
    },
}

fn main() -> LegacyResult<()> {
    let cli = Cli::parse();
    let converter = XmlToTtlConverter::new();

    match cli.command {
        Some(Commands::Dir {
            dir,
            output,
            recursive,
        }) => {
            convert_directory(&converter, &dir, &output, recursive)?;
        }
        Some(Commands::Validate { file }) => {
            validate_file(&converter, &file)?;
        }
        None => {
            // Simple file conversion
            if let Some(input) = cli.input {
                convert_single_file(&converter, &input, cli.output.as_ref(), cli.validate)?;
            } else {
                eprintln!("Error: No input file specified");
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn convert_single_file(
    converter: &XmlToTtlConverter,
    input: &PathBuf,
    output: Option<&PathBuf>,
    validate: bool,
) -> LegacyResult<()> {
    eprintln!("Converting: {}", input.display());

    let xml = std::fs::read_to_string(input)
        .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to read input: {}", e)))?;

    let ttl = converter.convert(&xml)?;

    if validate {
        eprintln!("Validating TTL output...");
        converter.validate_ttl(&ttl)?;
        eprintln!("✓ Validation passed");
    }

    if let Some(output_path) = output {
        std::fs::write(output_path, &ttl)
            .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to write output: {}", e)))?;
        eprintln!("✓ Written to: {}", output_path.display());
    } else {
        println!("{}", ttl);
    }

    Ok(())
}

fn convert_directory(
    converter: &XmlToTtlConverter,
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    recursive: bool,
) -> LegacyResult<()> {
    // Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to create output dir: {}", e)))?;

    let entries: Vec<std::path::PathBuf> = if recursive {
        walkdir::WalkDir::new(input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        std::fs::read_dir(input_dir)
            .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to read dir: {}", e)))?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .map(|e| e.path())
            .collect()
    };

    let mut converted = 0;
    let mut errors = 0;

    for path in entries {
        let path = &path;

        // Only process XML/YAWL files
        if let Some(ext) = path.extension() {
            if ext != "xml" && ext != "yawl" {
                continue;
            }
        } else {
            continue;
        }

        let relative = path.strip_prefix(input_dir)
            .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Path error: {}", e)))?;

        let mut output_path = output_dir.join(relative);
        output_path.set_extension("ttl");

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to create parent dir: {}", e)))?;
        }

        match converter.convert_file(path, &output_path) {
            Ok(_) => {
                eprintln!("✓ {} -> {}", path.display(), output_path.display());
                converted += 1;
            }
            Err(e) => {
                eprintln!("✗ {} : {}", path.display(), e);
                errors += 1;
            }
        }
    }

    eprintln!("\nConversion complete:");
    eprintln!("  Converted: {}", converted);
    eprintln!("  Errors: {}", errors);

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn validate_file(converter: &XmlToTtlConverter, file: &PathBuf) -> LegacyResult<()> {
    eprintln!("Validating: {}", file.display());

    let ttl = std::fs::read_to_string(file)
        .map_err(|e| knhk_workflow_xml_legacy::LegacyError::Io(format!("Failed to read file: {}", e)))?;

    converter.validate_ttl(&ttl)?;

    eprintln!("✓ Validation passed");
    Ok(())
}
