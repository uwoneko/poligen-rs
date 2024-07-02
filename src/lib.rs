use std::fs;
use std::path::{Path, PathBuf};
use gtk4::prelude::{FileExt, IOStreamExt, OutputStreamExtManual, ToSendValue};
use lazy_static::lazy_static;
use reqwest::{Method, Request, RequestBuilder, Url};
use bytes::{Buf, BufMut, Bytes};
use anyhow::{anyhow, Result};
use gtk4::gio;
use gtk4::gio::FileCreateFlags;
use gtk4::glib::Priority;

lazy_static! {
    // i mean its gonna persist thru the whole app lifetime why not use lazy static
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn generate_image(
    prompt: impl Into<String>,
    resolution: [u32; 2]
) -> Result<Bytes> {
    let url = format!(
        "https://image.pollinations.ai/prompt/{}",
        prompt.into()
    );
    let params = [
        ("width", resolution[0].to_string()),
        ("height", resolution[1].to_string()),
        ("nologo", false.to_string())
    ];

    let url = Url::parse_with_params(&url, params)?;

    let request = REQWEST_CLIENT
        .get(url);

    let image = request.send()
        .await?
        .bytes()
        .await?;

    Ok(image)
}

pub async fn save_image(bytes: Bytes, output_directory: &Path, extension: &str) -> Result<PathBuf> {
    fs::create_dir_all(output_directory)?;
    
    let file_names: Vec<String> = fs::read_dir(output_directory)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.path().file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    
    dbg!(&file_names);
    
    let mut i = 0;
    let file_name = loop {
        i += 1;
        let file_name = format!("output_{}", i);
        if file_names.contains(&file_name) {
            continue;
        }
        
        break file_name;
    };
    
    let path = output_directory.join(format!("{}.{}", file_name, extension));

    let file = gio::File::for_path(&path);
    let stream = file.create_readwrite_future(FileCreateFlags::NONE, Priority::DEFAULT).await?;
    stream.output_stream().write_all_future(bytes, Priority::DEFAULT).await.map_err(|(buf, err)| err)?;

    Ok(path)
}