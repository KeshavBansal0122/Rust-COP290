use crate::common::cell_value::CellValue;
use crate::common::structs::AbsCell;
use crate::embedded_backend::simple::EmbeddedBackend;
use egui::{Color32, FontId, Key, RichText, TextEdit};
use rfd::FileDialog;
use std::path::PathBuf;

pub struct SpreadsheetApp {
    backend: EmbeddedBackend,
    view_top_left: AbsCell,
    selected_cell: AbsCell,
    editing: bool,
    inline_editing: bool,
    inline_edit_value: String,
    formula_input: String,
    status_message: String,
    display_rows: i16,
    display_cols: i16,
    col_width: f32,
    row_height: f32,
    show_save_dialog: bool,
    show_load_dialog: bool,
    save_path: Option<PathBuf>,
    copied_cell: Option<AbsCell>,
    search_value: String,
    show_search_panel: bool,
    last_search_position: Option<AbsCell>,
}

impl Default for SpreadsheetApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SpreadsheetApp {
    pub fn new() -> Self {
        let backend = EmbeddedBackend::new(999, 18278);

        Self {
            backend,
            copied_cell: None,
            view_top_left: AbsCell::new(0, 0),
            selected_cell: AbsCell::new(0, 0),
            editing: false,
            inline_editing: false,
            inline_edit_value: String::new(),
            formula_input: String::new(),
            status_message: String::from("Ready"),
            display_rows: 10,
            display_cols: 10,
            col_width: 100.0,
            row_height: 30.0,
            show_save_dialog: false,
            show_load_dialog: false,
            save_path: None,
            // Initialize new search fields
            search_value: String::new(),
            show_search_panel: false,
            last_search_position: None,
        }
    }

    // Add new methods for search functionality
    fn toggle_search_panel(&mut self) {
        self.show_search_panel = !self.show_search_panel;
        if self.show_search_panel {
            self.search_value = String::new();
            self.last_search_position = None;
        }
    }

    fn search_next(&mut self) {
        if self.search_value.is_empty() {
            self.status_message = "Search value cannot be empty".to_string();
            return;
        }

        let start_cell = if let Some(last_pos) = self.last_search_position {
            last_pos
        } else {
            self.selected_cell
        };

        match self.backend.search(start_cell, &self.search_value) {
            Some(found_cell) => {
                self.selected_cell = found_cell;
                self.last_search_position = Some(found_cell);
                self.status_message = format!(
                    "Found match at {}{}",
                    Self::cell_to_label(found_cell.col),
                    found_cell.row + 1
                );

                // Ensure the found cell is visible in the viewport
                self.ensure_cell_visible(found_cell);

                // Update formula input for the selected cell
                if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                    self.formula_input = format!("={}", formula);
                } else {
                    self.formula_input = self.render_cell_value(self.selected_cell);
                }
            }
            None => {
                self.status_message = format!("No more matches found for '{}'", self.search_value);
                // Reset search position to start from beginning next time
                self.last_search_position = None;
            }
        }
    }

    fn search_from_beginning(&mut self) {
        if self.search_value.is_empty() {
            self.status_message = "Search value cannot be empty".to_string();
            return;
        }

        match self.backend.search_from_start(&self.search_value) {
            Some(found_cell) => {
                self.selected_cell = found_cell;
                self.last_search_position = Some(found_cell);
                self.status_message = format!(
                    "Found match at {}{}",
                    Self::cell_to_label(found_cell.col),
                    found_cell.row + 1
                );

                // Ensure the found cell is visible in the viewport
                self.ensure_cell_visible(found_cell);

                // Update formula input for the selected cell
                if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                    self.formula_input = format!("={}", formula);
                } else {
                    self.formula_input = self.render_cell_value(self.selected_cell);
                }
            }
            None => {
                self.status_message = format!("No matches found for '{}'", self.search_value);
                self.last_search_position = None;
            }
        }
    }

    // Helper method to ensure a cell is visible in the viewport
    fn ensure_cell_visible(&mut self, cell: AbsCell) {
        // Check if cell is outside visible area and adjust view if needed
        if cell.row < self.view_top_left.row {
            self.view_top_left.row = cell.row;
        } else if cell.row >= self.view_top_left.row + self.display_rows {
            self.view_top_left.row = cell.row - self.display_rows + 1;
        }

        if cell.col < self.view_top_left.col {
            self.view_top_left.col = cell.col;
        } else if cell.col >= self.view_top_left.col + self.display_cols {
            self.view_top_left.col = cell.col - self.display_cols + 1;
        }
    }

    fn copy_cell(&mut self) {
        self.copied_cell = Some(self.selected_cell);
        self.status_message = format!(
            "Copied cell {}{}",
            Self::cell_to_label(self.selected_cell.col),
            self.selected_cell.row + 1
        );
    }

    fn paste_cell(&mut self) {
        if let Some(source_cell) = self.copied_cell {
            if source_cell == self.selected_cell {
                self.status_message = "Cannot paste to same cell".to_string();
                return;
            }

            match self
                .backend
                .copy_cell_expression(source_cell, self.selected_cell)
            {
                Ok(_) => {
                    self.status_message = format!(
                        "Pasted from {}{} to {}{}",
                        Self::cell_to_label(source_cell.col),
                        source_cell.row + 1,
                        Self::cell_to_label(self.selected_cell.col),
                        self.selected_cell.row + 1
                    );

                    // Update formula input for the selected cell
                    if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                        self.formula_input = format!("={}", formula);
                    } else {
                        self.formula_input = self.render_cell_value(self.selected_cell);
                    }
                }
                Err(err) => {
                    self.status_message = format!("Paste error: {:?}", err);
                }
            }
        } else {
            self.status_message = "Nothing to paste".to_string();
        }
    }

    fn cell_to_label(col: i16) -> String {
        let mut result = String::new();
        let mut n = col as u32 + 1;

        while n > 0 {
            n -= 1;
            let c = (b'A' + (n % 26) as u8) as char;
            result.insert(0, c);
            n /= 26;
        }

        result
    }

    fn render_cell_value(&self, cell: AbsCell) -> String {
        match self.backend.get_cell_value(cell) {
            Ok(CellValue::Empty) => String::new(),
            Ok(CellValue::Number(num)) => format!("{}", num),
            Ok(CellValue::String(text)) => text.clone(),
            Err(_) => "#ERROR".to_string(),
        }
    }

    fn handle_cell_edit(&mut self, new_value: &str) {
        #[allow(clippy::manual_strip)]
        if new_value.starts_with('=') {
            match self
                .backend
                .set_cell_formula(self.selected_cell, &new_value[1..])
            {
                Ok(_) => self.status_message = "Formula updated".to_string(),
                Err(err) => self.status_message = format!("Formula error: {:?}", err),
            }
        } else if new_value.is_empty() {
            self.backend.set_cell_empty(self.selected_cell);
            self.status_message = "Cell cleared".to_string();
        } else if let Ok(num) = new_value.parse::<f64>() {
            self.backend
                .set_cell_value(self.selected_cell, CellValue::Number(num));
            self.status_message = "Number set".to_string();
        } else {
            self.backend
                .set_cell_value(self.selected_cell, CellValue::String(new_value.to_string()));
            self.status_message = "Text set".to_string();
        }
        self.formula_input = String::new();
        self.editing = false;
        self.inline_editing = false; // Reset inline editing flag when done

        // Force a refresh of all visible cells by triggering a recalculation
        // This ensures that any formulas dependent on the edited cell are updated
        // self.refresh_viewport_cells();
    }
    fn move_view(&mut self, row_delta: i16, col_delta: i16) {
        let new_row = self.view_top_left.row + row_delta;
        let new_col = self.view_top_left.col + col_delta;

        self.view_top_left.row = new_row.max(0).min(999 - self.display_rows);
        self.view_top_left.col = new_col.max(0).min(18278 - self.display_cols);
    }

    fn move_selection(&mut self, row_delta: i16, col_delta: i16) {
        // Calculate new position
        let new_row = self.selected_cell.row + row_delta;
        let new_col = self.selected_cell.col + col_delta;

        // Constrain to grid bounds
        let new_row = new_row.clamp(0, 998);
        let new_col = new_col.clamp(0, 18277);

        self.selected_cell.row = new_row;
        self.selected_cell.col = new_col;

        // Adjust view if selection would be outside visible area
        if self.selected_cell.row < self.view_top_left.row {
            self.view_top_left.row = self.selected_cell.row;
        } else if self.selected_cell.row >= self.view_top_left.row + self.display_rows {
            self.view_top_left.row = self.selected_cell.row - self.display_rows + 1;
        }

        if self.selected_cell.col < self.view_top_left.col {
            self.view_top_left.col = self.selected_cell.col;
        } else if self.selected_cell.col >= self.view_top_left.col + self.display_cols {
            self.view_top_left.col = self.selected_cell.col - self.display_cols + 1;
        }

        // Update formula input if not editing
        if !self.editing {
            if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                self.formula_input = format!("={}", formula);
            } else {
                self.formula_input = self.render_cell_value(self.selected_cell);
            }
        }
    }

    fn save_spreadsheet(&mut self) {
        if let Some(path) = &self.save_path {
            match std::fs::File::create(path) {
                Ok(file) => {
                    if let Err(e) = self.backend.save_to_file(&file) {
                        self.status_message = format!("Error saving file: {}", e);
                    } else {
                        self.status_message = format!("File saved to {:?}", path);
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error creating file: {}", e);
                }
            }
        } else {
            self.show_save_dialog = true;
        }
    }

    fn load_spreadsheet(&mut self) {
        self.show_load_dialog = true;
    }

    fn export_to_csv(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .save_file()
        {
            let bottom_right = AbsCell::new(
                self.view_top_left.row + self.display_rows - 1,
                self.view_top_left.col + self.display_cols - 1,
            );

            match self
                .backend
                .save_range_to_csv(self.view_top_left, bottom_right, &path)
            {
                Ok(_) => self.status_message = format!("Exported to CSV: {:?}", path),
                Err(e) => self.status_message = format!("CSV export error: {}", e),
            }
        }
    }

    // New method to start inline editing
    fn start_inline_editing(&mut self) {
        if !self.inline_editing {
            self.inline_editing = true;
            self.editing = true;
            // Initialize with current cell value or formula
            if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                self.inline_edit_value = format!("={}", formula);
            } else {
                self.inline_edit_value = self.render_cell_value(self.selected_cell);
            }
        }
    }
}

impl eframe::App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file dialogs
        if self.show_save_dialog {
            if let Some(path) = FileDialog::new()
                .add_filter("Spreadsheet files", &["xlsx", "sheet"])
                .save_file()
            {
                self.save_path = Some(path);
                self.save_spreadsheet();
            }
            self.show_save_dialog = false;
        }

        if self.show_load_dialog {
            if let Some(path) = FileDialog::new()
                .add_filter("Spreadsheet files", &["xlsx", "sheet"])
                .pick_file()
            {
                match std::fs::File::open(&path) {
                    Ok(file) => match EmbeddedBackend::from_file(&file) {
                        Ok(new_backend) => {
                            self.backend = new_backend;
                            self.status_message = format!("Loaded from {:?}", path);
                            self.save_path = Some(path);
                        }
                        Err(e) => {
                            self.status_message = format!("Error loading file: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error opening file: {}", e);
                    }
                }
            }
            self.show_load_dialog = false;
        }

        // Handle keyboard inputs
        if self.show_search_panel {
            // When search panel is active, handle search-specific keys
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.show_search_panel = false;
            }

            // F3 to search for next occurrence
            if ctx.input(|i| i.key_pressed(Key::F3)) {
                self.search_next();
            }

            // Shift+F3 to search from beginning
            if ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::F3)) {
                self.search_from_beginning();
            }
        } else if !self.inline_editing {
            // When search panel is NOT active and not editing a cell
            // Ctrl+F to open search panel
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::F)) {
                self.toggle_search_panel();
            }

            // F3 to quickly open search and search next
            if ctx.input(|i| i.key_pressed(Key::F3)) {
                self.show_search_panel = true;
            }
        }

        if self.inline_editing {
            // Check for Escape key specifically to handle it more reliably
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.inline_editing = false;
                self.editing = false;
                // Restore the formula input to the original value
                if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                    self.formula_input = format!("={}", formula);
                } else {
                    self.formula_input = self.render_cell_value(self.selected_cell);
                }
            }
        } else {
            // Handle navigation keys when not editing
            if ctx.input(|i| i.key_pressed(Key::Tab))
                || ctx.input(|i| i.key_pressed(Key::ArrowRight))
            {
                self.move_selection(0, 1);
            }
            if ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::Tab))
                || ctx.input(|i| i.key_pressed(Key::ArrowLeft))
            {
                self.move_selection(0, -1);
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                self.move_selection(-1, 0);
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                self.move_selection(1, 0);
            }
            if ctx.input(|i| i.key_pressed(Key::Enter)) {
                // Enter key should start editing mode instead of moving down
                self.start_inline_editing();
            }
            if ctx.input(|i| i.key_pressed(Key::PageUp)) {
                self.move_selection(-self.display_rows, 0);
            }
            if ctx.input(|i| i.key_pressed(Key::PageDown)) {
                self.move_selection(self.display_rows, 0);
            }
            //copy
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::C)) {
                self.copy_cell();
            }

            // Ctrl+V for paste
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::V)) {
                self.paste_cell();
            }

            // Ctrl+Z for undo
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Z)) {
                if self.backend.undo() {
                    self.status_message = "Undo successful".to_string();
                    // Update formula input for selected cell
                    if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                        self.formula_input = format!("={}", formula);
                    } else {
                        self.formula_input = self.render_cell_value(self.selected_cell);
                    }
                } else {
                    self.status_message = "Nothing to undo".to_string();
                }
            }

            // Ctrl+Y for redo
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Y)) {
                if self.backend.redo() {
                    self.status_message = "Redo successful".to_string();
                    // Update formula input for selected cell
                    if let Some(formula) = self.backend.get_cell_formula(self.selected_cell) {
                        self.formula_input = format!("={}", formula);
                    } else {
                        self.formula_input = self.render_cell_value(self.selected_cell);
                    }
                } else {
                    self.status_message = "Nothing to redo".to_string();
                }
            }

            // Ctrl+S for save
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::S)) {
                self.save_spreadsheet();
            }

            // Ctrl+O for open
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::O)) {
                self.load_spreadsheet();
            }

            // Ctrl+E for export current view to CSV
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::E)) {
                self.export_to_csv();
            }

            // Start editing on F2 or when typing any printable character
            if ctx.input(|i| {
                i.key_pressed(Key::F2)
                    || (!i.modifiers.ctrl
                        && !i.modifiers.alt
                        && !i.key_down(Key::Tab)
                        && i.events.iter().any(|e| {
                            if let egui::Event::Text(text) = e {
                                !text.is_empty() && text.chars().next().unwrap().is_ascii_graphic()
                            } else {
                                false
                            }
                        }))
            }) {
                self.editing = true;

                // Also start inline editing if we detect text input
                if ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Text(_)))) {
                    self.start_inline_editing();
                    // Capture the first character typed
                    if let Some(egui::Event::Text(text)) = ctx.input(|i| {
                        i.events
                            .iter()
                            .find(|e| matches!(e, egui::Event::Text(_)))
                            .cloned()
                    }) {
                        self.inline_edit_value = text;
                    }
                }
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.backend = EmbeddedBackend::new(999, 18278);
                        self.view_top_left = AbsCell::new(0, 0);
                        self.selected_cell = AbsCell::new(0, 0);
                        self.formula_input = String::new();
                        self.save_path = None;
                        self.status_message = "New spreadsheet created".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        self.load_spreadsheet();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_spreadsheet();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.show_save_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export to CSV...").clicked() {
                        self.export_to_csv();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Search", |ui| {
                    if ui.button("Find...").clicked() {
                        self.toggle_search_panel();
                        ui.close_menu();
                    }
                    if ui.button("Find Next").clicked() {
                        self.search_next();
                        ui.close_menu();
                    }
                    if ui.button("Find From Beginning").clicked() {
                        self.search_from_beginning();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy").clicked() {
                        self.copy_cell();
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        self.paste_cell();
                        ui.close_menu();
                    }
                    ui.separator();

                    if ui.button("Undo").clicked() {
                        if self.backend.undo() {
                            self.status_message = "Undo successful".to_string();
                            if let Some(formula) = self.backend.get_cell_formula(self.selected_cell)
                            {
                                self.formula_input = format!("={}", formula);
                            } else {
                                self.formula_input = self.render_cell_value(self.selected_cell);
                            }
                        } else {
                            self.status_message = "Nothing to undo".to_string();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        if self.backend.redo() {
                            self.status_message = "Redo successful".to_string();
                            if let Some(formula) = self.backend.get_cell_formula(self.selected_cell)
                            {
                                self.formula_input = format!("={}", formula);
                            } else {
                                self.formula_input = self.render_cell_value(self.selected_cell);
                            }
                        } else {
                            self.status_message = "Nothing to redo".to_string();
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Navigation", |ui| {
                    if ui.button("Go to Cell...").clicked() {
                        // TODO: Implement cell navigation popup
                        ui.close_menu();
                    }
                });
            });
        });

        if self.show_search_panel {
            egui::TopBottomPanel::top("search_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");

                    // Search input field with input capture
                    let mut search_text = self.search_value.clone();
                    let text_edit = TextEdit::singleline(&mut search_text)
                        .desired_width(ui.available_width() * 0.5)
                        .font(FontId::proportional(14.0))
                        .hint_text("Type to search...");

                    let response = ui.add(text_edit);

                    // Set focus to the search field when panel first opens
                    if self.search_value.is_empty() {
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    }

                    // Update search value and handle Enter key
                    if response.changed() {
                        self.search_value = search_text;
                    }

                    if (response.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)))
                        || (response.has_focus() && ctx.input(|i| i.key_pressed(Key::Enter)))
                    {
                        self.search_next();
                        // Return focus to search field after searching
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    }

                    // Search buttons
                    if ui.button("Search Next").clicked() {
                        self.search_next();
                        // Return focus to search field
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    }

                    if ui.button("From Beginning").clicked() {
                        self.search_from_beginning();
                        // Return focus to search field
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    }

                    if ui.button("Close").clicked() {
                        self.show_search_panel = false;
                    }
                });
            });

            // The search panel is modal by nature
            // We don't need explicit event consumption since we're checking
            // for self.show_search_panel before starting cell editing elsewhere
        }

        // Formula bar
        egui::TopBottomPanel::top("formula_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}{}:",
                    Self::cell_to_label(self.selected_cell.col),
                    self.selected_cell.row + 1
                ));

                let mut input = self.formula_input.clone();
                let text_edit = TextEdit::singleline(&mut input)
                    .desired_width(ui.available_width())
                    .font(FontId::proportional(16.0));

                let response = ui.add(text_edit);

                if response.gained_focus() {
                    self.editing = true;
                }

                if self.editing && !self.inline_editing {
                    self.formula_input = input;

                    if response.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)) {
                        self.handle_cell_edit(&self.formula_input.clone());
                    }
                }
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "View: {}{} to {}{}",
                        Self::cell_to_label(self.view_top_left.col),
                        self.view_top_left.row + 1,
                        Self::cell_to_label(self.view_top_left.col + self.display_cols - 1),
                        self.view_top_left.row + self.display_rows
                    ));
                });
            });
        });

        // Main spreadsheet area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Navigation buttons
            ui.horizontal(|ui| {
                if ui.button("⬅️").clicked() {
                    self.move_view(0, -1);
                }
                if ui.button("➡️").clicked() {
                    self.move_view(0, 1);
                }
                if ui.button("⬆️").clicked() {
                    self.move_view(-1, 0);
                }
                if ui.button("⬇️").clicked() {
                    self.move_view(1, 0);
                }
                if ui.button("⏮️").clicked() {
                    self.view_top_left.col = 0;
                }
                if ui.button("⏭️").clicked() {
                    self.view_top_left.col = 18278 - self.display_cols;
                }
                if ui.button("⏫").clicked() {
                    self.view_top_left.row = 0;
                }
                if ui.button("⏬").clicked() {
                    self.view_top_left.row = 999 - self.display_rows;
                }
            });

            let table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto().at_least(40.0))
                .columns(
                    egui_extras::Column::auto().at_least(self.col_width),
                    self.display_cols as usize,
                );

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });

                    for col in 0..self.display_cols {
                        let col_idx = self.view_top_left.col + col;
                        header.col(|ui| {
                            ui.strong(Self::cell_to_label(col_idx));
                        });
                    }
                })
                .body(|mut body| {
                    for row in 0..self.display_rows {
                        let row_idx = self.view_top_left.row + row;
                        body.row(self.row_height, |mut row| {
                            // Row header
                            row.col(|ui| {
                                ui.strong(format!("{}", row_idx + 1));
                            });

                            // Cell data
                            for col in 0..self.display_cols {
                                let col_idx = self.view_top_left.col + col;
                                let cell = AbsCell::new(row_idx, col_idx);
                                let is_selected = self.selected_cell.row == row_idx
                                    && self.selected_cell.col == col_idx;

                                row.col(|ui| {
                                    // Check if this is the selected cell and we're inline editing
                                    if is_selected && self.inline_editing {
                                        // Show editable text field for this cell
                                        let mut edit_value = self.inline_edit_value.clone();
                                        let text_edit = TextEdit::singleline(&mut edit_value)
                                            .desired_width(ui.available_width())
                                            .font(FontId::proportional(14.0))
                                            .margin(egui::vec2(2.0, 0.0))
                                            .frame(true);

                                        let response = ui.add(text_edit);
                                        ui.memory_mut(|mem| mem.request_focus(response.id));

                                        self.inline_edit_value = edit_value;

                                        // Handle completion of editing
                                        if ctx.input(|i| i.key_pressed(Key::Enter)) {
                                            // Commit changes when Enter is pressed
                                            self.handle_cell_edit(&self.inline_edit_value.clone());
                                        } else if ctx.input(|i| i.key_pressed(Key::Escape)) {
                                            // Cancel editing when Escape is pressed
                                            self.inline_editing = false;
                                            self.editing = false;
                                            // Restore the formula input to the original value
                                            if let Some(formula) =
                                                self.backend.get_cell_formula(self.selected_cell)
                                            {
                                                self.formula_input = format!("={}", formula);
                                            } else {
                                                self.formula_input =
                                                    self.render_cell_value(self.selected_cell);
                                            }
                                        } else if ctx.input(|i| i.key_pressed(Key::Tab)) {
                                            // Commit changes and move to next/previous cell when Tab is pressed
                                            self.handle_cell_edit(&self.inline_edit_value.clone());
                                            if ctx.input(|i| i.modifiers.shift) {
                                                self.move_selection(0, -1);
                                            } else {
                                                self.move_selection(0, 1);
                                            }
                                        } else if response.lost_focus()
                                            && !ctx.input(|i| i.key_pressed(Key::Escape))
                                        {
                                            // Commit changes when focus is lost (unless it's because Escape was pressed)
                                            self.handle_cell_edit(&self.inline_edit_value.clone());
                                        }
                                    } else {
                                        let cell_value = self.render_cell_value(cell);

                                        // Get the text ready
                                        let text = RichText::new(&cell_value);
                                        let text = if is_selected { text.strong() } else { text };

                                        // Create the cell area - important: use the full rect here
                                        let rect = ui.available_rect_before_wrap();

                                        // Draw cell background if selected
                                        if is_selected {
                                            // ui.painter().rect_filled(
                                            //     rect,
                                            //     0.0,
                                            //     Color32::from_rgb(0, 0, 0), // Light blue background
                                            // );

                                            // Border for selected cell
                                            ui.painter().rect_stroke(
                                                rect,
                                                0.0,
                                                egui::Stroke::new(
                                                    2.0,
                                                    Color32::from_rgb(0, 90, 180),
                                                ), // Darker blue border
                                                egui::StrokeKind::Middle,
                                            );
                                        }

                                        // Add the label with its text
                                        ui.add(egui::Label::new(text));

                                        // Add an invisible button over the entire cell area to capture clicks
                                        // Position it at the same place as the cell
                                        let response = ui.put(
                                            rect,
                                            egui::Button::new("") // Empty text
                                                .frame(false)  // No visible frame
                                                .fill(Color32::TRANSPARENT) // Transparent fill
                                        );

                                        // Handle clicks on the invisible button covering the entire cell
                                        if response.clicked() {
                                            // If we were editing another cell, commit those changes
                                            if self.inline_editing {
                                                self.handle_cell_edit(
                                                    &self.inline_edit_value.clone(),
                                                );
                                            }

                                            self.selected_cell = cell;
                                            self.inline_editing = false;
                                            self.editing = false;

                                            // Update formula input when selecting a cell
                                            if let Some(formula) =
                                                self.backend.get_cell_formula(self.selected_cell)
                                            {
                                                self.formula_input = format!("={}", formula);
                                            } else {
                                                self.formula_input = cell_value;
                                            }
                                        }

                                        // Double-click starts editing
                                        if response.double_clicked() {
                                            self.selected_cell = cell;
                                            self.start_inline_editing();
                                        }
                                    }
                                });
                            }
                        });
                    }
                });
        });

        // Request repaint to keep the UI responsive
        ctx.request_repaint();
    }
}

// Main entry point
pub fn run_spreadsheet_app() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Spreadsheet",
        options,
        Box::new(|_cc| Ok(Box::new(SpreadsheetApp::new()))),
    )
}

// Add this to your main.rs
/*
fn main() -> Result<(), eframe::Error> {
    run_spreadsheet_app()
}
*/
