use crate::error_template::{AppError, ErrorTemplate};
use leptos::{html::Div, *};
use leptos_meta::*;
use leptos_router::*;

use std::{rc::Rc, time::Duration};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/memory-game-prototype.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| view! {<ErrorPage/>}>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <ClassicRoutes/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn ErrorPage() -> impl IntoView {
    let mut outside_errors = Errors::default();
    outside_errors.insert_with_default_key(AppError::NotFound);
    view! {
        <ErrorTemplate outside_errors/>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="column gap">
            <A href="/classic" class="button">Classic</A>
        </div>
    }
}

#[component(transparent)]
fn ClassicRoutes() -> impl IntoView {
    let passthrough = || view! {<Outlet/>};
    view! {
        <Route path="classic" view=passthrough>
            <Route path="" view=ClassicPage/>
            <Route path=":size" view=ClassicGamePage/>
        </Route>
    }
}

#[component]
fn ClassicPage() -> impl IntoView {
    let links = (3..=20)
        .map(|x| view! { <A href={x.to_string()} class="button">2x{x.to_string()}</A>})
        .collect_view();
    view! {
        <div class="links rows gap">{links}</div>
    }
}

#[derive(Params, PartialEq, Eq, Clone, Copy)]
struct ClassicGameParams {
    size: u8,
}

#[component]
fn ClassicGamePage() -> impl IntoView {
    let params = use_params::<ClassicGameParams>();
    let game = move || {
        params().map(|params| {
            view! {
                <ClassicGame number_of_pairs=params.size/>
            }
        })
    };
    view! {
        <ErrorBoundary fallback=|_| view!{<ErrorPage/>}>
          {game}
        </ErrorBoundary>
    }
}

#[component]
fn ClassicGame(number_of_pairs: u8) -> impl IntoView {
    let number_of_pairs = number_of_pairs as usize;
    let available_pairs = [
        Pair {
            matches: ["A".into(), "A".into()],
        },
        Pair {
            matches: ["B".into(), "B".into()],
        },
        Pair {
            matches: ["C".into(), "C".into()],
        },
        Pair {
            matches: ["D".into(), "D".into()],
        },
        Pair {
            matches: ["E".into(), "E".into()],
        },
        Pair {
            matches: ["F".into(), "F".into()],
        },
    ];

    let mut pairs = Vec::new();
    while pairs.len() < number_of_pairs.into() {
        pairs.extend(
            available_pairs
                .iter()
                .take(number_of_pairs - pairs.len())
                .cloned(),
        );
    }

    view! {
        <Game options=pairs.into()/>
    }
}

#[component]
fn GamePage() -> impl IntoView {
    let options = [
        Pair {
            matches: ["A".into(), "A".into()],
        },
        Pair {
            matches: ["B".into(), "B".into()],
        },
        Pair {
            matches: ["C".into(), "C".into()],
        },
        Pair {
            matches: ["D".into(), "D".into()],
        },
        Pair {
            matches: ["E".into(), "E".into()],
        },
        Pair {
            matches: ["F".into(), "F".into()],
        },
    ]
    .into();

    view! {
        <Game options/>
    }
}

type Pairs = Rc<[Pair]>;

#[component]
fn Game(options: Pairs) -> impl IntoView {
    let (pairs, set_pairs) = create_signal([].into());
    let start = move || {
        set_pairs(options.clone());
    };
    let start_up = start.clone();
    create_effect(move |_| start_up());

    view! {
        <Show when=move || !pairs().is_empty() fallback=|| ()>
            <GameMatch pairs=pairs() restart=start.clone()/>
        </Show>
    }
}

#[component]
fn GameMatch<Restart>(pairs: Pairs, restart: Restart) -> impl IntoView
where
    Restart: Fn() + Clone + 'static,
{
    let mut cards: Vec<_> = pairs
        .into_iter()
        .flat_map(|x| &x.matches)
        .enumerate()
        .map(|(id, item)| {
            let (state, set_state) = create_signal(CardState::default());
            CardData {
                id,
                item: item.clone(),
                state,
                set_state,
            }
        })
        .collect();
    fastrand::shuffle(&mut cards);
    let number_of_cards = cards.len();
    let (cards_left, set_cards_left) = create_signal(cards.len());
    let (win, set_win) = create_signal(false);

    create_effect(move |_| {
        if cards_left() == 0 {
            set_timeout(move || set_win(true), Duration::from_secs(1));
        }
    });

    let (_, set_selected) = create_signal(Vec::<CardData>::new());
    let cards = cards
        .into_iter()
        .map(|card| {
            let card_copy = card.clone();
            let select = move || {
                match (card_copy.state)() {
                    CardState::Unselected | CardState::Failure => {}
                    _ => {
                        return;
                    }
                };
                set_selected.update(|selected| {
                    if selected.len() > 1 {
                        for selected in selected.iter() {
                            selected.set_state.update(|state| {
                                *state = match *state {
                                    CardState::Failure => CardState::Unselected,
                                    CardState::Success => CardState::Hidden,
                                    x => x,
                                };
                            })
                        }

                        selected.clear();
                    }

                    if let Some(other) = selected.last() {
                        if other.id != card_copy.id {
                            let state = if other.item == card_copy.item {
                                set_cards_left.update(|left| {
                                    *left -= selected.len() + 1;
                                });
                                CardState::Success
                            } else {
                                CardState::Failure
                            };

                            other.set_state.set(state);
                            card_copy.set_state.set(state);
                            selected.push(card_copy.clone());
                        }
                    } else {
                        card_copy.set_state.set(CardState::Selected);
                        selected.push(card_copy.clone());
                    }
                });
            };

            let CardData { item, state, .. } = card;

            view! {
                <Card item state select/>
            }
        })
        .collect_view();

    let board_ref = create_node_ref::<Div>();
    let (board_size, set_board_size) = create_signal(None);

    create_effect(move |_| {
        if let Some(board) = board_ref.get_untracked() {
            set_board_size(Some((board.offset_width(), board.offset_height())));
        }
    });
    let handle = window_event_listener(ev::resize, move |_| {
        if let Some(board) = board_ref.get_untracked() {
            set_board_size(Some((board.offset_width(), board.offset_height())));
        }
    });
    on_cleanup(move || handle.remove());

    let board_aspect_ratio =
        move || board_size().map(|(width, height)| width as f32 / height as f32);

    let card_aspect_ratio = 1.4142;
    let gap = 20;
    let columns = move || {
        board_aspect_ratio()
            .map(|x| num_columns(card_aspect_ratio, number_of_cards, x))
            .unwrap_or(1)
    };

    let width = move || {
        board_size()
            .map(|(width, height)| {
                let card_width = (width as f32 - (gap * (columns() - 1)) as f32) / columns() as f32;
                let card_width = (card_width - 0.5).floor();

                let card_height = card_width / card_aspect_ratio;
                let mut rows = number_of_cards / columns();
                if number_of_cards % columns() != 0 {
                    rows += 1;
                }
                let full_height = card_height * rows as f32 + (gap * (rows - 1)) as f32;
                if full_height < height as f32 {
                    return card_width;
                }

                let card_height = (height as f32 - (gap * (rows - 1)) as f32) / rows as f32;
                (card_height - 0.5).floor() * card_aspect_ratio
            })
            .unwrap_or(100.0)
    };

    view! {
        <div class="frame">
            <div
                node_ref=board_ref
                class="board"
                style:font-size=move || format!("{}px", width() / 2.0)
                style=("--gap", format!("{}px",gap))
                style=("--aspect-ratio", card_aspect_ratio)
                style=("--width", move || format!("{}px", width()))
            >
                {cards}
            </div>
        </div>
        <Show when=win fallback=|| ()>
            <WinScreen restart=restart.clone()/>
        </Show>
    }
}

// aspect ratio is width/height.
fn num_columns(card_aspect_ratio: f32, number_of_cards: usize, board_aspect_ratio: f32) -> usize {
    let mut best_aspect_ratio = aspect_ratio_of_layout(card_aspect_ratio, number_of_cards, 1);
    for columns in 2.. {
        let aspect_ratio = aspect_ratio_of_layout(card_aspect_ratio, number_of_cards, columns);
        if (board_aspect_ratio - best_aspect_ratio).abs()
            < (board_aspect_ratio - aspect_ratio).abs()
        {
            return columns - 1;
        }
        best_aspect_ratio = aspect_ratio;
    }

    return 1;
}

fn aspect_ratio_of_layout(card_aspect_ratio: f32, number_of_cards: usize, columns: usize) -> f32 {
    let mut rows = number_of_cards / columns;
    if number_of_cards % columns != 0 {
        rows += 1;
    }

    let width = columns as f32;
    let height = rows as f32 / card_aspect_ratio;

    width / height
}

#[derive(Clone)]
struct CardData {
    id: usize,
    item: Item,
    state: ReadSignal<CardState>,
    set_state: WriteSignal<CardState>,
}

type Item = Rc<str>;

#[derive(Clone)]
struct Pair {
    matches: [Item; 2],
}

#[component]
fn Card<StateFn, SelectFn>(item: Item, state: StateFn, mut select: SelectFn) -> impl IntoView
where
    StateFn: Fn() -> CardState + Copy + 'static,
    SelectFn: FnMut() + 'static,
{
    let flipped = move || state() != CardState::Unselected;
    let success = move || state() == CardState::Success;
    let fail = move || state() == CardState::Failure;
    let show = move || state() != CardState::Hidden;
    view! {
        <div on:click=move |_| select() class="card" class:flipped=flipped class:success=success class:fail=fail>
            <Show when=show fallback=|| ()>
                <div class="front">{item.to_string()}</div>
                <div class="back"></div>
            </Show>
        </div>
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
enum CardState {
    #[default]
    Unselected,
    Selected,
    Failure,
    Success,
    Hidden,
}

#[component]
fn WinScreen<Restart>(restart: Restart) -> impl IntoView
where
    Restart: Fn() + 'static,
{
    view! {
        <div class="shim">
            <div class="popup">
                <h2>"🎉 You win! 🎉"</h2>
                <div class="column gap">
                    <div class="button" on:click=move |_| restart()>"Play Again"</div>
                    <A class="button" href="/">"Home"</A>
                </div>
            </div>
        </div>
    }
}
