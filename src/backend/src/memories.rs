use crate::memory::*;
use crate::types::*;

// Image memory functions
#[ic_cdk::update]
pub fn create_image_memory(
    name: String,
    data: Vec<u8>,
    caption: Option<String>,
    title: Option<String>,
    description: Option<String>,
    is_public: bool,
    metadata: ImageMetadata,
) -> String {
    memory::create_image_memory(name, data, caption, title, description, is_public, metadata)
}

#[ic_cdk::query]
pub fn get_image_memory(id: String) -> Option<ImageMemory> {
    memory::get_image_memory(id)
}

#[ic_cdk::query]
pub fn list_user_images(owner_id: String) -> Vec<ImageMemory> {
    memory::list_user_images(owner_id)
}

#[ic_cdk::query]
pub fn list_public_images() -> Vec<ImageMemory> {
    memory::list_public_images()
}

// Note memory functions
#[ic_cdk::update]
pub fn create_note_memory(
    title: String,
    content: String,
    is_public: bool,
    metadata: NoteMetadata,
) -> String {
    memory::create_note_memory(title, content, is_public, metadata)
}

#[ic_cdk::query]
pub fn get_note_memory(id: String) -> Option<NoteMemory> {
    memory::get_note_memory(id)
}

#[ic_cdk::query]
pub fn list_user_notes(owner_id: String) -> Vec<NoteMemory> {
    memory::list_user_notes(owner_id)
}

// User management functions
#[ic_cdk::query]
pub fn whoami() -> String {
    msg_caller().to_string()
}

#[ic_cdk::query]
pub fn get_user_memories(owner_id: String) -> HashMap<String, Vec<String>> {
    memory::get_user_memories(owner_id)
}
