use std::collections::HashMap;

use clap::crate_authors;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use clap::App;
use clap::Arg;
use reqwest::blocking;
use serde_derive::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Folder {
    id: String,
    title: String,
    children: Option<Vec<Folder>>,
    parent_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Note {
    id: String,
    title: String,
    parent_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NoteTag {
    id: String,
    title: String,
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("inbox")
                .long("inbox")
                .short("i")
                .takes_value(true)
                .required(true)
                .help("Inbox folder name"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .takes_value(true)
                .required(true)
                .default_value("41184")
                .help("Joplin Web Clipper port"),
        )
        .arg(
            Arg::with_name("token")
                .long("token")
                .short("t")
                .takes_value(true)
                .required(true)
                .help("Joplin Web Clipper access token"),
        )
        .get_matches();

    let inbox_name = matches.value_of("inbox").unwrap();
    let port = matches.value_of("port").unwrap();
    let token = matches.value_of("token").unwrap();

    let folder_list = dbg!(folders(&url(port, token, &"folders")));
    let inbox = dbg!(folder_list
        .into_iter()
        .filter(|folder| folder.title == inbox_name)
        .take(1)
        .collect::<Vec<Folder>>()
        .pop()
        .unwrap());
    let inbox_id = inbox.id.to_string();

    let mut children: HashMap<String, String> = HashMap::new();
    for folder in inbox.children.unwrap().iter() {
        children.insert(folder.title.to_string(), folder.id.to_string());
    }
    dbg!(&children);

    get_notes(&dbg!(folder_url(port, token, &inbox.id)))
        .iter()
        .filter(|note| {
            let tag_url = dbg!(note_tags_url(port, token, &note.id));
            let tags = get_note_tags(&tag_url);
            tags.len() > 0
        })
        .for_each(|note| {
            let frags: Vec<&str> = note.title.split('/').collect();
            let (author, id) = dbg!((frags[0], frags[1]));
            let new_folder_id = dbg!(if children.contains_key(author) {
                children.get(author).unwrap().into()
            } else {
                let new_folder_url = url(port, token, &"folders");
                let result = create_folder(&new_folder_url, author, &inbox_id);
                result.id
            });

            let url = note_url(port, token, &note.id);
            move_note_to_folder(&url, &note.id, &new_folder_id, &id);
        });
}

fn url(port: &str, token: &str, resource: &str) -> String {
    format!(
        "http://localhost:{port}/{resource}?token={token}",
        port = port,
        resource = resource,
        token = token
    )
}

fn folder_url(port: &str, token: &str, folder_id: &str) -> String {
    let resource = format!("folders/{folder_id}/notes", folder_id = folder_id);
    url(port, token, &resource)
}

fn note_url(port: &str, token: &str, note_id: &str) -> String {
    let resource = format!("notes/{note_id}", note_id = note_id);
    url(port, token, &resource)
}

fn note_tags_url(port: &str, token: &str, note_id: &str) -> String {
    let resource = dbg!(format!("notes/{note_id}/tags", note_id = note_id));
    url(port, token, &resource)
}

fn folders(url: &str) -> Vec<Folder> {
    let folders: Vec<Folder> = blocking::get(url).unwrap().json().unwrap();
    folders
}

fn get_notes(url: &str) -> Vec<Note> {
    let notes: Vec<Note> = blocking::get(url).unwrap().json().unwrap();
    notes
}

fn get_note_tags(url: &str) -> Vec<NoteTag> {
    let tags: Vec<NoteTag> = blocking::get(url).unwrap().json().unwrap();
    tags
}

fn create_folder(url: &str, title: &str, parent_id: &str) -> Folder {
    let mut request: HashMap<String, String> = HashMap::new();
    request.insert("title".into(), title.into());
    request.insert("parent_id".into(), parent_id.into());
    let folder: Folder = blocking::Client::new()
        .post(url)
        .json(&request)
        .send()
        .unwrap()
        .json()
        .unwrap();
    folder
}

fn move_note_to_folder(url: &str, note_id: &str, new_folder_id: &str, new_title: &str) {
    let mut request: HashMap<String, String> = HashMap::new();
    request.insert("id".into(), note_id.into());
    request.insert("title".into(), new_title.into());
    request.insert("parent_id".into(), new_folder_id.into());
    blocking::Client::new()
        .put(url)
        .json(&request)
        .send()
        .unwrap();
}
