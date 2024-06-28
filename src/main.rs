use std::cell::Cell;
use gtk4 as gtk;
use gtk4::{Application, ApplicationWindow, Button, CssProvider, Entry, glib, Image, Orientation, ScrolledWindow, Box, ScrollablePolicy, PolicyType, SignalListItemFactory, ListItem, Frame};
use gtk4::gdk::Display;
use gtk4::glib::Properties;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, Cast, EditableExt, EntryExt, GtkWindowExt, ListItemExt, ObjectExt, WidgetExt};

const APP_ID: &str = "com.lewdneko.poligen";

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id(APP_ID)
        .build();

    application.connect_startup(|_| load_css());
    application.connect_activate(build_ui);

    application.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let input = Entry::builder()
        .placeholder_text("Prompt")
        .build();

    let button = Button::with_label("Generate");

    let top_box = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["top-box"])
        .build();

    top_box.append(&input);
    top_box.append(&button);
    
    let top_box_frame = Frame::builder()
        .child(&top_box)
        .css_classes(["top-box-frame"])
        .build();

    let image = Image::builder()
        .file("test.jpg")
        .hexpand(true)
        .vexpand(true)
        .build();
    
    button.connect_clicked(move |_| {
        let prompt = input.text();
    });
    
    let image_frame = Frame::builder()
        .child(&image)
        .hexpand(true)
        .vexpand(true)
        .css_classes(["image-frame"])
        .build();

    let image_list_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .height_request(100)
        .homogeneous(true)
        .css_classes(["image-list-box"])
        .build();

    for _ in 0..20 {
        let image = Image::builder()
            .file("test.jpg")
            .height_request(100)
            .width_request(100)
            .css_classes(["image-preview"])
            .build();
        
        let frame = Frame::builder()
            .child(&image)
            .css_classes(["image-preview-frame"])
            .build();

        image_list_box.append(&frame);
    }

    let image_list = ScrolledWindow::builder()
        .vscrollbar_policy(PolicyType::Never)
        .hscrollbar_policy(PolicyType::Automatic)
        .kinetic_scrolling(true)
        .child(&image_list_box)
        .css_classes(["image-list"])
        .build();
    
    let image_list_frame = Frame::builder()
        .child(&image_list)
        .css_classes(["image-list-frame"])
        .build();

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["main-box"])
        .build();

    main_box.append(&top_box_frame);
    main_box.append(&image_frame);
    main_box.append(&image_list_frame);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Polli-Gen Image Generator")
        .default_width(520)
        .default_height(846)
        .child(&main_box)
        .build();

    window.present();
}