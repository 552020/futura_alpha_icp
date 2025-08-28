use crate::types::*;
use candid::Principal;
use ic_cdk::api::msg_caller;
use std::collections::HashMap;

// Memory storage
thread_local! {
    static IMAGES: std::cell::RefCell<HashMap<String, ImageMemory>> = std::cell::RefCell::new(HashMap::new());
    static VIDEOS: std::cell::RefCell<HashMap<String, VideoMemory>> = std::cell::RefCell::new(HashMap::new());
    static NOTES: std::cell::RefCell<HashMap<String, NoteMemory>> = std::cell::RefCell::new(HashMap::new());
    static DOCUMENTS: std::cell::RefCell<HashMap<String, DocumentMemory>> = std::cell::RefCell::new(HashMap::new());
    static AUDIO: std::cell::RefCell<HashMap<String, AudioMemory>> = std::cell::RefCell::new(HashMap::new());

    // User storage for Internet Identity integration
    static USERS: std::cell::RefCell<HashMap<Principal, User>> = std::cell::RefCell::new(HashMap::new());
}

// Helper function to generate secure codes
pub fn generate_secure_code() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    ic_cdk::api::time().hash(&mut hasher);
    msg_caller().hash(&mut hasher);

    format!("{:x}", hasher.finish())
}

// Core memory operations
pub fn create_image_memory(
    name: String,
    data: Vec<u8>,
    caption: Option<String>,
    title: Option<String>,
    description: Option<String>,
    is_public: bool,
    metadata: ImageMetadata,
) -> String {
    let owner_id = msg_caller().to_string();
    let id = format!("img_{}", ic_cdk::api::time());
    let owner_secure_code = generate_secure_code();

    let image = ImageMemory {
        id: id.clone(),
        owner_id,
        name,
        data,
        caption,
        title,
        description,
        is_public,
        owner_secure_code,
        parent_folder_id: None,
        metadata,
        created_at: ic_cdk::api::time().to_string(),
    };

    IMAGES.with(|images| {
        images.borrow_mut().insert(id.clone(), image);
    });

    id
}

pub fn get_image_memory(id: String) -> Option<ImageMemory> {
    IMAGES.with(|images| images.borrow().get(&id).cloned())
}

pub fn list_user_images(owner_id: String) -> Vec<ImageMemory> {
    IMAGES.with(|images| {
        images
            .borrow()
            .values()
            .filter(|img| img.owner_id == owner_id)
            .cloned()
            .collect()
    })
}

pub fn list_public_images() -> Vec<ImageMemory> {
    IMAGES.with(|images| {
        images
            .borrow()
            .values()
            .filter(|img| img.is_public)
            .cloned()
            .collect()
    })
}

pub fn create_note_memory(
    title: String,
    content: String,
    is_public: bool,
    metadata: NoteMetadata,
) -> String {
    let owner_id = msg_caller().to_string();
    let id = format!("note_{}", ic_cdk::api::time());
    let owner_secure_code = generate_secure_code();

    let note = NoteMemory {
        id: id.clone(),
        owner_id,
        title,
        content,
        is_public,
        owner_secure_code,
        parent_folder_id: None,
        metadata,
        created_at: ic_cdk::api::time().to_string(),
        updated_at: ic_cdk::api::time().to_string(),
    };

    NOTES.with(|notes| {
        notes.borrow_mut().insert(id.clone(), note);
    });

    id
}

pub fn get_note_memory(id: String) -> Option<NoteMemory> {
    NOTES.with(|notes| notes.borrow().get(&id).cloned())
}

pub fn list_user_notes(owner_id: String) -> Vec<NoteMemory> {
    NOTES.with(|notes| {
        notes
            .borrow()
            .values()
            .filter(|note| note.owner_id == owner_id)
            .cloned()
            .collect()
    })
}

// User management functions
pub fn get_user_memories(owner_id: String) -> HashMap<String, Vec<String>> {
    let mut memories = HashMap::new();

    // Get image IDs
    let image_ids: Vec<String> = IMAGES.with(|images| {
        images
            .borrow()
            .values()
            .filter(|img| img.owner_id == owner_id)
            .map(|img| img.id.clone())
            .collect()
    });
    memories.insert("images".to_string(), image_ids);

    // Get note IDs
    let note_ids: Vec<String> = NOTES.with(|notes| {
        notes
            .borrow()
            .values()
            .filter(|note| note.owner_id == owner_id)
            .map(|note| note.id.clone())
            .collect()
    });
    memories.insert("notes".to_string(), note_ids);

    memories
}

// HTTP serving functions
pub fn load_memory(url: &str) -> Option<(Vec<u8>, String)> {
    use regex::Regex;
    use std::borrow::Cow;

    // Match patterns like /memory/img_123, /memory/note_456, etc.
    let re = Regex::new(r"^/memory/([a-z]+)_([a-zA-Z0-9_]+)$").unwrap();

    if let Some(captures) = re.captures(url) {
        let memory_type = captures.get(1).unwrap().as_str();
        let memory_id = captures.get(2).unwrap().as_str();

        match memory_type {
            "img" => {
                if let Some(image) = IMAGES.with(|images| images.borrow().get(memory_id).cloned()) {
                    return Some((image.data, image.metadata.common.mime_type.clone()));
                }
            }
            "note" => {
                if let Some(note) = NOTES.with(|notes| notes.borrow().get(memory_id).cloned()) {
                    // Convert note content to bytes
                    let content_bytes = note.content.as_bytes().to_vec();
                    return Some((content_bytes, "text/plain".to_string()));
                }
            }
            _ => {}
        }
    }

    None
}

pub fn create_memory_response(
    memory_data: Vec<u8>,
    content_type: String,
) -> ic_http_certification::HttpResponse<'static> {
    use std::borrow::Cow;

    ic_http_certification::HttpResponse::ok(
        Cow::Owned(memory_data),
        vec![
            ("Content-Type".to_string(), content_type),
            (
                "Cache-Control".to_string(),
                "public, max-age=31536000, immutable".to_string(),
            ),
        ],
    )
    .build()
}

pub fn create_not_found_response() -> ic_http_certification::HttpResponse<'static> {
    use std::borrow::Cow;

    ic_http_certification::HttpResponse::not_found(
        Cow::Borrowed(b"Memory not found" as &[u8]),
        vec![("Content-Type".to_string(), "text/plain".to_string())],
    )
    .build()
}

// User storage functions
pub fn with_users<R>(f: impl FnOnce(&mut HashMap<Principal, User>) -> R) -> R {
    USERS.with(|users| f(&mut users.borrow_mut()))
}

pub fn with_users_read<R>(f: impl FnOnce(&HashMap<Principal, User>) -> R) -> R {
    USERS.with(|users| f(&users.borrow()))
}
