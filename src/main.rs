mod i18n;

use std::str::FromStr;

use clap::Parser;
use dioxus::prelude::*;
use dioxus_desktop::tao::window::Theme;
use dioxus_desktop::{launch_with_props, Config, WindowBuilder};
use fluent_bundle::{FluentBundle, FluentResource};
use i18n::get_fluent_bundle;
use sys_locale::get_locale;
use tracing::subscriber::set_global_default;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;
use tuple::{Map, TupleElements};
use unic_langid::subtags::Language;
use unic_langid::LanguageIdentifier;

#[derive(Clone, Parser, Debug)]
#[command(author, version, about)]
struct AppSettings {
    #[arg(
        long,
        conflicts_with = "dark_theme",
        help = "Ignores system theme and uses light theme"
    )]
    light_theme: bool,
    #[arg(
        long,
        conflicts_with = "light_theme",
        help = "Ignores system theme and uses dark theme"
    )]
    dark_theme: bool,
    #[arg(short, long, help = "Explicitly sets the locale")]
    locale: Option<LanguageIdentifier>,
}

impl AppSettings {
    fn theme(&self) -> Option<Theme> {
        if self.light_theme {
            Some(Theme::Light)
        } else if self.dark_theme {
            Some(Theme::Dark)
        } else {
            None
        }
    }

    fn theme_string(&self) -> Option<&'static str> {
        if self.light_theme {
            Some("light")
        } else if self.dark_theme {
            Some("dark")
        } else {
            None
        }
    }
}

fn main() {
    #[allow(unused_assignments)]
    let mut debug = false;
    #[cfg(debug_assertions)]
    {
        debug = true;
    }

    let subscriber = FmtSubscriber::builder()
        .with_max_level(if debug { Level::DEBUG } else { Level::INFO })
        .finish();
    set_global_default(subscriber).expect("Failed to set global subscriber");

    let app_options = AppSettings::parse();
    debug!("App options: {:?}", app_options);
    let locale = app_options
        .locale
        .to_owned()
        .or_else(|| {
            let Some(sys_loc) = get_locale() else { return None };
            LanguageIdentifier::from_str(sys_loc.as_str()).ok()
        })
        .unwrap_or_else(|| {
            LanguageIdentifier::from_parts(
                Language::from_str("en")
                    .expect("Failed to initialize language: en"),
                None,
                None,
                &[],
            )
        });

    let fluent_bundle = get_fluent_bundle(locale);
    let locale = fluent_bundle.locales[0].to_owned();
    let app_props = AppProps {
        fluent_bundle,
        locale,
    };

    launch_with_props(
        app,
        app_props,
        Config::default()
            .with_custom_head(
                "<script src=\"https://cdn.tailwindcss.com\"></script>"
                    .to_string()
                    + if let Some(theme_cls) = app_options.theme_string() {
                        format!(
                        "<script>tailwind.config = {{ darkMode: 'class' }}; \
                        document.documentElement.classList.add('{0}')</script>",
                        theme_cls
                    )
                    } else {
                        String::new()
                    }
                    .as_str(),
            )
            .with_disable_context_menu(!debug)
            //.with_file_drop_handler(|w, e| {})
            .with_window(
                WindowBuilder::default()
                    .with_title("EXIF Paranoia")
                    .with_theme(app_options.theme()),
            ),
    );
}

struct AppProps {
    locale: LanguageIdentifier,
    fluent_bundle: FluentBundle<FluentResource>,
}

impl AppProps {
    fn format_messages<
        'a,
        M: TupleElements<Element = String>,
        I: Map<String, Element = &'a str, Output = M>,
    >(
        &self,
        msg_ids: I,
    ) -> M {
        msg_ids.map(|msg_id| {
            let pattern = self
                .fluent_bundle
                .get_message(msg_id)
                .and_then(|msg| msg.value())
                .expect(&format!("Failed to get message: {}", msg_id));
            let mut errors = vec![];
            self.fluent_bundle
                .format_pattern(pattern, None, &mut errors)
                .to_string()
        })
    }
}

fn app(cx: Scope<AppProps>) -> Element {
    let (drag_here_msg, select_folder_msg) = cx.props.format_messages((
        "blank-slate-drag-here",
        "blank-slate-select-folder",
    ));
    cx.render(rsx! {
        main {
            class: "container mx-auto bg-white dark:bg-slate-800",
            lang: "{cx.props.locale.to_string()}",
            div {
                class: "grid h-screen content-center text-slate-900 dark:text-white",
                div {
                    class: "text-center cursor-default select-none",
                    p {
                        class: "text-lg",
                        "{drag_here_msg}"
                    },
                    p {
                        class: "text-base text-slate-500 dark:text-slate-400",
                        "{select_folder_msg}"
                    },
                },
            },
        },
    })
}
