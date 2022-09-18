use gloo_console::{console, log};
use web_sys::{EventTarget, HtmlElement, HtmlInputElement, UrlSearchParams};
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::{navigator, prelude::*, switch::_SwitchProps::render};

#[derive(Properties, PartialEq)]
pub struct GridProps {
    render_cell: Callback<String, Html>,
}

#[function_component]
fn Grid(props: &GridProps) -> Html {
    let rows = (0..=16).map(|i| {
        let cols = (0..=16).map(|j| {
            let mut key = "".to_owned();
            key.push_str(&i.to_string());
            key.push_str("+");
            key.push_str(&j.to_string());
            props.render_cell.emit(key)
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

fn from_key(key: &String) -> (i32, i32) {
    let split = key.split("+").collect::<Vec<&str>>();
    let items: Vec<i32> = split
        .iter()
        .map(|key| key.to_string().parse::<i32>().unwrap())
        .collect();
    (items[0], items[1])
}

fn is_cell_in_snake(snake: Vec<Vec<i32>>, key: &String) -> bool {
    let (row, col) = from_key(key);
    for r in snake.iter() {
        if r[0] == row && r[1] == col {
            return true;
        }
    }
    false
}

#[function_component]
fn App() -> Html {
    let snake_state = use_state(|| vec![vec![5, 5]]);
    let render_cell = Callback::from(move |key: String| {
        let mut class = "col".to_string();
        if is_cell_in_snake((*snake_state).clone(), &key) {
            class.push_str(" snake");
        }

        html! {
            <div {class}>
            {key}
                </div>
        }
    });
    html! {
        <Grid {render_cell} />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
