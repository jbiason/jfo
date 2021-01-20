/*
   JFO - Joplin Folder Organizer
   Copyright (C) 2020  Julio Biason

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::collections::HashMap;

use reqwest::blocking;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct FolderList {
    items: Vec<Folder>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct Folder {
    id: String,
    title: String,
    children: Option<Vec<Folder>>,
    parent_id: String,
}

#[derive(Debug, Deserialize)]
struct NoteList {
    items: Vec<Note>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct Note {
    id: String,
    title: String,
    parent_id: String,
}

#[derive(Debug, Deserialize)]
struct NoteTagList {
    items: Vec<NoteTag>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct NoteTag {
    id: String,
    title: String,
}

fn main() {
    match (
        std::env::args().nth(1),
        std::env::args().nth(2),
        std::env::args().nth(3),
    ) {
        (Some(folder), Some(token), Some(port)) => process(folder, token, port),
        (Some(folder), Some(token), None) => process(folder, token, "41184".to_string()),
        (_, _, _) => usage(),
    }
}

fn usage() {
    println!("Usage: jfo <inbox> <token> [port]\n");
    println!("<inbox>: Base notebook name where the posts will be checked;");
    println!("<token>: Token provided by the webclipper to access the notes;");
    println!("[port]: Web clipper port, defaults to 41184");
}

fn process(inbox_name: String, token: String, port: String) {
    let mut folder_list = dbg!(folders(&url(&port, &token, &"folders")));
    folder_list.sort_by(|a, b| a.title.cmp(&b.title));
    let inbox_pos = dbg!(folder_list
        .binary_search_by(|folder| folder.title.cmp(&inbox_name))
        .unwrap());
    let inbox_id = folder_list[inbox_pos].id.to_string();

    let mut children: HashMap<String, String> = HashMap::new();
    for folder in folder_list.iter() {
        if folder.parent_id == inbox_id {
            children.insert(folder.title.to_string(), folder.id.to_string());
        }
    }
    dbg!(&children);

    get_notes(&dbg!(folder_url(&port, &token, &inbox_id)))
        .iter()
        .filter(|note| {
            let tag_url = dbg!(note_tags_url(&port, &token, &note.id));
            let tags = get_note_tags(&tag_url);
            tags.len() > 0
        })
        .for_each(|note| {
            let frags: Vec<&str> = note.title.split('/').collect();
            let (author, id) = dbg!((frags[0], frags[1]));
            let new_folder_id = dbg!(if children.contains_key(author) {
                children.get(author).unwrap().into()
            } else {
                let new_folder_url = url(&port, &token, &"folders");
                let result = create_folder(&new_folder_url, author, &inbox_id);
                result.id
            });

            let url = note_url(&port, &token, &note.id);
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
    let mut result: Vec<Folder> = Vec::new();
    let mut has_more = true;
    let mut page = 1;

    while has_more {
        let paged_url = format!("{base}&page={page}", base = url, page = page);
        let folder_list: FolderList = dbg!(blocking::get(&paged_url).unwrap().json().unwrap());
        has_more = folder_list.has_more;
        result.extend(folder_list.items);
        page += 1;
    }
    result
}

fn get_notes(url: &str) -> Vec<Note> {
    let mut result: Vec<Note> = Vec::new();
    let mut has_more = true;
    let mut page = 1;

    while has_more {
        let paged_url = format!("{base}&page={page}", base = url, page = page);
        let note_list: NoteList = blocking::get(&paged_url).unwrap().json().unwrap();
        has_more = note_list.has_more;
        result.extend(note_list.items);
        page += 1;
    }
    result
}

fn get_note_tags(url: &str) -> Vec<NoteTag> {
    let mut result: Vec<NoteTag> = Vec::new();
    let mut has_more = true;
    let mut page = 1;

    while has_more {
        let paged_url = format!("{base}&page={page}", base = url, page = page);
        let tags: NoteTagList = blocking::get(&paged_url).unwrap().json().unwrap();
        has_more = tags.has_more;
        result.extend(tags.items);
        page += 1;
    }
    result
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
