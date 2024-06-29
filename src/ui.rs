use std::path::Path;
use gtk4::{AlertDialog, Application, ApplicationWindow, Button, Entry, Frame, glib, Image, Orientation, PolicyType, ScrolledWindow};
use gtk4::glib::clone;
use gtk4::prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt};
use poligen_rs::{generate_image, save_image};
use crate::runtime;

pub fn build_ui(app: &Application) {
    let input = Entry::builder()
        .placeholder_text("Prompt")
        .build();

    let generate_button = Button::with_label("Generate");

    let top_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["top-box"])
        .build();

    top_box.append(&input);
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

    let image_list_box = gtk4::Box::builder()
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

    let main_box = gtk4::Box::builder()
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
    
    let (sender, receiver) = async_channel::bounded(1);

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
