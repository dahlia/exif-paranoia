use std::{fs::read_to_string, path::Path};

use fluent_bundle::{FluentBundle, FluentResource};
use fluent_langneg::{
    convert_vec_str_to_langids_lossy, negotiate_languages, NegotiationStrategy,
};
use itertools::Itertools;
use tracing::debug;
use unic_langid::{langid, LanguageIdentifier};

const FTL_RESOURCES_DIR: &str = "res";

const FTL_RESOURCE_FILENAME: &str = "messages.ftl";

pub fn list_available_locales() -> Vec<LanguageIdentifier> {
    let Ok(dirs) = std::fs::read_dir(FTL_RESOURCES_DIR) else { return Vec::default() };
    let dir_names = dirs.filter_map(|r| {
        r.ok()
            .and_then(|entry| entry.file_name().into_string().ok())
    });
    convert_vec_str_to_langids_lossy(dir_names)
}

pub fn get_fluent_bundle(
    locale: LanguageIdentifier,
) -> FluentBundle<FluentResource> {
    let available = list_available_locales();
    debug!("Available locales: {:?}", available);
    let default_locale = langid!("en");
    let supported: Vec<LanguageIdentifier> = negotiate_languages(
        &[locale],
        &available,
        Some(&default_locale),
        NegotiationStrategy::Filtering,
    )
    .into_iter()
    .cloned()
    .collect();
    debug!("Supported locales: {:?}", supported);
    let mut bundle = FluentBundle::new(supported.to_owned());
    for locale in supported {
        let ftl_path = Path::new(FTL_RESOURCES_DIR)
            .join(locale.to_string())
            .join(FTL_RESOURCE_FILENAME);
        debug!("Loading Fluent resource: {:?}", ftl_path);
        let ftl_path_str = ftl_path.to_owned();
        let ftl_path_str = ftl_path_str.to_str().unwrap_or_default();
        let res_string = read_to_string(ftl_path).expect(
            format!("Failed to load Fluent resource: {}", ftl_path_str)
                .as_str(),
        );
        let res = match FluentResource::try_new(res_string) {
            Ok(res) => res,
            Err((_, err)) => {
                panic!(
                    "Failed to parse Fluent resource: {}\n\n{}",
                    ftl_path_str,
                    Itertools::intersperse(
                        err.into_iter().map(|e| e.to_string()),
                        "\n".to_string()
                    )
                    .collect::<String>()
                )
            }
        };
        let _ = bundle.add_resource(res);
    }
    bundle
}
