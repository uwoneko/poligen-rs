use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;
use gtk4::{AlertDialog, Align, Application, ApplicationWindow, BoxLayout, Button, CheckButton, Entry, Frame, glib, Grid, GridLayout, Image, Orientation, PolicyType, ScrolledWindow};
use gtk4::glib::clone;
use gtk4::prelude::{BoxExt, ButtonExt, Cast, CellLayoutExt, CheckButtonExt, EditableExt, GridExt, GtkWindowExt, LayoutManagerExt, WidgetExt};
use gtk4::Box;
use poligen_rs::{generate_image, save_image};
use crate::runtime;

const ASPECT_PRESETS: [(&str, [i32; 2]); 3] = [
    ("1:1", [1024, 1024]),
    ("3:4", [768, 1024]),
    ("16:9", [1024, 576]),
];

pub fn build_ui(app: &Application) {
    let input = Entry::builder()
        .placeholder_text("Prompt")
        .build();

    let aspect_ratio_grid = Grid::builder()
        .column_homogeneous(false)
        .build();
    
    let aspect_ratio_choice = Rc::new(Cell::new([0, 0]));
    let mut aspect_ratio_checks = Vec::with_capacity(ASPECT_PRESETS.len());
    
    {
        let mut first_checkbox = None;

        for (i, (aspect_ratio, resolution)) in ASPECT_PRESETS.into_iter().enumerate() {
            let aspect_ratio_check = CheckButton::builder()
                .halign(Align::Center)
                .hexpand(true)
                .label(aspect_ratio)
                .build();

            aspect_ratio_check.connect_toggled(clone!(@strong aspect_ratio_choice => move |ratio_input| {
                if ratio_input.is_active() {
                    aspect_ratio_choice.set(resolution);
                    dbg!(&aspect_ratio_choice);
                }
            }));

            match &first_checkbox {
                None => {
                    aspect_ratio_check.set_active(true);
                    first_checkbox = Some(aspect_ratio_check.clone());
                }
                Some(first_checkbox) => {
                    aspect_ratio_check.set_group(Some(first_checkbox));
                }
            }

            aspect_ratio_grid.attach(&aspect_ratio_check, i as i32, 0, 1, 1);
            aspect_ratio_checks.push(aspect_ratio_check);
        }
    }
    
    let custom_aspect_ratio_input_x = Entry::builder()
        .placeholder_text("Width")
        .text("1024")
        .hexpand(true)
        .build();
    
    let custom_aspect_ratio_input_y = Entry::builder()
        .placeholder_text("Height")
        .text("1024")
        .hexpand(true)
        .build();
    
    let custom_aspect_ratio_box = Box::builder()
        .hexpand(true)
        .sensitive(false)
        .opacity(0.0)
        .css_classes(["custom-aspect-ratio-box"])
        .build();
    
    custom_aspect_ratio_box.append(&custom_aspect_ratio_input_x);
    custom_aspect_ratio_box.append(&custom_aspect_ratio_input_y);
    
    aspect_ratio_grid.attach(&custom_aspect_ratio_box, 0, 0, ASPECT_PRESETS.len() as i32, 1);

    let custom_aspect_ratio_check = CheckButton::builder()
        .halign(Align::Center)
        .hexpand(true)
        .label("Custom")
        .build();
    
    custom_aspect_ratio_check.connect_toggled(clone!(@weak custom_aspect_ratio_box => move |check| {
        let active = check.is_active();
        
        custom_aspect_ratio_box.set_opacity(if active { 1.0 } else { 0.0 });
        custom_aspect_ratio_box.set_sensitive(active);
        for check in &aspect_ratio_checks {
            check.set_visible(!active);
        }
    }));

    aspect_ratio_grid.attach(&custom_aspect_ratio_check, ASPECT_PRESETS.len() as i32, 0, 1, 1);

    let generate_button = Button::with_label("Generate");

    let top_box = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["top-box"])
        .build();

    top_box.append(&input);
    top_box.append(&aspect_ratio_grid);
    top_box.append(&generate_button);

    let top_box_frame = Frame::builder()
        .child(&top_box)
        .css_classes(["top-box-frame"])
        .build();

    let image = Image::builder()
        .file("test.jpg")
        .hexpand(true)
        .vexpand(true)
        .build();

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

    let (sender, receiver) = async_channel::unbounded();

    generate_button.connect_clicked(clone!(@weak input => move |_| {
        let prompt = input.text();
        eprintln!("{prompt}");

        runtime().spawn(clone!(@strong sender => async move {
            let generate_result = generate_image(
                prompt
            ).await;

            sender.send(generate_result).await.expect("channel has to be open");
        }));
    }));

    glib::spawn_future_local(clone!(@weak image, @weak window => async move {
        while let Ok(response) = receiver.recv().await {
            let bytes = match response {
                Ok(bytes) => bytes,
                Err(err) => {
                    let alert = AlertDialog::builder()
                        .buttons(["OK"])
                        .message("Image generation error")
                        .detail(format!("{}", err))
                        .modal(true)
                        .build();

                    alert.show(Some(&window));

                    continue;
                }
            };

            let file_path = match save_image(bytes, Path::new("outputs/"), "jpg").await {
                Ok(path) => path,
                Err(err) => {
                    let alert = AlertDialog::builder()
                        .buttons(["OK"])
                        .message("File saving error")
                        .detail(format!("{}", err))
                        .modal(true)
                        .build();

                    alert.show(Some(&window));

                    continue;
                }
            };

            image.set_file(Some(file_path.canonicalize().unwrap().to_str().unwrap()));
        }
    }));

    window.present();
}
