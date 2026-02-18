use ironbullet::pipeline::block::{Block, BlockSettings};

/// Recursively find a mutable block by ID in a block tree (searches inside IfElse/Loop)
pub(super) fn find_block_mut(blocks: &mut Vec<Block>, id: uuid::Uuid) -> Option<&mut Block> {
    for block in blocks.iter_mut() {
        if block.id == id {
            return Some(block);
        }
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if let Some(b) = find_block_mut(&mut s.true_blocks, id) { return Some(b); }
                if let Some(b) = find_block_mut(&mut s.false_blocks, id) { return Some(b); }
            }
            BlockSettings::Loop(s) => {
                if let Some(b) = find_block_mut(&mut s.blocks, id) { return Some(b); }
            }
            BlockSettings::Group(s) => {
                if let Some(b) = find_block_mut(&mut s.blocks, id) { return Some(b); }
            }
            _ => {}
        }
    }
    None
}

/// Recursively remove a block by ID from a block tree
pub(super) fn remove_block_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid) -> bool {
    let len = blocks.len();
    blocks.retain(|b| b.id != id);
    if blocks.len() < len {
        return true;
    }
    for block in blocks.iter_mut() {
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if remove_block_recursive(&mut s.true_blocks, id) { return true; }
                if remove_block_recursive(&mut s.false_blocks, id) { return true; }
            }
            BlockSettings::Loop(s) => {
                if remove_block_recursive(&mut s.blocks, id) { return true; }
            }
            BlockSettings::Group(s) => {
                if remove_block_recursive(&mut s.blocks, id) { return true; }
            }
            _ => {}
        }
    }
    false
}

/// Recursively set disabled on a block by ID
pub(super) fn set_block_disabled_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid, disabled: bool) -> bool {
    if let Some(block) = find_block_mut(blocks, id) {
        block.disabled = disabled;
        return true;
    }
    false
}

/// Extract a block by ID from anywhere in the tree, returning (extracted_block, success)
pub(super) fn extract_block_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid) -> Option<Block> {
    // Check top-level
    if let Some(pos) = blocks.iter().position(|b| b.id == id) {
        return Some(blocks.remove(pos));
    }
    // Check nested
    for block in blocks.iter_mut() {
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if let Some(b) = extract_block_recursive(&mut s.true_blocks, id) { return Some(b); }
                if let Some(b) = extract_block_recursive(&mut s.false_blocks, id) { return Some(b); }
            }
            BlockSettings::Loop(s) => {
                if let Some(b) = extract_block_recursive(&mut s.blocks, id) { return Some(b); }
            }
            BlockSettings::Group(s) => {
                if let Some(b) = extract_block_recursive(&mut s.blocks, id) { return Some(b); }
            }
            _ => {}
        }
    }
    None
}

/// Add a block into a nested container (IfElse branch or Loop body)
pub(super) fn add_block_to_nested(blocks: &mut Vec<Block>, parent_id: uuid::Uuid, branch: &str, new_block: Block, index: Option<usize>) -> bool {
    for block in blocks.iter_mut() {
        if block.id == parent_id {
            let target = match (&mut block.settings, branch) {
                (BlockSettings::IfElse(s), "true") => Some(&mut s.true_blocks),
                (BlockSettings::IfElse(s), "false") => Some(&mut s.false_blocks),
                (BlockSettings::Loop(s), "body") => Some(&mut s.blocks),
                (BlockSettings::Group(s), "body") => Some(&mut s.blocks),
                _ => None,
            };
            if let Some(target) = target {
                if let Some(idx) = index {
                    if idx <= target.len() {
                        target.insert(idx, new_block);
                    } else {
                        target.push(new_block);
                    }
                } else {
                    target.push(new_block);
                }
                return true;
            }
        }
        // Recurse into nested containers
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if add_block_to_nested(&mut s.true_blocks, parent_id, branch, new_block.clone(), index) { return true; }
                if add_block_to_nested(&mut s.false_blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            BlockSettings::Loop(s) => {
                if add_block_to_nested(&mut s.blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            BlockSettings::Group(s) => {
                if add_block_to_nested(&mut s.blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            _ => {}
        }
    }
    false
}
