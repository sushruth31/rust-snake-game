use gloo::timers::callback::Interval;
use gloo::{events::EventListener, utils::window};
use gloo_console::{console, log};
use rand::Rng;
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
const GRID_SIZE: i32 = 10;

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
        <>
            {for rows}
        </>
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

fn is_cell_in_snake(snake: Vec<(i32, i32)>, key: &String) -> bool {
    let (row, col) = from_key(key);
    for r in snake.iter() {
        if r.0 == row && r.1 == col {
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

fn mutate_snake(snake: &mut Vec<(i32, i32)>, newhead: (i32, i32)) -> &mut Vec<(i32, i32)> {
    snake.remove(0);
    snake.push(newhead);
    snake
}

fn create_food() -> (i32, i32) {
    let rand1 = rand::thread_rng().gen_range(0..GRID_SIZE);
    let rand2 = rand::thread_rng().gen_range(0..GRID_SIZE);
    (rand1, rand2)
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum GameResult {
    WIN,
    LOSE,
}

#[function_component(App)]
fn app() -> Html {
    let snake_state: UseStateHandle<Vec<(i32, i32)>> = use_state(|| vec![(5, 5)]);
    let direction_state = use_state(Direction::default);
    let interval_speed = use_state(|| 500);
    let food = use_state(create_food);
    let gameresult: UseStateHandle<Option<GameResult>> = use_state(|| None);
    let result = gameresult.clone();
    let foodval = *food;
    let direction_queue = use_state(|| vec![Direction::default()]);

    {
        let direction_state = direction_state.clone();
        let d = direction_state.clone();
        let snake_state = snake_state.clone();
        let interval_speed = interval_speed.clone();
        use_effect_with_deps(
            move |deps| {
                let (snake, direction, gameresult, food, interval_speed) = deps.clone();
                let snake = snake.clone();
                let handler = Interval::new(*interval_speed, move || {
                    if gameresult.is_some() {
                        return;
                    }
                    let mut movefn: Box<dyn Fn((i32, i32)) -> (i32, i32)> = Box::new(move_left);
                    let prevhead = &(*snake)[snake.len() - 1];
                    let mut newsnake = snake.to_vec();
                    match *direction_state {
                        Direction::LEFT => {
                            movefn = Box::new(move_left);
                        }
                        Direction::RIGHT => {
                            movefn = Box::new(move_right);
                        }
                        Direction::UP => {
                            movefn = Box::new(move_up);
                        }
                        Direction::DOWN => {
                            movefn = Box::new(move_down);
                        }
                        _ => return,
                    }
                    let newhead = (*movefn)(*prevhead);
                    //check if snake is out of bounds
                    if is_out_of_bounds(&newhead) {
                        return gameresult.set(Some(GameResult::LOSE));
                    }
                    mutate_snake(&mut newsnake, newhead);
                    //check if on food
                    if tuples_equal(newhead, *food) {
                        //grow the snake
                        //get the new tail of the snake
                        let mut newtail: (i32, i32);
                        let oldtail = newsnake[0];
                        match *direction {
                            Direction::DOWN => newtail = (oldtail.0 - 1, oldtail.1),
                            Direction::LEFT => newtail = (oldtail.0, oldtail.1 + 1),
                            Direction::UP => newtail = (oldtail.0 + 1, oldtail.1),
                            Direction::RIGHT => newtail = (oldtail.0, oldtail.1 - 1),
                        }
                        newsnake.insert(0, newtail);
                        food.set(create_food());
                        if *interval_speed > 200 {
                            interval_speed.set(*interval_speed - 100);
                        }
                        //increase speed
                    }
                    snake.set(newsnake);
                    //update food;
                });
                || drop(handler)
            },
            (snake_state, d, gameresult, food, interval_speed),
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
        let (row, col) = from_key(&key);
        if is_cell_in_snake((*snake_state).clone(), &key) {
            class.push_str(" snake");
        } else if row == foodval.0 && col == foodval.1 {
            class.push_str(" food");
        }

        html! {
            <div {class}>
            {key}
                </div>
        }
    });
    html! {
        <>
            <h1>{"Welcome to Snake!"}</h1>
            if let Some(result) = *result{
                if result == GameResult::LOSE {
                    <h1>{"You lose"}</h1>
                } else {
                    <h1>{"You Win"}</h1>
                }
            }
            <Grid {render_cell} />
            </>
    }
}

fn is_out_of_bounds(head: &(i32, i32)) -> bool {
    head.0 < 0 || head.1 < 0 || head.0 >= GRID_SIZE || head.1 >= GRID_SIZE
}

fn tuples_equal(t1: (i32, i32), t2: (i32, i32)) -> bool {
    let a1 = tuple_to_vec(&t1);
    let a2 = tuple_to_vec(&t2);
    for num in a1 {
        if !a2.contains(&num) {
            return false;
        }
    }
    true
}

fn main() {
    yew::Renderer::<App>::new().render();
}
