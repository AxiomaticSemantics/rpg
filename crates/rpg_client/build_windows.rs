#[cfg(target_os = "windows")]
extern crate embed_resource;

#[cfg(target_os = "windows")]
fn main() {
    // on windows we will set our game icon as icon for the executable
    embed_resource::compile("build/windows/icon.rc");
}
