use gloo_console::{console, log};
use web_sys::{EventTarget, HtmlElement, HtmlInputElement, UrlSearchParams};
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::{navigator, prelude::*, switch::_SwitchProps::render};

#[derive(Properties, PartialEq)]
pub struct GridProps {
    on_cell_click: Callback<String>,
}

#[function_component]
fn Grid(props: &GridProps) -> Html {
    let rows = (0..=16).map(|i| {
        let cols = (0..=16).map(|j| {
            let mut key = "".to_owned();
            key.push_str(&i.to_string());
            key.push_str("+");
            key.push_str(&j.to_string());
            let key_copy = key.clone();
            let on_cell_click = props.on_cell_click.clone();

            let onclick = Callback::from(move |e: MouseEvent| {
                let key = key_copy.to_owned();
                on_cell_click.emit(key);
            });
            html! {
                <div {onclick} class="col">
                {key}
                    </div>
            }
        });
        html! {
            <div class="row">
            {for cols}
                </div>
        }
    });

    html! {
        <div>
            <h1>{"Welcome to Snake!"}</h1>
            {for rows}
        </div>
    }
}

#[function_component]
fn App() -> Html {
    let on_cell_click = Callback::from(move |key: String| log!(key));
    html! {
        <Grid {on_cell_click} />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
