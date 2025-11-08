// rust/knhk-cli/src/commands/hook.rs
// Hook commands - Knowledge hook operations using knhk-hot FFI

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Hook storage entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookEntry {
    pub id: String,
    pub name: String,
    pub op: String,
    pub pred: u64,
    pub off: u64,
    pub len: u64,
    pub s: Option<u64>,
    pub p: Option<u64>,
    pub o: Option<u64>,
    pub k: Option<u64>,
}

/// Hook storage
#[derive(Debug, Serialize, Deserialize)]
struct HookStorage {
    hooks: Vec<HookEntry>,
}

/// Create a hook
/// hook(#{name, op, run := #{pred, off, len}, args})
#[allow(clippy::too_many_arguments)]
pub fn create(
    name: String,
    op: String,
    pred: u64,
    off: u64,
    len: u64,
    s: Option<u64>,
    p: Option<u64>,
    o: Option<u64>,
    k: Option<u64>,
) -> Result<(), String> {
    println!("Creating hook: {}", name);
    println!("  Operation: {}", op);
    println!("  Run: pred={}, off={}, len={}", pred, off, len);

    // Validate run length ≤ 8
    if len > 8 {
        return Err(format!("Run length {} exceeds max_run_len 8", len));
    }

    // Validate operation
    let valid_ops = [
        "ASK_SP",
        "ASK_SPO",
        "COUNT_SP_GE",
        "COUNT_SP_EQ",
        "COUNT_SP_LE",
        "COUNT_O_P_GE",
        "COUNT_O_P_EQ",
        "COUNT_O_P_LE",
        "SELECT_SP",
        "VALIDATE_SP",
        "COMPARE_O",
        "CONSTRUCT8",
    ];
    let op_upper = op.to_uppercase();
    if !valid_ops.iter().any(|&o| o == op_upper) {
        return Err(format!(
            "Invalid operation: {}. Must be one of: {:?}",
            op, valid_ops
        ));
    }

    // Load existing hooks
    let mut storage = load_hooks()?;

    // Check if hook with same name exists
    if storage.hooks.iter().any(|h| h.name == name) {
        return Err(format!("Hook with name '{}' already exists", name));
    }

    // Create hook entry
    let hook_id = format!("hook_{}", storage.hooks.len() + 1);
    storage.hooks.push(HookEntry {
        id: hook_id.clone(),
        name: name.clone(),
        op: op.clone(),
        pred,
        off,
        len,
        s,
        p,
        o,
        k,
    });

    // Save hooks
    save_hooks(&storage)?;

    println!("✓ Hook created (id: {})", hook_id);

    Ok(())
}

/// List hooks
pub fn list() -> Result<Vec<String>, String> {
    let storage = load_hooks()?;

    Ok(storage.hooks.iter().map(|h| h.name.clone()).collect())
}

/// Evaluate a hook using knhk-hot FFI
pub fn eval(hook_name: String) -> Result<String, String> {
    println!("Evaluating hook: {}", hook_name);

    // Load hooks
    let storage = load_hooks()?;

    // Find hook
    let hook = storage
        .hooks
        .iter()
        .find(|h| h.name == hook_name)
        .ok_or_else(|| format!("Hook '{}' not found", hook_name))?;

    #[cfg(feature = "std")]
    {
        use knhk_hot::{Engine, Ir, Op, Receipt as HotReceipt, Run as HotRun};

        // Create dummy SoA arrays for evaluation
        // In production, these would come from loaded ontology O
        let mut s_array = [0u64; 8];
        let mut p_array = [0u64; 8];
        let mut o_array = [0u64; 8];

        // Initialize arrays with hook values
        if hook.len > 0 {
            s_array[hook.off as usize] = hook.s.unwrap_or(0);
            p_array[hook.off as usize] = hook.pred;
            o_array[hook.off as usize] = hook.o.unwrap_or(0);
        }

        // Initialize engine (unsafe FFI call)
        let mut engine =
            unsafe { Engine::new(s_array.as_ptr(), p_array.as_ptr(), o_array.as_ptr()) };

        // Pin run
        let hot_run = HotRun {
            pred: hook.pred,
            off: hook.off,
            len: hook.len,
        };

        engine
            .pin_run(hot_run)
            .map_err(|e| format!("Failed to pin run: {}", e))?;

        // Parse operation
        let op = match hook.op.to_uppercase().as_str() {
            "ASK_SP" => Op::AskSp,
            "ASK_SPO" => Op::AskSpo,
            "COUNT_SP_GE" => Op::CountSpGe,
            "COUNT_SP_EQ" => Op::CountSpEq,
            "COUNT_SP_LE" => Op::CountSpLe,
            "ASK_OP" => Op::AskOp,
            "UNIQUE_SP" => Op::UniqueSp,
            "COUNT_OP_GE" => Op::CountOpGe,
            "COUNT_OP_LE" => Op::CountOpLe,
            "COUNT_OP_EQ" => Op::CountOpEq,
            "COMPARE_O_EQ" => Op::CompareOEQ,
            "COMPARE_O_GT" => Op::CompareOGT,
            "COMPARE_O_LT" => Op::CompareOLT,
            "COMPARE_O_GE" => Op::CompareOGE,
            "COMPARE_O_LE" => Op::CompareOLE,
            "CONSTRUCT8" => Op::Construct8,
            _ => return Err(format!("Unsupported operation: {}", hook.op)),
        };

        // Create IR
        let _out_s = [0u64; 8];
        let _out_p = [0u64; 8];
        let _out_o = [0u64; 8];

        let mut ir = Ir {
            op,
            s: hook.s.unwrap_or(0),
            p: hook.pred,
            o: hook.o.unwrap_or(0),
            k: hook.k.unwrap_or(0),
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0,
        };

        // Execute hook
        let mut receipt = HotReceipt::default();

        // Execute hook via hot path
        let result = engine.eval_bool(&mut ir, &mut receipt);
        println!("  ✓ Hook executed via hot path (result: {})", result);
        println!("    Ticks: {} (budget: ≤8)", receipt.ticks);

        Ok(format!(
            "Hot path: result={}, ticks={}, lanes={}",
            result, receipt.ticks, receipt.lanes
        ))
    }

    #[cfg(not(feature = "std"))]
    {
        Err("Hook evaluation requires std feature".to_string())
    }
}

/// Show hook details
pub fn show(hook_name: String) -> Result<HookEntry, String> {
    let storage = load_hooks()?;

    let hook = storage
        .hooks
        .iter()
        .find(|h| h.name == hook_name)
        .ok_or_else(|| format!("Hook '{}' not found", hook_name))?
        .clone();

    Ok(hook)
}

fn get_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").map_err(|_| "APPDATA not set")?);
        path.push("knhk");
        Ok(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        Ok(path)
    }
}

fn load_hooks() -> Result<HookStorage, String> {
    let config_dir = get_config_dir()?;
    let hooks_file = config_dir.join("hooks.json");

    if !hooks_file.exists() {
        return Ok(HookStorage { hooks: Vec::new() });
    }

    let content =
        fs::read_to_string(&hooks_file).map_err(|e| format!("Failed to read hooks file: {}", e))?;

    let storage: HookStorage =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse hooks file: {}", e))?;

    Ok(storage)
}

fn save_hooks(storage: &HookStorage) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let hooks_file = config_dir.join("hooks.json");
    let content = serde_json::to_string_pretty(storage)
        .map_err(|e| format!("Failed to serialize hooks: {}", e))?;

    fs::write(&hooks_file, content).map_err(|e| format!("Failed to write hooks file: {}", e))?;

    Ok(())
}
