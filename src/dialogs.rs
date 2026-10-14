use std::{cell::Cell, rc::Rc};

use gtk::{gio, prelude::*};
use gtk4 as gtk;

use crate::{desktop, entry};

fn show_legacy_file_chooser(
    parent: &gtk::ApplicationWindow,
    app_container: &gtk::Box,
    target_url: Option<String>,
) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Choose your browser's desktop entry: ")
        .action(gtk::FileChooserAction::Open)
        .build();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Desktop entries"));
    filter.add_pattern("*.desktop");
    dialog.add_filter(&filter);
    dialog.set_filter(&filter);
    let default_path = gio::File::for_path("/usr/share/applications");
    if let Err(e) = dialog.set_current_folder(Some(&default_path)) {
        eprintln!(
            "Error: Failed to read \"usr/share/applications\" folder: {}",
            e
        );
    }
    dialog.add_buttons(&[
        ("Open", gtk::ResponseType::Accept),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);
    let container = app_container.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    match path.file_name().and_then(|n| n.to_str()) {
                        Some(file_name) => {
                            if let Err(e) = desktop::install_candidate(&path, file_name) {
                                eprintln!("Error: Failed to copy desktop entry: {}", e);
                            } else {
                                while let Some(child) = container.first_child() {
                                    container.remove(&child);
                                }
                                entry::populate(&container, target_url.clone());
                            }
                        }
                        None => {
                            eprintln!("Error: Invalid file name for desktop entry");
                        }
                    }
                } else {
                    eprintln!("Error: Could not resolve chosen file path");
                }
            } else {
                eprintln!("Error: No file chosen");
            }
        };
        dialog.destroy()
    });
    dialog.show();
}

pub fn add_entries_dialog(
    parent: &gtk::ApplicationWindow,
    app_container: &gtk::Box,
    target_url: Option<String>,
) {
    let candidates = desktop::scan_browser_candidates();
    if candidates.is_empty() {
        show_legacy_file_chooser(parent, app_container, target_url);
        return;
    }

    let dialog = gtk::Dialog::builder()
        .title("Add browsers")
        .modal(true)
        .build();
    dialog.set_transient_for(Some(parent));
    dialog.set_destroy_with_parent(true);
    dialog.set_default_width(400);
    dialog.set_default_height(400);

    let content = dialog.content_area();
    content.set_spacing(6);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    scrolled.set_child(Some(&list));
    content.append(&scrolled);

    let added_any = Rc::new(Cell::new(false));

    for candidate in candidates {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let image = gtk::Image::builder()
            .icon_name(&candidate.icon)
            .pixel_size(24)
            .build();

        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text_box.set_hexpand(true);

        let name_label = gtk::Label::new(Some(&candidate.name));
        name_label.set_xalign(0.0);
        name_label.set_single_line_mode(true);

        let file_label = gtk::Label::builder()
            .label(&candidate.file_name)
            .css_classes(vec!["dim-label", "caption"])
            .build();
        file_label.set_xalign(0.0);
        file_label.set_single_line_mode(true);

        text_box.append(&name_label);
        text_box.append(&file_label);

        let add_button = gtk::Button::with_label("Add");
        let source_path = candidate.source_path.clone();
        let file_name = candidate.file_name.clone();
        let added_flag = added_any.clone();
        add_button.connect_clicked(move |button| {
            match desktop::install_candidate(&source_path, &file_name) {
                Ok(_) => {
                    println!("Added desktop entry: {}", file_name);
                    button.set_label("Added");
                    button.set_sensitive(false);
                    added_flag.set(true);
                }
                Err(e) => {
                    eprintln!("Error: Failed to copy desktop entry: {}", e);
                }
            }
        });

        row_box.append(&image);
        row_box.append(&text_box);
        row_box.append(&add_button);

        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&row_box));
        list.append(&row);
    }

    dialog.add_buttons(&[("Done", gtk::ResponseType::Close)]);

    let container = app_container.clone();
    let added_flag = added_any.clone();
    dialog.connect_response(move |dialog, _| {
        dialog.destroy();
        // Single reload on close, only when something was actually added.
        if added_flag.get() {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }
            entry::populate(&container, target_url.clone());
        }
    });
    dialog.show();
}
