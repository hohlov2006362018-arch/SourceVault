//! Main egui application for SourceVault.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use eframe::egui::{
    self, Align, CentralPanel, Color32, Layout, RichText, ScrollArea, TopBottomPanel,
};
use eframe::CreationContext;
use sourcevault_core::{formats, Archive, ArchiveEntry, EntryKind, OpenedArchive};

use crate::i18n::{self, Lang};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Persisted state between launches (window position, last folder, language…).
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Settings {
    lang: Option<String>,
    last_open_dir: Option<PathBuf>,
    last_extract_dir: Option<PathBuf>,
}

pub struct SourceVaultApp {
    settings: Settings,
    lang: Lang,
    archive: Option<LoadedArchive>,
    selected_dir: Option<String>,
    selected_entry: Option<String>,
    expanded: BTreeSet<String>,
    error: Option<String>,
    show_about: bool,
    preview_cache: Option<(String, PreviewKind)>,
}

struct LoadedArchive {
    path: PathBuf,
    inner: OpenedArchive,
    tree: TreeNode,
    total_size: u64,
}

#[derive(Default, Debug)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    files: BTreeMap<String, ArchiveEntry>,
}

enum PreviewKind {
    Text(String),
    Image {
        size: (u32, u32),
        texture: egui::TextureHandle,
    },
    Binary(usize),
    Empty,
}

impl SourceVaultApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "sourcevault_settings"))
            .unwrap_or_default();
        let lang = match settings.lang.as_deref() {
            Some(code) => Lang::ALL
                .iter()
                .copied()
                .find(|l| l.0 == code)
                .unwrap_or_else(i18n::detect),
            None => i18n::detect(),
        };
        // Slightly larger default fonts for legibility.
        let mut style = (*cc.egui_ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size *= 1.1;
        }
        cc.egui_ctx.set_style(style);

        Self {
            settings,
            lang,
            archive: None,
            selected_dir: None,
            selected_entry: None,
            expanded: BTreeSet::new(),
            error: None,
            show_about: false,
            preview_cache: None,
        }
    }

    fn t(&self, key: &str) -> String {
        i18n::t(self.lang, key)
    }

    fn open_archive(&mut self, path: PathBuf) {
        self.preview_cache = None;
        self.selected_dir = None;
        self.selected_entry = None;
        self.expanded.clear();
        match formats::open(&path) {
            Ok(inner) => {
                let entries = inner.entries();
                let tree = build_tree(entries);
                let total_size = entries.iter().map(|e| e.size).sum();
                self.archive = Some(LoadedArchive {
                    path: path.clone(),
                    inner,
                    tree,
                    total_size,
                });
                self.settings.last_open_dir = path.parent().map(|p| p.to_path_buf());
                self.error = None;
            }
            Err(err) => {
                self.archive = None;
                self.error = Some(format!("{err}"));
            }
        }
    }

    fn close_archive(&mut self) {
        self.archive = None;
        self.preview_cache = None;
        self.selected_dir = None;
        self.selected_entry = None;
        self.expanded.clear();
    }

    fn extract_all_dialog(&mut self) {
        let Some(archive) = self.archive.as_mut() else {
            return;
        };
        let start = self
            .settings
            .last_extract_dir
            .clone()
            .or_else(|| archive.path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        if let Some(dir) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, "dialog.choose_dest"))
            .set_directory(start)
            .pick_folder()
        {
            self.settings.last_extract_dir = Some(dir.clone());
            match archive.inner.extract_all(&dir, None) {
                Ok(n) => log::info!("extracted {n} files to {}", dir.display()),
                Err(e) => self.error = Some(format!("{e}")),
            }
        }
    }

    fn extract_selection_dialog(&mut self) {
        let Some(entry_path) = self.selected_entry.clone() else {
            return;
        };
        let Some(archive) = self.archive.as_mut() else {
            return;
        };
        let start = self
            .settings
            .last_extract_dir
            .clone()
            .or_else(|| archive.path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let suggested = entry_path
            .rsplit('/')
            .next()
            .unwrap_or(&entry_path)
            .to_string();
        if let Some(dest) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, "dialog.choose_dest"))
            .set_directory(start)
            .set_file_name(&suggested)
            .save_file()
        {
            match archive.inner.extract_entry(&entry_path, &dest) {
                Ok(n) => log::info!("extracted {n} bytes to {}", dest.display()),
                Err(e) => self.error = Some(format!("{e}")),
            }
        }
    }

    fn refresh_preview(&mut self, ctx: &egui::Context) {
        let Some(path) = self.selected_entry.clone() else {
            self.preview_cache = None;
            return;
        };
        if matches!(&self.preview_cache, Some((p, _)) if p == &path) {
            return;
        }
        let Some(archive) = self.archive.as_mut() else {
            return;
        };
        match archive.inner.read_entry(&path) {
            Ok(bytes) if bytes.is_empty() => {
                self.preview_cache = Some((path, PreviewKind::Empty));
            }
            Ok(bytes) => {
                // Try image first.
                if path.ends_with(".png")
                    || path.ends_with(".jpg")
                    || path.ends_with(".jpeg")
                    || path.ends_with(".bmp")
                    || path.ends_with(".tga")
                {
                    match image::load_from_memory(&bytes) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let size = (rgba.width(), rgba.height());
                            let color = egui::ColorImage::from_rgba_unmultiplied(
                                [size.0 as usize, size.1 as usize],
                                rgba.as_raw(),
                            );
                            let tex = ctx.load_texture("preview", color, Default::default());
                            self.preview_cache =
                                Some((path, PreviewKind::Image { size, texture: tex }));
                            return;
                        }
                        Err(_) => { /* fall through to text/binary */ }
                    }
                }
                // Text heuristic: mostly ASCII, no null bytes.
                let printable = bytes
                    .iter()
                    .take(4096)
                    .filter(|&&b| b == b'\n' || b == b'\r' || b == b'\t' || (32..127).contains(&b))
                    .count();
                let total = bytes.len().min(4096);
                if total > 0 && printable * 10 / total >= 9 && !bytes.contains(&0) {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    let trimmed = if text.len() > 64 * 1024 {
                        let mut s = text[..64 * 1024].to_owned();
                        s.push_str("\n\n[truncated]");
                        s
                    } else {
                        text
                    };
                    self.preview_cache = Some((path, PreviewKind::Text(trimmed)));
                } else {
                    self.preview_cache = Some((path, PreviewKind::Binary(bytes.len())));
                }
            }
            Err(e) => {
                self.error = Some(format!("{e}"));
                self.preview_cache = None;
            }
        }
    }

    fn ui_menu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(self.t("menu.file"), |ui| {
                if ui.button(self.t("menu.file.open")).clicked() {
                    self.pick_archive();
                    ui.close_menu();
                }
                let has = self.archive.is_some();
                if ui
                    .add_enabled(has, egui::Button::new(self.t("menu.file.close")))
                    .clicked()
                {
                    self.close_archive();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(self.t("menu.file.exit")).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button(self.t("menu.extract"), |ui| {
                let has = self.archive.is_some();
                if ui
                    .add_enabled(has, egui::Button::new(self.t("menu.extract.all")))
                    .clicked()
                {
                    self.extract_all_dialog();
                    ui.close_menu();
                }
                let has_sel = self.selected_entry.is_some();
                if ui
                    .add_enabled(has_sel, egui::Button::new(self.t("menu.extract.selection")))
                    .clicked()
                {
                    self.extract_selection_dialog();
                    ui.close_menu();
                }
            });
            ui.menu_button(self.t("menu.language"), |ui| {
                for lang in Lang::ALL {
                    let selected = lang.0 == self.lang.0;
                    let label = format!("{}{}", if selected { "• " } else { "  " }, lang.label());
                    if ui.button(label).clicked() {
                        self.lang = *lang;
                        self.settings.lang = Some(lang.0.to_string());
                        ui.close_menu();
                    }
                }
            });
            ui.menu_button(self.t("menu.help"), |ui| {
                if ui.button(self.t("menu.help.about")).clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
                if ui.button(self.t("menu.help.docs")).clicked() {
                    let _ = open_url("https://github.com/hohlov2006362018-arch/SourceVault");
                    ui.close_menu();
                }
            });
        });
    }

    fn pick_archive(&mut self) {
        let start = self
            .settings
            .last_open_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        if let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, "dialog.choose_archive"))
            .add_filter(
                i18n::t(self.lang, "dialog.filter"),
                &["vpk", "pak", "gcf", "sga", "wad", "xzp", "ncf"],
            )
            .set_directory(start)
            .pick_file()
        {
            self.open_archive(path);
        }
    }

    fn ui_tree(&mut self, ui: &mut egui::Ui) {
        if self.archive.is_none() {
            ui.label(self.t("status.no_archive"));
            return;
        }
        ScrollArea::vertical()
            .id_source("tree_scroll")
            .show(ui, |ui| {
                let mut selected_dir = self.selected_dir.clone();
                let mut to_open: Option<String> = None;
                let mut to_close: Option<String> = None;
                if let Some(archive) = &self.archive {
                    draw_tree_node(
                        ui,
                        &archive.tree,
                        "",
                        "",
                        &self.expanded,
                        &mut selected_dir,
                        &mut to_open,
                        &mut to_close,
                    );
                }
                self.selected_dir = selected_dir;
                if let Some(p) = to_open {
                    self.expanded.insert(p);
                }
                if let Some(p) = to_close {
                    self.expanded.remove(&p);
                }
            });
    }

    fn ui_entry_list(&mut self, ui: &mut egui::Ui) {
        let dir = self.selected_dir.clone().unwrap_or_default();
        let selected_entry = self.selected_entry.clone();

        // Snapshot the node so we don't borrow `self.archive` across the closure.
        type ListingSnapshot = (Vec<String>, Vec<(String, ArchiveEntry)>);
        let snapshot: Option<ListingSnapshot> = self
            .archive
            .as_ref()
            .and_then(|a| lookup_node(&a.tree, &dir))
            .map(|node| {
                (
                    node.children.keys().cloned().collect(),
                    node.files
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                )
            });
        let Some((sub_dirs, files)) = snapshot else {
            return;
        };

        let label_name = self.t("column.name");
        let label_size = self.t("column.size");
        let label_crc = self.t("column.crc");

        let mut to_select_dir: Option<String> = None;
        let mut to_select_entry: Option<String> = None;

        ScrollArea::vertical()
            .id_source("entries_scroll")
            .show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .column(egui_extras::Column::auto().at_least(220.0))
                    .column(egui_extras::Column::auto().at_least(80.0))
                    .column(egui_extras::Column::auto().at_least(90.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong(&label_name);
                        });
                        header.col(|ui| {
                            ui.strong(&label_size);
                        });
                        header.col(|ui| {
                            ui.strong(&label_crc);
                        });
                    })
                    .body(|mut body| {
                        for child_name in &sub_dirs {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button(format!("📁 {child_name}")).clicked() {
                                        let new_dir = if dir.is_empty() {
                                            child_name.clone()
                                        } else {
                                            format!("{dir}/{child_name}")
                                        };
                                        to_select_dir = Some(new_dir);
                                    }
                                });
                                row.col(|_| {});
                                row.col(|_| {});
                            });
                        }
                        for (name, entry) in &files {
                            let selected = matches!(&selected_entry, Some(p) if p == &entry.path);
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    let label =
                                        RichText::new(format!("📄 {name}")).color(if selected {
                                            Color32::LIGHT_BLUE
                                        } else {
                                            ui.visuals().text_color()
                                        });
                                    if ui.selectable_label(selected, label).clicked() {
                                        to_select_entry = Some(entry.path.clone());
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(format_bytes(entry.size));
                                });
                                row.col(|ui| {
                                    if let Some(c) = entry.crc32 {
                                        ui.label(format!("{c:08X}"));
                                    } else {
                                        ui.label("—");
                                    }
                                });
                            });
                        }
                    });
            });

        if let Some(d) = to_select_dir {
            self.selected_dir = Some(d.clone());
            self.expanded.insert(d);
        }
        if let Some(e) = to_select_entry {
            self.selected_entry = Some(e);
        }
    }

    fn ui_preview(&mut self, ui: &mut egui::Ui) {
        let Some((_, kind)) = &self.preview_cache else {
            ui.label(self.t("status.no_archive"));
            return;
        };
        match kind {
            PreviewKind::Empty => {
                ui.label(self.t("preview.empty"));
            }
            PreviewKind::Text(text) => {
                ScrollArea::both()
                    .id_source("text_preview_scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut text.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .code_editor(),
                        );
                    });
            }
            PreviewKind::Image { size, texture } => {
                let label = self
                    .t("preview.image")
                    .replace("{w}", &size.0.to_string())
                    .replace("{h}", &size.1.to_string());
                ui.label(label);
                ScrollArea::both()
                    .id_source("image_preview_scroll")
                    .show(ui, |ui| {
                        ui.image(texture);
                    });
            }
            PreviewKind::Binary(bytes) => {
                let label = self
                    .t("preview.binary")
                    .replace("{bytes}", &bytes.to_string());
                ui.label(label);
            }
        }
    }
}

impl eframe::App for SourceVaultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file drops.
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(file) = dropped.iter().find_map(|f| f.path.clone()) {
            self.open_archive(file);
        }

        self.refresh_preview(ctx);

        TopBottomPanel::top("menu_bar").show(ctx, |ui| self.ui_menu(ctx, ui));

        TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| match &self.archive {
                Some(a) => {
                    let label =
                        i18n::t(self.lang, "status.format").replace("{fmt}", a.inner.format_name());
                    ui.label(label);
                    ui.separator();
                    let label = i18n::t(self.lang, "status.entries")
                        .replace("{count}", &a.inner.entries().len().to_string())
                        .replace("{bytes}", &format_bytes(a.total_size));
                    ui.label(label);
                }
                None => {
                    ui.label(i18n::t(self.lang, "status.no_archive"));
                }
            });
        });

        egui::SidePanel::left("tree_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading(self.t("panel.tree"));
                self.ui_tree(ui);
            });

        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.heading(self.t("panel.preview"));
                self.ui_preview(ui);
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.t("panel.entries"));
            self.ui_entry_list(ui);
        });

        // About dialog.
        if self.show_about {
            let title = self.t("menu.help.about");
            let body = self.t("about.body");
            let ver = self.t("about.version").replace("{ver}", VERSION);
            let mut open = true;
            egui::Window::new(title).open(&mut open).show(ctx, |ui| {
                ui.label(ver);
                ui.separator();
                ui.label(body);
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("OK").clicked() {
                        self.show_about = false;
                    }
                });
            });
            if !open {
                self.show_about = false;
            }
        }

        // Error toast.
        if let Some(err) = self.error.clone() {
            let title = self.t("error.title");
            let mut open = true;
            egui::Window::new(title).open(&mut open).show(ctx, |ui| {
                ui.colored_label(Color32::LIGHT_RED, err);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("OK").clicked() {
                        self.error = None;
                    }
                });
            });
            if !open {
                self.error = None;
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "sourcevault_settings", &self.settings);
    }
}

fn build_tree(entries: &[ArchiveEntry]) -> TreeNode {
    let mut root = TreeNode::default();
    for e in entries {
        if e.kind != EntryKind::File {
            continue;
        }
        let parts: Vec<&str> = e.path.split('/').collect();
        let (file_name, dir_parts) = match parts.split_last() {
            Some(x) => x,
            None => continue,
        };
        let mut cur = &mut root;
        for p in dir_parts {
            cur = cur
                .children
                .entry(p.to_string())
                .or_insert_with(TreeNode::default);
        }
        cur.files.insert(file_name.to_string(), e.clone());
    }
    root
}

fn lookup_node<'a>(root: &'a TreeNode, path: &str) -> Option<&'a TreeNode> {
    if path.is_empty() {
        return Some(root);
    }
    let mut cur = root;
    for p in path.split('/') {
        cur = cur.children.get(p)?;
    }
    Some(cur)
}

#[allow(clippy::too_many_arguments)]
fn draw_tree_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    full_path: &str,
    name: &str,
    expanded: &BTreeSet<String>,
    selected: &mut Option<String>,
    to_open: &mut Option<String>,
    to_close: &mut Option<String>,
) {
    let displayed = if name.is_empty() { "/" } else { name };
    let is_expanded = expanded.contains(full_path);
    ui.horizontal(|ui| {
        let icon = if node.children.is_empty() {
            ""
        } else if is_expanded {
            "▼ "
        } else {
            "▶ "
        };
        let sel = matches!(selected, Some(s) if s == full_path);
        if ui
            .selectable_label(sel, format!("{icon}{displayed}"))
            .clicked()
        {
            *selected = Some(full_path.to_string());
            if !node.children.is_empty() {
                if is_expanded {
                    *to_close = Some(full_path.to_string());
                } else {
                    *to_open = Some(full_path.to_string());
                }
            }
        }
    });
    if is_expanded {
        ui.indent(full_path, |ui| {
            for (child_name, child) in &node.children {
                let child_path = if full_path.is_empty() {
                    child_name.clone()
                } else {
                    format!("{full_path}/{child_name}")
                };
                draw_tree_node(
                    ui,
                    child,
                    &child_path,
                    child_name,
                    expanded,
                    selected,
                    to_open,
                    to_close,
                );
            }
        });
    }
}

fn format_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

#[cfg(target_os = "windows")]
fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "linux")]
fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
}
