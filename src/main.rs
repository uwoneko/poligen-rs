mod ui;

use std::cell::Cell;
use std::sync::OnceLock;
use gtk4 as gtk;
use gtk4::{Application, ApplicationWindow, Button, CssProvider, Entry, glib, Image, Orientation, ScrolledWindow, Box, ScrollablePolicy, PolicyType, SignalListItemFactory, ListItem, Frame};
use gtk4::gdk::Display;
use gtk4::glib::Properties;
use gtk4::glib::translate::ToGlibPtr;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, Cast, EditableExt, EntryExt, GtkWindowExt, ListItemExt, ObjectExt, ToSendValue, WidgetExt};
use tokio::runtime::Runtime;
use crate::ui::build_ui;

const APP_ID: &str = "com.lewdneko.poligen";

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Setting up tokio runtime needs to succeed.")
    })
}

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
