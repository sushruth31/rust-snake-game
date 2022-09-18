use gloo::timers::callback::Interval;
use gloo::{events::EventListener, utils::window};
use gloo_console::{console, log};
use std::default;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{EventTarget, HtmlElement, HtmlInputElement, UrlSearchParams};
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::{navigator, prelude::*, switch::_SwitchProps::render};

#[derive(Properties, PartialEq)]
pub struct GridProps {
    render_cell: Callback<String, Html>,
}
const GRID_SIZE: i32 = 16;

#[function_component]
fn Grid(props: &GridProps) -> Html {
    let rows = (0..=GRID_SIZE).map(|i| {
        let cols = (0..=GRID_SIZE).map(|j| {
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

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}
impl Direction {
    pub fn default() -> Self {
        Direction::LEFT
    }
}

fn move_left(head: (i32, i32)) -> (i32, i32) {
    let (r, c) = head.to_owned();
    (r, c - 1)
}
fn move_right(head: (i32, i32)) -> (i32, i32) {
    let (r, c) = head.to_owned();
    (r, c + 1)
}
fn move_up(head: (i32, i32)) -> (i32, i32) {
    let (r, c) = head.to_owned();
    (r - 1, c)
}
fn move_down(head: (i32, i32)) -> (i32, i32) {
    let (r, c) = head.to_owned();
    (r + 1, c)
}

fn vec_to_tuple(vec: &Vec<i32>) -> (i32, i32) {
    (vec[0], vec[1])
}
fn tuple_to_vec(tuple: &(i32, i32)) -> Vec<i32> {
    let mut new: Vec<i32> = vec![];
    new.push(tuple.0);
    new.push(tuple.1);
    new
}

fn mutate_snake(snake: &mut Vec<Vec<i32>>, newhead: (i32, i32)) -> &mut Vec<Vec<i32>> {
    snake.remove(0);
    snake.push(tuple_to_vec(&newhead));
    snake
}

#[function_component(App)]
fn app() -> Html {
    let snake_state = use_state(|| vec![vec![5, 5]]);
    let direction_state = use_state(Direction::default);
    let interval_speed = use_state(|| 500);

    {
        let direction_state = direction_state.clone();
        let d = direction_state.clone();
        let snake_state = snake_state.clone();
        use_effect_with_deps(
            move |deps| {
                let (snake, direction) = deps.clone();
                let snake = snake.clone();
                let mut movefn: Box<dyn Fn((i32, i32)) -> (i32, i32)> = Box::new(move_left);
                let handler = Interval::new(*interval_speed, move || match *direction_state {
                    Direction::LEFT => {
                        movefn = Box::new(move_left);
                        let prevhead = &(*snake)[snake.len() - 1];
                        let newhead = move_left(vec_to_tuple(prevhead));
                        let mut newsnake = snake.to_vec();
                        mutate_snake(&mut newsnake, newhead);
                        snake.set(newsnake);
                    }
                    Direction::RIGHT => {
                        let prevhead = &(*snake)[snake.len() - 1];
                        let newhead = move_right(vec_to_tuple(prevhead));
                        let mut newsnake = snake.to_vec();
                        mutate_snake(&mut newsnake, newhead);
                        snake.set(newsnake);
                    }
                    Direction::UP => {
                        let prevhead = &(*snake)[snake.len() - 1];
                        let newhead = move_up(vec_to_tuple(prevhead));
                        let mut newsnake = snake.to_vec();
                        mutate_snake(&mut newsnake, newhead);
                        snake.set(newsnake);
                    }
                    Direction::DOWN => {
                        let prevhead = &(*snake)[snake.len() - 1];
                        let newhead = move_down(vec_to_tuple(prevhead));
                        let mut newsnake = snake.to_vec();
                        mutate_snake(&mut newsnake, newhead);
                        snake.set(newsnake);
                    }
                    _ => return,
                });
                || drop(handler)
            },
            (snake_state, d),
        );
    }

    {
        use_effect_with_deps(
            move |_| {
                let document = gloo::utils::document();
                let listener = EventListener::new(&document, "keydown", move |e| {
                    let direction = direction_state.clone();
                    let e = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                    let key = e.key();
                    if key == "Meta".to_string()
                        || key == "Shift".to_string()
                        || key == "Escape".to_string()
                    {
                        return;
                    }
                    if key.contains("Arrow") {
                        match key.as_str() {
                            "ArrowRight" => direction.set(Direction::RIGHT),
                            "ArrowLeft" => direction.set(Direction::LEFT),
                            "ArrowDown" => direction.set(Direction::DOWN),
                            "ArrowUp" => direction.set(Direction::UP),
                            _ => return,
                        }
                    }
                });
                || drop(listener)
            },
            (),
        );
    }
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
