//! Ratio · cliente móvil.

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            h1 { "Ratio" }
            p { "El motor de proporciones está en el crate `proporciones`." }
        }
    }
}
