//! Page-table tree dumping for debug / bring-up.

use core::fmt;

use crate::arch::Arch;
use crate::structures::PhysicalAddressRange;

use super::entry::PageTableEntry;
use super::table::PageTable;

/// Controls [`PageTable::debug_tree_with`] output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugTreeOpts {
    /// Skip repetitive kernel RWX/PMA 4KiB leaves.
    pub special_only: bool,
    /// Summarize skipped runs as `[i..j] ×N …` instead of omitting silently.
    pub summarize_skipped: bool,
}

impl Default for DebugTreeOpts {
    fn default() -> Self {
        Self {
            special_only: true,
            summarize_skipped: false,
        }
    }
}

impl DebugTreeOpts {
    pub const ALL: Self = Self {
        special_only: false,
        summarize_skipped: false,
    };
}

impl<A: Arch> PageTable<A> {
    pub fn debug_tree(&self, f: impl Fn(fmt::Arguments<'_>)) {
        self.debug_tree_with(DebugTreeOpts::ALL, f);
    }

    /// Hide boring Limine-style RAM leaves (no log line per skipped run).
    pub fn debug_tree_special(&self, f: impl Fn(fmt::Arguments<'_>)) {
        self.debug_tree_with(DebugTreeOpts::default(), f);
    }

    pub fn debug_tree_with(&self, opts: DebugTreeOpts, f: impl Fn(fmt::Arguments<'_>)) {
        self.debug_tree_inner(&f, 0, opts);
    }

    /// Dump only leaves whose physical mapping overlaps one of `ranges`.
    ///
    /// Table pointers are printed only when a descendant leaf matches, so this
    /// stays quiet for large HHDM RAM regions when filtering for MMIO / IO maps.
    pub fn debug_tree_for_physical_ranges(
        &self,
        ranges: &[PhysicalAddressRange],
        f: impl Fn(fmt::Arguments<'_>),
    ) {
        if ranges.is_empty() {
            return;
        }
        let _ = self.debug_tree_ranges_inner(&f, 0, ranges);
    }

    fn debug_tree_inner(&self, f: &dyn Fn(fmt::Arguments<'_>), depth: usize, opts: DebugTreeOpts) {
        let indent = tree_indent(depth);
        f(format_args!(
            "{indent}PageTable L{} @ {:#x}",
            self.level(),
            self.physical_address().address()
        ));

        let mut skip_start: Option<usize> = None;
        let mut skip_pte: Option<usize> = None;

        for i in 0..A::PAGE_ENTRIES {
            let Some(entry) = (unsafe { self.entry(i) }) else {
                flush_skipped(self, f, indent, opts, skip_start, i.saturating_sub(1), skip_pte);
                skip_start = None;
                skip_pte = None;
                continue;
            };

            if !entry.present() {
                flush_skipped(self, f, indent, opts, skip_start, i.saturating_sub(1), skip_pte);
                skip_start = None;
                skip_pte = None;
                continue;
            }

            if entry.is_table_pointer() {
                flush_skipped(self, f, indent, opts, skip_start, i.saturating_sub(1), skip_pte);
                skip_start = None;
                skip_pte = None;

                f(format_args!(
                    "{indent}  [{i}] table pte={:#x} -> {:#x}",
                    entry.data(),
                    entry.address().ok().map(|p| p.address()).unwrap_or(0)
                ));
                if let Some(next_level_table) = unsafe { self.next_level_table(i) } {
                    next_level_table.debug_tree_inner(f, depth + 1, opts);
                }
                continue;
            }

            if opts.special_only && !entry.is_noteworthy(self.level()) {
                if skip_start.is_none() {
                    skip_start = Some(i);
                    skip_pte = Some(entry.data());
                } else if skip_pte != Some(entry.data()) {
                    flush_skipped(self, f, indent, opts, skip_start, i - 1, skip_pte);
                    skip_start = Some(i);
                    skip_pte = Some(entry.data());
                }
                continue;
            }

            flush_skipped(self, f, indent, opts, skip_start, i.saturating_sub(1), skip_pte);
            skip_start = None;
            skip_pte = None;

            let kind = if entry.is_leaf() { "leaf" } else { "invalid" };
            let phys = entry.address().ok().map(|p| p.address()).unwrap_or(0);
            let va = self.entry_base_address(i).map(|v| v.address()).unwrap_or(0);
            f(format_args!(
                "{indent}  [{i}] {kind} va={va:#x} phys={phys:#x} pte={:#x} flags={:?}",
                entry.data(),
                entry.flags()
            ));
        }

        flush_skipped(
            self,
            f,
            indent,
            opts,
            skip_start,
            A::PAGE_ENTRIES.saturating_sub(1),
            skip_pte,
        );
    }

    fn debug_tree_ranges_inner(
        &self,
        f: &dyn Fn(fmt::Arguments<'_>),
        depth: usize,
        ranges: &[PhysicalAddressRange],
    ) -> bool {
        if !self.subtree_overlaps_physical(ranges) {
            return false;
        }

        let indent = tree_indent(depth);
        f(format_args!(
            "{indent}PageTable L{} @ {:#x}",
            self.level(),
            self.physical_address().address()
        ));

        for i in 0..A::PAGE_ENTRIES {
            let Some(entry) = (unsafe { self.entry(i) }) else {
                continue;
            };
            if !entry.present() {
                continue;
            }

            if entry.is_table_pointer() {
                let Some(next) = (unsafe { self.next_level_table(i) }) else {
                    continue;
                };
                if !next.subtree_overlaps_physical(ranges) {
                    continue;
                }
                f(format_args!(
                    "{indent}  [{i}] table pte={:#x} -> {:#x}",
                    entry.data(),
                    entry.address().ok().map(|p| p.address()).unwrap_or(0)
                ));
                next.debug_tree_ranges_inner(f, depth + 1, ranges);
                continue;
            }

            if !leaf_overlaps_ranges(&entry, self.level(), ranges) {
                continue;
            }

            let kind = if entry.is_leaf() { "leaf" } else { "invalid" };
            let phys = entry.address().ok().map(|p| p.address()).unwrap_or(0);
            let va = self.entry_base_address(i).map(|v| v.address()).unwrap_or(0);
            f(format_args!(
                "{indent}  [{i}] {kind} va={va:#x} phys={phys:#x} pte={:#x} flags={:?}",
                entry.data(),
                entry.flags()
            ));
        }

        true
    }

    fn subtree_overlaps_physical(&self, ranges: &[PhysicalAddressRange]) -> bool {
        for i in 0..A::PAGE_ENTRIES {
            let Some(entry) = (unsafe { self.entry(i) }) else {
                continue;
            };
            if !entry.present() {
                continue;
            }
            if entry.is_table_pointer() {
                if let Some(next) = unsafe { self.next_level_table(i) } {
                    if next.subtree_overlaps_physical(ranges) {
                        return true;
                    }
                }
            } else if leaf_overlaps_ranges(&entry, self.level(), ranges) {
                return true;
            }
        }
        false
    }
}

fn leaf_phys_range<A: Arch>(entry: &PageTableEntry<A>, level: usize) -> Option<PhysicalAddressRange> {
    let base = entry.address().ok()?;
    let size = 1usize << (A::PAGE_SHIFT + level * A::PAGE_ENTRY_SHIFT);
    Some(PhysicalAddressRange::from_base_len(base, size))
}

fn leaf_overlaps_ranges<A: Arch>(
    entry: &PageTableEntry<A>,
    level: usize,
    ranges: &[PhysicalAddressRange],
) -> bool {
    let Some(leaf) = leaf_phys_range(entry, level) else {
        return false;
    };
    ranges.iter().any(|range| leaf.overlaps(*range))
}

fn flush_skipped<A: Arch>(
    table: &PageTable<A>,
    f: &dyn Fn(fmt::Arguments<'_>),
    indent: &str,
    opts: DebugTreeOpts,
    start: Option<usize>,
    end: usize,
    pte: Option<usize>,
) {
    let (Some(start), Some(pte)) = (start, pte) else {
        return;
    };
    if !opts.summarize_skipped {
        return;
    }
    if start == end {
        f(format_args!(
            "{indent}  [{start}] … skipped kernel leaf pte={pte:#x}"
        ));
    } else {
        let count = end - start + 1;
        let va = table
            .entry_base_address(start)
            .map(|v| v.address())
            .unwrap_or(0);
        f(format_args!(
            "{indent}  [{start}..{end}] ×{count} kernel leaf pte={pte:#x} va={va:#x}.."
        ));
    }
}

const fn tree_indent(depth: usize) -> &'static str {
    match depth {
        0 => "",
        1 => "  ",
        2 => "    ",
        3 => "      ",
        _ => "        ",
    }
}
