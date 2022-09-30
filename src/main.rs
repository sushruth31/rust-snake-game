use gloo::timers::callback::Interval;
use gloo::{events::EventListener, utils::window};
use rand::Rng;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{console, EventTarget, HtmlElement, HtmlInputElement, UrlSearchParams};
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
    let rows = (0..GRID_SIZE).map(|i| {
        let cols = (0..GRID_SIZE).map(|j| {
            let mut key = "".to_string();
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

fn is_cell_in_snake(snake: &Vec<(i32, i32)>, key: &String) -> bool {
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

fn mutate_snake(snake: &mut Vec<(i32, i32)>, newhead: (i32, i32)) -> &mut Vec<(i32, i32)> {
    snake.remove(0);
    snake.push(newhead);
    snake
}

fn create_food(snake: Vec<(i32, i32)>) -> (i32, i32) {
    let mut result: Option<(i32, i32)> = None;

    loop {
        let rand1 = rand::thread_rng().gen_range(0..GRID_SIZE);
        let rand2 = rand::thread_rng().gen_range(0..GRID_SIZE);
        let attempt = (rand1, rand2);
        for tuple in snake.to_vec() {
            if tuples_equal(&tuple, &attempt) {
                continue;
            }
            result = Some(attempt);
        }
        match result {
            Some(val) => {
                return val;
            }
            None => {
                continue;
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum GameResult {
    WIN,
    LOSE,
}

#[function_component(App)]
fn app() -> Html {
    let snake_state: UseStateHandle<Vec<(i32, i32)>> = use_state(|| vec![(5, 5)]);
    let interval_speed = use_state(|| 500);
    let food = use_state(|| create_food(snake_state.to_vec()));
    let gameresult: UseStateHandle<Option<GameResult>> = use_state(|| None);
    let result = gameresult.clone();
    let foodval = *food;
    let direction_state = use_state(Direction::default);
    let game_started = use_state(|| false);
    let game_started_clone = game_started.clone();

    {
        let snake_state = snake_state.clone();
        let interval_speed = interval_speed.clone();
        let direction_state = direction_state.clone();
        let gameresult = gameresult.clone();
        let game_started = game_started.clone();
        use_effect_with_deps(
            move |deps| {
                let (snake, gameresult, food, interval_speed, direction_state, game_started) =
                    deps.clone();
                let handler = Interval::new(*interval_speed, move || {
                    if gameresult.is_some() || !*game_started {
                        return;
                    }
                    //web_sys::console::log_1(&format!("{newdirection:#?}").into());
                    let mut movefn: Box<dyn Fn((i32, i32)) -> (i32, i32)> = Box::new(move_left);
                    let prevhead = &(*snake)[snake.len() - 1];
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
                    }
                    let newhead = (*movefn)(*prevhead);
                    let mut newsnake = snake.to_vec();
                    //check if snake is out of bounds
                    if is_out_of_bounds(&newhead) {
                        snake.set(newsnake);
                        return gameresult.set(Some(GameResult::LOSE));
                    }
                    mutate_snake(&mut newsnake, newhead);

                    //check if on itself
                    if is_snake_in_itself(&newsnake) {
                        return gameresult.set(Some(GameResult::LOSE));
                    }

                    //check if on food
                    if tuples_equal(&newhead, &*food) {
                        //grow the snake
                        //get the new tail of the snake
                        let newtail: (i32, i32);
                        let tail = snake[0];
                        //this should be based on the position of the second el not the direction
                        //if length is 1 add based on direction
                        if snake.to_vec().len() == 1 {
                            match *direction_state {
                                Direction::DOWN => newtail = (tail.0 - 1, tail.1),
                                Direction::UP => newtail = (tail.0 + 1, tail.1),
                                Direction::LEFT => newtail = (tail.0, tail.1 + 1),
                                Direction::RIGHT => newtail = (tail.0, tail.1 - 1),
                            }
                        } else {
                            let compare = newsnake[1];
                            //add based on second to last pos
                            //check if compare is to left
                            if compare.1 < tail.1 {
                                //add left
                                newtail = (tail.0, tail.1 - 1);
                            } else if compare.1 > tail.1 {
                                newtail = (tail.0, tail.1 + 1);
                            } else if compare.0 > tail.0 {
                                newtail = (tail.0 - 1, tail.1);
                            } else {
                                newtail = (tail.0 + 1, tail.1);
                            }
                        }

                        newsnake.insert(0, newtail);
                        food.set(create_food(newsnake.to_vec()));
                        if *interval_speed > 101 {
                            interval_speed.set(*interval_speed - 100);
                        }
                        //increase speed
                    }
                    snake.set(newsnake);
                });
                || drop(handler)
            },
            (
                snake_state,
                gameresult,
                food,
                interval_speed,
                direction_state,
                game_started,
            ),
        );
    }

    {
        let direction_state = direction_state.clone();
        let game_result = gameresult.clone();
        let snake_state = snake_state.clone();
        use_effect_with_deps(
            move |snake_state| {
                let snake_state = snake_state.clone();
                let document = gloo::utils::document();
                let listener = EventListener::new(&document, "keydown", move |e| {
                    let e = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                    let key = e.key();
                    if key == "Meta".to_string()
                        || key == "Shift".to_string()
                        || key == "Escape".to_string()
                    {
                        return;
                    }
                    if key.contains("Arrow") {
                        //check prev direction here to make sure its valid
                        match key.as_str() {
                            "ArrowRight" => {
                                if *direction_state == Direction::LEFT
                                    && snake_state.to_vec().len() > 1
                                {
                                    return game_result.set(Some(GameResult::LOSE));
                                }
                                direction_state.set(Direction::RIGHT);
                            }
                            "ArrowLeft" => {
                                if *direction_state == Direction::RIGHT
                                    && snake_state.to_vec().len() > 1
                                {
                                    return game_result.set(Some(GameResult::LOSE));
                                }
                                direction_state.set(Direction::LEFT);
                            }

                            "ArrowDown" => {
                                if *direction_state == Direction::UP
                                    && snake_state.to_vec().len() > 1
                                {
                                    return game_result.set(Some(GameResult::LOSE));
                                }
                                direction_state.set(Direction::DOWN);
                            }

                            "ArrowUp" => {
                                if *direction_state == Direction::DOWN
                                    && snake_state.to_vec().len() > 1
                                {
                                    return game_result.set(Some(GameResult::LOSE));
                                }
                                direction_state.set(Direction::UP);
                            }
                            _ => return,
                        }
                    }
                });
                || drop(listener)
            },
            snake_state,
        );
    }
    let start_game = Callback::from(move |_: MouseEvent| {
        game_started_clone.set(!*game_started_clone);
    });
    let render_cell = Callback::from(move |key: String| {
        let mut class = "col".to_string();
        let keyvals = from_key(&key);
        if is_cell_in_snake(&snake_state, &key) {
            class.push_str(" snake");
        } else if tuples_equal(&keyvals, &foodval) {
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
            <button onclick={start_game}>{if *game_started {"Pause"} else {"Start"}}</button>
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
    let (r, c) = head.to_owned();
    r < -1 || r >= GRID_SIZE || c < -1 || c >= GRID_SIZE
}

fn tuples_equal(t1: &(i32, i32), t2: &(i32, i32)) -> bool {
    t1.0 == t2.0 && t1.1 == t2.1
}

fn is_snake_in_itself(snake: &Vec<(i32, i32)>) -> bool {
    //remove head
    let len = snake.len();
    for i in 0..len {
        for j in i + 1..len {
            if tuples_equal(&snake[i], &snake[j]) {
                return true;
            }
        }
    }
    false
}

fn main() {
    yew::Renderer::<App>::new().render();
}
