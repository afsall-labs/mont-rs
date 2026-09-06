#[cfg(feature = "ssr")]
fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let spec = web::build_spec();
    montrs_core::serve::montrs_serve(spec.router, || {
        leptos::prelude::view! { <web::Shell /> }
    })
    .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
