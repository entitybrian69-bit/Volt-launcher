fn list_punchy(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "punchy".into(),
        title: crate::game::punchy::TITLE.into(),
        description: "Self-contained Rust animation preview. Animation data included.".into(),
        keys: vec![
            [
                "inspect".into(),
                "Inspect item".into(),
                "key.keyboard.f6".into(),
            ],
            [
                "reload".into(),
                "Reload animations".into(),
                "key.keyboard.f7".into(),
            ],
        ],
    });
}
