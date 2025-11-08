#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(pub u8);

// 1..43 are valid per YAWL patterns
pub const ALL_PATTERN_IDS: [PatternId; 43] = {
    let mut arr: [PatternId; 43] = [PatternId(1); 43];
    let mut i = 0;
    while i < 43 { arr[i] = PatternId((i+1) as u8); i += 1; }
    arr
};