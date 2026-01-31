use ratatui::text::Text;

/// Height of the sticky file header (file name line + divider line)
pub const STICKY_FILE_HEADER_HEIGHT: usize = 2;

/// Height of the sticky hunk header (box top + marker + box bottom)
pub const STICKY_HUNK_HEADER_HEIGHT: usize = 3;

pub struct DiffState {
    pub content: Text<'static>,
    pub scroll_offset: usize,
    pub hunk_positions: Vec<usize>, // Navigation targets for hunk jumping
    pub file_header_positions: Vec<usize>, // Line positions of file headers (Δ, added:, etc.)
    pub hunk_marker_positions: Vec<usize>, // Line positions of hunk markers (•)
    pub current_hunk: usize,
    pub total_lines: usize,
    pub has_both: bool,       // Has both staged and unstaged changes
    pub showing_staged: bool, // Currently showing staged diff
}

impl DiffState {
    pub fn new() -> Self {
        Self {
            content: Text::default(),
            scroll_offset: 0,
            hunk_positions: Vec::new(),
            file_header_positions: Vec::new(),
            hunk_marker_positions: Vec::new(),
            current_hunk: 0,
            total_lines: 0,
            has_both: false,
            showing_staged: false,
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.total_lines.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
        self.update_current_hunk();
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.update_current_hunk();
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.current_hunk = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_lines.saturating_sub(1);
        self.update_current_hunk();
    }

    pub fn next_hunk(&mut self) {
        if self.current_hunk + 1 < self.hunk_positions.len() {
            self.current_hunk += 1;
            let target = self.hunk_positions[self.current_hunk];
            self.scroll_offset = self.adjusted_scroll_for_sticky(target);
        }
    }

    pub fn prev_hunk(&mut self) {
        if self.current_hunk > 0 {
            self.current_hunk -= 1;
            let target = self.hunk_positions[self.current_hunk];
            self.scroll_offset = self.adjusted_scroll_for_sticky(target);
        }
    }

    /// Adjust scroll position to account for sticky headers.
    /// When navigating to a position that will have sticky headers above it,
    /// we scroll back a bit so the target content is visible below the sticky area.
    fn adjusted_scroll_for_sticky(&self, target: usize) -> usize {
        // If navigating to a file header, no adjustment needed
        if self.file_header_positions.contains(&target) {
            return target;
        }

        // Otherwise, scroll to show the target below where sticky headers will be
        // We want the hunk box top line to be visible, so adjust for file header only
        target.saturating_sub(STICKY_FILE_HEADER_HEIGHT)
    }

    fn update_current_hunk(&mut self) {
        for (i, &pos) in self.hunk_positions.iter().enumerate().rev() {
            if self.scroll_offset >= pos {
                self.current_hunk = i;
                return;
            }
        }
        self.current_hunk = 0;
    }

    /// Get the file header line index that should be shown as sticky header.
    /// Returns Some(line_index) if we've scrolled past a file header,
    /// or None if the file header is still visible.
    pub fn sticky_file_header(&self) -> Option<usize> {
        // Find the most recent file header we've scrolled past
        let header_pos = *self
            .file_header_positions
            .iter()
            .rfind(|&&pos| pos < self.scroll_offset)?;

        // Find the next file header after the current one
        let next_header = self
            .file_header_positions
            .iter()
            .find(|&&pos| pos >= self.scroll_offset)
            .copied();

        // Don't show sticky if next header is within the sticky zone
        let sticky_zone_end = self.scroll_offset + STICKY_FILE_HEADER_HEIGHT;
        if next_header.is_some_and(|pos| pos < sticky_zone_end) {
            return None;
        }

        Some(header_pos)
    }

    /// Get the hunk marker line index that should be shown as sticky header.
    /// Returns Some(line_index) if we've scrolled past a hunk marker,
    /// or None if the hunk marker is still visible.
    /// The sticky hunk header appears below the sticky file header (3 lines: box top + marker + box bottom).
    pub fn sticky_hunk_header(&self) -> Option<usize> {
        let file_header_offset = if self.sticky_file_header().is_some() {
            STICKY_FILE_HEADER_HEIGHT
        } else {
            0
        };
        let effective_scroll = self.scroll_offset + file_header_offset;
        let sticky_zone_end = effective_scroll + STICKY_HUNK_HEADER_HEIGHT;

        // Find the most recent hunk whose box_top has entered the sticky zone
        let marker_pos = *self
            .hunk_marker_positions
            .iter()
            .rfind(|&&pos| pos.saturating_sub(1) <= effective_scroll)?;

        // Find the next hunk marker after the current one
        let next_marker = self
            .hunk_marker_positions
            .iter()
            .find(|&&pos| pos.saturating_sub(1) > effective_scroll)
            .copied();

        // Don't show sticky if next hunk's box top is within the sticky zone
        if next_marker.is_some_and(|pos| pos.saturating_sub(1) < sticky_zone_end) {
            return None;
        }

        // Don't show sticky if next file header is within the sticky zone
        let next_file_header = self
            .file_header_positions
            .iter()
            .find(|&&pos| pos > marker_pos)
            .copied();

        if next_file_header.is_some_and(|pos| pos < sticky_zone_end) {
            return None;
        }

        Some(marker_pos)
    }
}

impl Default for DiffState {
    fn default() -> Self {
        Self::new()
    }
}
