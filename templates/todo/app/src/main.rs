#[cfg(feature = "ssr")]
fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let spec = app::build_spec();
    montrs_core::serve::montrs_serve(spec.router, || {
        leptos::prelude::view! { <app::Shell /> }
    })
    .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
