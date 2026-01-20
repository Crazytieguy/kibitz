use ratatui::text::Text;

pub struct DiffState {
    pub content: Text<'static>,
    pub scroll_offset: usize,
    pub hunk_positions: Vec<usize>,
    pub current_hunk: usize,
    pub total_lines: usize,
    pub has_both: bool,        // Has both staged and unstaged changes
    pub showing_staged: bool,  // Currently showing staged diff
}

impl DiffState {
    pub fn new() -> Self {
        Self {
            content: Text::default(),
            scroll_offset: 0,
            hunk_positions: Vec::new(),
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
            self.scroll_offset = self.hunk_positions[self.current_hunk];
        }
    }

    pub fn prev_hunk(&mut self) {
        if self.current_hunk > 0 {
            self.current_hunk -= 1;
            self.scroll_offset = self.hunk_positions[self.current_hunk];
        }
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
}

impl Default for DiffState {
    fn default() -> Self {
        Self::new()
    }
}
