# Files Requiring Archive or Revision Analysis

**Date**: December 2024  
**Analysis**: Review of git-tracked files that haven't been modified since initial commit

## Files to Archive

### 1. Implementation Detail Diagrams (`.mmd` files)

**Status**: Unchanged since initial commit  
**Action**: Archive

These are implementation detail diagrams that should be archived:

- `docs/construct8-architecture.mmd` - CONSTRUCT8 architecture diagram
- `docs/construct8-data-flow.mmd` - CONSTRUCT8 data flow diagram  
- `docs/construct8-flow.mmd` - CONSTRUCT8 flow diagram
- `docs/construct8-simd-operations.mmd` - SIMD operations diagram

**Reason**: These are implementation-specific diagrams. While useful for reference, they should be in archived/implementation-details/

**Recommendation**: Move to `docs/archived/implementation-details/diagrams/`

### 2. Conversation/Notes Files

**Status**: Unchanged since initial commit  
**Action**: Archive

- `docs/CONVO.txt` - Conversation/notes file (4373 lines)

**Reason**: This appears to be a conversation log or notes file. Should be archived for historical reference.

**Recommendation**: Move to `docs/archived/` or keep as reference material

### 3. Cursor IDE Configuration

**Status**: Unchanged since initial commit  
**Action**: Review and potentially move

- `docs/cursor-rules-commands.md` - Cursor IDE rules and commands

**Reason**: This is IDE configuration documentation. Should be in `.cursor/` directory or archived.

**Recommendation**: 
- If actively used: Move to `.cursor/rules/` directory
- If outdated: Archive to `docs/archived/`

### 4. Status Files

**Status**: Unchanged since initial commit  
**Action**: Archive

- `docs/unrdf-integration-status.md` - UNRDF integration status

**Reason**: Status files are typically temporary and should be archived after completion.

**Recommendation**: Move to `docs/archived/status/`

## Files Requiring Revision

### 1. Code Organization

**File**: `docs/code-organization.md`  
**Status**: Unchanged since initial commit  
**Issues**:
- References "KNKHS" instead of "KNHK" 
- May need v0.4.0 status updates

**Action**: Update references and add v0.4.0 status

### 2. Data Flow

**File**: `docs/data-flow.md`  
**Status**: Unchanged since initial commit  
**Issues**:
- References "KNKHS" instead of "KNHK"
- References "Raptor library" - may be outdated

**Action**: Update references and verify accuracy

### 3. Architecture Diagrams

**Files**: 
- `docs/architecture.mmd`
- `docs/data-flow.mmd`
- `docs/hot-path.mmd`
- `docs/performance.mmd`
- `docs/simd-optimization.mmd`

**Status**: Unchanged since initial commit  
**Action**: Review for accuracy and v0.4.0 updates

**Note**: These are diagrams (`.mmd` files). If they're still accurate, keep them. If outdated, archive.

## Files Already Properly Archived ✅

These files are correctly in the `archived/` directory:
- `docs/archived/analysis/` - Analysis documents
- `docs/archived/status/` - Status reports  
- `docs/archived/versions/` - Version-specific docs
- `docs/archived/implementation-details/` - Implementation details (recently moved)

## Summary of Actions Required

### Immediate Archive Actions
1. Move `construct8-*.mmd` files to `docs/archived/implementation-details/diagrams/`
2. Move `docs/CONVO.txt` to `docs/archived/` 
3. Move `docs/unrdf-integration-status.md` to `docs/archived/status/`
4. Review `docs/cursor-rules-commands.md` - move to `.cursor/` or archive

### Revision Actions
1. Update `docs/code-organization.md` - fix "KNKHS" → "KNHK"
2. Update `docs/data-flow.md` - fix "KNKHS" → "KNHK", verify Raptor reference
3. Review all `.mmd` diagram files for accuracy

### Git Cleanup
After archiving, commit deletions:
```bash
git rm docs/construct8-*.mmd
git rm docs/CONVO.txt
git rm docs/unrdf-integration-status.md
# (after reviewing cursor-rules-commands.md)
```

---

**Next Steps**: Execute archive actions and revisions as identified above.

