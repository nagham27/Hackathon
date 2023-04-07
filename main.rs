use yew::prelude::*;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::event::ClickEvent;
use log::info;
//use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};
use stdweb::web::html_element::CanvasElement;
use stdweb::web::Date;
use stdweb::web::FillRule;
use web_sys::Element;
use stdweb::web::{document, window, CanvasRenderingContext2d,IElement};
//use yew::format::Json;
//use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{prelude::*, virtual_dom::VNode, Properties};
use yew::{html, Component, Html, Callback};



#[derive(PartialEq, Copy, Clone, Debug,Default)]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub struct CanvasModel {
    props: Props,
    canvas_id: String,
    canvas: Option<CanvasElement>,
    ctx: Option<CanvasRenderingContext2d>,
    cbk: Callback<ClickEvent>,
    animate_cbk: Callback<(usize, i64, usize, usize, bool)>,
    map: Vec<Vec<i64>>,
    current_move: i64,
    won: bool,
    paused: bool,
    reject_click: bool,
    // fetch_service: FetchService,
    // fetch_task: Option<FetchTask>,
    link: ComponentLink<CanvasModel>,
}

// #[derive(Clone, PartialEq, Properties)]
// pub struct Props {
//     pub player1: Option<String>,
//     pub player2: Option<String>,
//     pub difficulty: Difficulty,
//     pub canvas_id: Option<String>,
//     pub game_done_cbk: Callback<i64>,
// }

impl CanvasModel {
    pub fn reset(&mut self) {
        self.map = vec![vec![0; 7]; 6];
        self.current_move = 0;
        self.paused = false;
        self.won = false;
        self.reject_click = false;
        self.clear();
        self.draw_mask();
    }

    #[inline]
    pub fn check_state(&self, state: &Vec<Vec<i64>>) -> (i64, i64) {
        let mut win_val = 0;
        let mut chain_val = 0;
        let (mut temp_r, mut temp_b, mut temp_br, mut temp_tr) = (0, 0, 0, 0);
        for i in 0..6 {
            for j in 0..7 {
                temp_r = 0;
                temp_b = 0;
                temp_br = 0;
                temp_tr = 0;
                for k in 0..=3 {
                    if j + k < 7 {
                        temp_r += state[i][j + k];
                    }

                    if i + k < 6 {
                        temp_b += state[i + k][j];
                    }

                    if i + k < 6 && j + k < 7 {
                        temp_br += state[i + k][j + k];
                    }

                    if i >= k && j + k < 7 {
                        temp_tr += state[i - k][j + k];
                    }
                }
                chain_val += temp_r * temp_r * temp_r;
                chain_val += temp_b * temp_b * temp_b;
                chain_val += temp_br * temp_br * temp_br;
                chain_val += temp_tr * temp_tr * temp_tr;

                if temp_r.abs() == 4 {
                    win_val = temp_r;
                } else if temp_b.abs() == 4 {
                    win_val = temp_b;
                } else if temp_br.abs() == 4 {
                    win_val = temp_br;
                } else if temp_tr.abs() == 4 {
                    win_val = temp_tr;
                }
            }
        }

        return (win_val, chain_val);
    }

    pub fn value(
        &self,
        ai_move_value: i64,
        state: &Vec<Vec<i64>>,
        depth: i64,
        mut alpha: i64,
        mut beta: i64,
    ) -> (i64, i64) {
        let val = self.check_state(state);
        let max_depth = match self.props.difficulty {
            Easy => 1,
            Medium => 3,
            Hard => 5,
        };
        //info!("{:?}", self.props.difficulty);
        if depth >= max_depth {
            // if slow (or memory consumption is high), lower the value
            let mut ret_val = 0;

            // if win, value = +inf
            let win_val = val.0;
            let chain_val = val.1 * ai_move_value;
            ret_val = chain_val;

            // If it lead to winning, then do it
            if win_val == 4 * ai_move_value {
                // AI win, AI wants to win of course
                ret_val = 999999;
            } else if win_val == 4 * ai_move_value * -1 {
                // AI lose, AI hates losing
                ret_val = 999999 * -1;
            }
            ret_val -= depth * depth;

            return (ret_val, -1);
        }

        let win = val.0;
        // if already won, then return the value right away
        if win == 4 * ai_move_value {
            // AI win, AI wants to win of course
            return (999999 - depth * depth, -1);
        }
        if win == 4 * ai_move_value * -1 {
            // AI lose, AI hates losing
            return (999999 * -1 - depth * depth, -1);
        }

        if depth % 2 == 0 {
            return self.min_state(ai_move_value, state, depth + 1, alpha, beta);
        }
        return self.max_state(ai_move_value, state, depth + 1, alpha, beta);
    }

    pub fn max_state(
        &self,
        ai_move_value: i64,
        state: &Vec<Vec<i64>>,
        depth: i64,
        mut alpha: i64,
        mut beta: i64,
    ) -> (i64, i64) {
        let mut v = -100000000007;
        let mut new_move: i64 = -1;
        let mut move_queue = Vec::new();

        for j in 0..7 {
            let temp_state = self.fill_map(state, j, ai_move_value);
            if temp_state[0][0] != 999 {
                let temp_val = self.value(ai_move_value, &temp_state, depth, alpha, beta);
                if temp_val.0 > v {
                    v = temp_val.0;
                    new_move = j as i64;
                    move_queue = Vec::new();
                    move_queue.push(j);
                } else if temp_val.0 == v {
                    move_queue.push(j);
                }

                // alpha-beta pruning
                if v > beta {
                    new_move = self.choose(&move_queue);
                    return (v, new_move);
                }
                alpha = std::cmp::max(alpha, v);
            }
        }
        new_move = self.choose(&move_queue);

        return (v, new_move);
    }

    pub fn min_state(
        &self,
        ai_move_value: i64,
        state: &Vec<Vec<i64>>,
        depth: i64,
        mut alpha: i64,
        mut beta: i64,
    ) -> (i64, i64) {
        let mut v = 100000000007;
        let mut new_move: i64 = -1;
        let mut move_queue = Vec::new();

        for j in 0..7 {
            let temp_state = self.fill_map(state, j, ai_move_value * -1);
            if temp_state[0][0] != 999 {
                let temp_val = self.value(ai_move_value, &temp_state, depth, alpha, beta);
                if temp_val.0 < v {
                    v = temp_val.0;
                    new_move = j as i64;
                    move_queue = Vec::new();
                    move_queue.push(j);
                } else if temp_val.0 == v {
                    move_queue.push(j);
                }

                // alpha-beta pruning
                if v < alpha {
                    new_move = self.choose(&move_queue);
                    return (v, new_move);
                }
                beta = std::cmp::min(beta, v);
            }
        }
        new_move = self.choose(&move_queue);

        return (v, new_move);
    }

    #[inline]
    pub fn get_random_val(&self, val: usize) -> usize {
        //let rand = js! { return Math.random(); };
        let rand = 1;
        let base = rand as f64;
        //let base: f64 = rand.try_into().unwrap();
        let max_val = val as f64;

        return (base * max_val).floor() as usize;
    }

    #[inline]
    pub fn choose(&self, choice: &Vec<usize>) -> i64 {
        let index = self.get_random_val(choice.len());
        return choice[index] as i64;
    }

    pub fn ai(&mut self, ai_move_value: i64) {
        let new_map = self.map.clone();
        let val_choice = self.max_state(ai_move_value, &new_map, 0, -100000000007, 100000000007);

        let val = val_choice.0;
        let choice = val_choice.1;

        self.paused = false;
        // TODO: Add rejectclick callback
        let mut done = self.action(choice as usize, true);

        // TODO: Add rejectclick callback
        while done < 0 {
            //log::info!("Using random agent");
            let random_choice = self.get_random_val(7);
            done = self.action(random_choice, true);
        }
    }

    pub fn fill_map(&self, new_state: &Vec<Vec<i64>>, column: usize, value: i64) -> Vec<Vec<i64>> {
        let mut temp_map = new_state.clone();
        if temp_map[0][column] != 0 || column > 6 {
            temp_map[0][0] = 999; // error code
        }

        let mut done = false;
        let mut row = 0;

        for i in 0..5 {
            if temp_map[i + 1][column] != 0 {
                done = true;
                row = i;
                break;
            }
        }
        if !done {
            row = 5;
        }

        temp_map[row][column] = value;
        return temp_map;
    }

    pub fn draw_circle(&self, x: u32, y: u32, fill: &str, stroke: &str, text: &str) {
        self.ctx.as_ref().unwrap().save();
        self.ctx.as_ref().unwrap().set_fill_style_color(&fill);
        self.ctx.as_ref().unwrap().set_stroke_style_color(&stroke);
        self.ctx.as_ref().unwrap().begin_path();
        self.ctx
            .as_ref()
            .unwrap()
            .arc(x as f64, y as f64, 25.0, 0.0, 2.0 * 3.14159265359, false);
        self.ctx.as_ref().unwrap().fill(FillRule::NonZero);
        self.ctx.as_ref().unwrap().restore();

        let context = self.ctx.as_ref().unwrap();
        context.set_font("bold 30px serif");
        context.restore();
        context.fill_text(text, x as f64 - 12.0, y as f64 + 12.0, None);
    }

    pub fn draw_mask(&self) {
        self.ctx.as_ref().unwrap().save();
        self.ctx.as_ref().unwrap().set_fill_style_color("#00bfff");
        self.ctx.as_ref().unwrap().begin_path();
        for y in 0..6 {
            for x in 0..7 {
                self.ctx.as_ref().unwrap().arc(
                    (75 * x + 100) as f64,
                    (75 * y + 50) as f64,
                    25.0,
                    0.0,
                    2.0 * 3.14159265359,
                    false,
                );
                self.ctx.as_ref().unwrap().rect(
                    (75 * x + 150) as f64,
                    (75 * y) as f64,
                    -100.0,
                    100.0,
                );
            }
        }
        self.ctx.as_ref().unwrap().fill(FillRule::NonZero);
        self.ctx.as_ref().unwrap().restore();
    }

    pub fn draw(&self) {
        for y in 0..6 {
            for x in 0..7 {
                let mut fg_color = "transparent";
                if self.map[y][x] >= 1 {
                    fg_color = "#ff4136";
                } else if self.map[y][x] <= -1 {
                    fg_color = "#ffff00";
                }
                self.draw_circle(
                    (75 * x + 100) as u32,
                    (75 * y + 50) as u32,
                    &fg_color,
                    "black",
                    if self.map[y][x] >= 1 {
                        "X"
                    } else if self.map[y][x] <= -1 {
                        "O"
                    } else {
                        ""
                    },
                );
            }
        }
    }

    pub fn check(&mut self) {
        let (mut temp_r, mut temp_b, mut temp_br, mut temp_tr) = (0, 0, 0, 0);
        for i in 0..6 {
            for j in 0..7 {
                temp_r = 0;
                temp_b = 0;
                temp_br = 0;
                temp_tr = 0;
                for k in 0..=3 {
                    if j + k < 7 {
                        temp_r += self.map[i][j + k];
                    }

                    if i + k < 6 {
                        temp_b += self.map[i + k][j];
                    }

                    if i + k < 6 && j + k < 7 {
                        temp_br += self.map[i + k][j + k];
                    }

                    if i >= k && j + k < 7 {
                        temp_tr += self.map[i - k][j + k];
                    }
                }
                if temp_r.abs() == 4 {
                    self.win(temp_r);
                } else if temp_b.abs() == 4 {
                    self.win(temp_b);
                } else if temp_br.abs() == 4 {
                    self.win(temp_br);
                } else if temp_tr.abs() == 4 {
                    self.win(temp_tr);
                }
            }
        }
        // check if draw
        if (self.current_move == 42) && (!self.won) {
            self.win(0);
        }
    }

    pub fn clear(&self) {
        self.ctx.as_ref().unwrap().clear_rect(
            0.0,
            0.0,
            self.canvas.as_ref().unwrap().width() as f64,
            self.canvas.as_ref().unwrap().height() as f64,
        );
    }

    pub fn on_region(&self, coord: f64, x: f64, radius: f64) -> bool {
        return ((coord - x) * (coord - x) <= radius * radius);
    }

    pub fn player_move(&self) -> i64 {
        if self.current_move % 2 == 0 {
            return 1;
        }
        return -1;
    }

    pub fn animate(
        &mut self,
        column: usize,
        current_move: i64,
        to_row: usize,
        cur_pos: usize,
        mode: bool,
    ) {
        let mut fg_color = "transparent";
        if current_move >= 1 {
            fg_color = "#ff4136";
        } else if current_move <= -1 {
            fg_color = "#ffff00";
        }

        if to_row * 75 >= cur_pos {
            self.clear();
            self.draw();
            self.draw_circle(
                (75 * column + 100) as u32,
                (cur_pos + 50) as u32,
                &fg_color,
                "black",
                if self.player_move() == 1 { "X" } else { "O" },
            );
            self.draw_mask();

            let cloned = self.animate_cbk.clone();
            window().request_animation_frame(enclose!((cloned) move |_| {
                cloned.emit((column, current_move, to_row, cur_pos+25, mode));
            }));
        } else {
            self.map[to_row][column] = self.player_move();
            self.current_move += 1;
            self.draw();
            self.check();
            if mode == false && self.props.player2.as_ref().unwrap() == "Computer" {
                self.ai(-1);
            } else {
                self.reject_click = false;
            }
        }
    }

    pub fn action(&mut self, column: usize, mode: bool) -> i64 {
        if self.paused || self.won {
            return 0;
        }

        if self.map[0][column] != 0 || column > 6 {
            return -1;
        }

        let mut done = false;
        let mut row = 0;
        for i in 0..5 {
            if self.map[i + 1][column] != 0 {
                done = true;
                row = i;
                break;
            }
        }
        if !done {
            row = 5;
        }

        self.animate(column, self.player_move(), row, 0, mode);

        self.paused = true;
        return 1;
    }

    pub fn win(&mut self, player: i64) {
        self.paused = true;
        self.won = true;
        self.reject_click = false;

        let mut msg = String::new();
        if player > 0 {
            msg = format!("{} wins", self.props.player1.as_ref().unwrap());
        } else if player < 0 {
            msg = format!("{} wins", self.props.player2.as_ref().unwrap());
        } else {
            msg = "It's a draw".to_string();
        }

        let to_print = format!("{} - Click on game board to reset", msg);

        self.ctx.as_ref().unwrap().save();
        self.ctx.as_ref().unwrap().set_font("14pt sans-serif");
        self.ctx.as_ref().unwrap().set_fill_style_color("#111");
        self.ctx
            .as_ref()
            .unwrap()
            .fill_text(&to_print, 150.0, 20.0, None);

        // construct game to post
        // let game = Game {
        //     gameNumber: String::new(),
        //     gameType: String::from("Connect-4"),
        //     Player1Name: self.props.player1.as_ref().unwrap().clone(),
        //     Player2Name: self.props.player2.as_ref().unwrap().clone(),
        //     WinnerName: if player > 0 {
        //         self.props.player1.as_ref().unwrap().clone()
        //     } else if player < 0 {
        //         self.props.player2.as_ref().unwrap().clone()
        //     } else {
        //         String::from("Draw")
        //     },
        //     GameDate: Date::now() as u64,
        // };

        // construct callback
        // let callback = self
        //     .link
        //     .callback(move |response: Response<Result<String, Error>>| {
        //         //info!("successfully saved!");
        //         //Message::Ignore
        //     });

        // construct request
        // let request = Request::post("/games")
        //     .header("Content-Type", "application/json")
        //     .body(Json(&game))
        //     .unwrap();

        // send the request
        //self.fetch_task = self.fetch_service.fetch(request, callback).ok();

        self.ctx.as_ref().unwrap().restore();
    }
}

impl Component for CanvasModel {
     type Message = Message;
     type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let canvas_id = props.canvas_id.clone().unwrap();

        let mut map: Vec<Vec<i64>> = vec![vec![0; 7]; 6];

        Self {
            props,
            canvas_id,
            canvas: None,
            ctx: None,
            cbk: link.callback(|e: ClickEvent| Message::Click(e)),
            animate_cbk: link
                .callback(|e: (usize, i64, usize, usize, bool)| Message::AnimateCallback(e)),
            map,
            current_move: 0,
            paused: false,
            won: false,
            reject_click: false,
            // fetch_service: FetchService::new(),
            // fetch_task: None,
            link,
        }
    }

    fn update(&mut self, message: Self::Message) -> ShouldRender {
        match message {
            Message::Click(e) => {
                if self.reject_click {
                    return false;
                }

                if self.won {
                    self.reset();
                    //self.props.game_done_cbk.emit(0);
                    return true;
                }

                let rect = self.canvas.as_ref().unwrap().get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.get_left();

                for j in 0..7 {
                    if self.on_region(x, (75 * j + 100) as f64, 25 as f64) {
                        self.paused = false;

                        let valid = self.action(j, false);
                        if valid == 1 {
                            self.reject_click = true;
                        };

                        break;
                    }
                }
            }
            Message::AnimateCallback((a, b, c, d, e)) => {
                self.animate(a, b, c, d, e);
            }
            Message::Ignore => {}
        };

        true
    }

    fn view(&self) -> Html {
        html! {
            //&self.canvas_id
            <>
            <h1>{"HELLOWW"} </h1>
            <canvas id={"connect_human"} height="480" width="640"></canvas>
            </>
        }
    }

    fn rendered(&mut self,first_render: bool) {
        self.canvas = Some(canvas(self.canvas_id.as_str()));
        self.ctx = Some(context(self.canvas_id.as_str()));

        let ctx = self.ctx.as_ref().unwrap();
        let cloned_cbk = self.cbk.clone();

        self.canvas.as_ref().unwrap().add_event_listener(enclose!(
            (ctx) move | event: ClickEvent | {
                cloned_cbk.emit(event);
            }
        ));

        // clears and draws mask
        self.reset();
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

#[inline(always)]
fn canvas(id: &str) -> CanvasElement {
    // document()
    //     .query_selector(&format!("#{}", id))
    //     .unwrap()
    //     .expect(&format!("Failed to select canvas id #{}", id))
    //     .try_into()
    //     .unwrap()

    stdweb::unstable::TryInto::try_into(document()
         .query_selector("#connect_human")
         .unwrap()
        .expect(&format!("Failed to select canvas id #{}", id))).unwrap()

    // let Some(element) = web_sys::window()
    //                 .unwrap()
    //                 .document()
    //                 .unwrap()
    //                 .get_element_by_id(&format!("#{}", id));
    // element
    // stdweb::unstable::TryInto::try_into(document()
    //      .get_element_by_id("connect_human")
    //      .unwrap()).unwrap().clone().into_canvas().unwrap()
    
    //      document()
    // .get_element_by_id("my-canvas")
    // .unwrap()
    // .try_into::<IElement>()
    // .unwrap()
    // .clone()
    // .into_canvas()
    // .unwrap()
}

#[derive(Clone, PartialEq, Properties,Default)]
pub struct Props {
    pub player1: Option<String>,
    pub player2: Option<String>,
    pub difficulty: Difficulty,
    pub canvas_id: Option<String>,
    //pub game_done_cbk: Callback<i64>,
}

#[derive(Debug)]
pub struct Player {
    pub value: String,
}

#[inline(always)]
fn context(id: &str) -> CanvasRenderingContext2d {
    canvas(id).get_context().unwrap()
}

pub enum Message {
    Click(ClickEvent),
    AnimateCallback((usize, i64, usize, usize, bool)),
    Ignore,
}
//use yew_hooks::prelude::*;
// #[function_component(CanvasMod)]
// fn canvas_mod(props: &Props)-> Html {
//     let canvas_id = props.canvas_id.clone().unwrap();
//     let mut map: Vec<Vec<i64>> = vec![vec![0; 7]; 6];

//     let mut c =CanvasModel {
//         // player1: props.player1.clone(),
//         // player2: props.player2.clone(),
//         // difficulty: props.difficulty.clone(),
//         props: props.clone(),
//         canvas_id,
//         canvas: None,
//         ctx: None,
//         //cbk: link.callback(|e: ClickEvent| Message::Click(e)),
//         // animate_cbk: Callback::from(|e: (usize, i64, usize, usize, bool)| Message::AnimateCallback(e)),
//         animate_cbk: Callback::from(|e: (usize, i64, usize, usize, bool)| ()),
//         map,
//         current_move: 0,
//         paused: false,
//         won: false,
//         reject_click: false,
//         // fetch_service: FetchService::new(),
//         // fetch_task: None,
//         // link,
//     };

//     // let mut c_context_v = None;
//     // let mut context_clone = c_context_v.clone();
//     // let mut c_canvas_clone = None;
//     // let mut xdd = c_canvas_clone.clone();
//     let c_id_clone = c.canvas_id.clone();
//     let canv_state = use_state(|| None);
//     let canv = canv_state.clone();
//     //let cid = use_state(|| None);
//     use_effect(move  || {
        
//         canv.set(Some(canvas(c_id_clone.as_str())));
//         //context_clone = Some(context(c_id_clone.as_str()));

//         || {}

//     });
//     let cdf = canv_state.clone();
//     //let mut x = &*cdf;
//     c.canvas = &*cdf;
//     //c.ctx = c_context_v;

//     // use_mount(|| {
//     //     //debug!("Running effect once on mount");
//     // });

//     wasm_logger::init(wasm_logger::Config::default());

//     match c.canvas {
//         None => (log::info!("CANVAS IS NONE")),
//         _ => (log::info!("CANVAS IS NOTTT NONE"))
//     };
//     // use_effect_once(|| {
//     //     debug!("Running effect once on mount");
        
//     //     || debug!("Running clean-up of effect on unmount")
//     // });
//     //c.canvas = Some(canvas(c.canvas_id.as_str()));
//     //c.ctx = Some(context(c.canvas_id.as_str()));
//     //c.reset();

//     html! {
        
//         <canvas id={props.canvas_id.clone().unwrap()} height="480" width="640"></canvas>
       
//     }
// }

// #[function_component(App)]
// fn app() -> Html {
//     let player1 = Player {
//         value: "SAM".to_string(),
//     };

//     let player2 = Player {
//         value: "JON".to_string(),
//     };
//     html! {
//         <>
//         <h1>{ "Hello World" }</h1>
//         <CanvasModel 
//                     canvas_id = {"connect_human"} 
//                     player1 = {player1.value.clone()} 
//                     player2={player2.value.clone()}
//                     difficulty = {Difficulty::Easy}/>
//                     //game_done_cbk=&self.end_game_callback/>
//         </>
//     }
// }

fn main() {
        let player1 = Player {
        value: "SAM".to_string(),
    };

    let player2 = Player {
        value: "JON".to_string(),
    };
    let p = Props {
        canvas_id : Some("connect_human".to_string()),
        player1 : Some(player1.value.clone()),
        player2 : Some(player2.value.clone()),
        difficulty : Difficulty::Easy

    };
    //yew::Renderer::<App>::new().render();
    //yew::start_app::<CanvasModel>();
    yew::start_app_with_props::<CanvasModel>(p)
}
