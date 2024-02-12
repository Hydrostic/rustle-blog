use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "templates/email"]
pub struct MailTemplates;

use fluent_templates::static_loader;

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
        // customise: |bundle| bundle.set_use_isolating(false),
    };
}
