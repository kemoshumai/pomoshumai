fn main() {
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("assets/windows/app.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed Windows resources");
    }
}
