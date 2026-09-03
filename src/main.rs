mod rdfind;
use std::collections::HashMap;

enum SortType {
    Id,
    Depth,
    Size,
    Device,
    Inode,
    Priority,
    Name,
    Copies,
}

const PADDING : i32 = 2;

impl std::fmt::Display for SortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortType::Id => write!(f,"Id"),
            SortType::Depth => write!(f,"Depth"),
            SortType::Size => write!(f,"Size"),
            SortType::Device => write!(f,"Device"),
            SortType::Inode => write!(f,"Inode"),
            SortType::Priority => write!(f,"Priority"),
            SortType::Name => write!(f,"Name"),
            SortType::Copies => write!(f,"Copies"),
        }
    }
}

#[derive(PartialEq)]
enum Mode {
    BrowsingList,
    EditingFilter,
    OpenedGroup,
    Closed
}

struct State<'a> {
    mode: Mode,
    //main data
    groups: HashMap<i64, rdfind::RDGroup<'a>>,
    shown: Vec<i64>,
    //flags
    needs_screen_refresh: bool,
    needs_list_refresh: bool,
    //filters and sorting
    sort: SortType,
    flipped: bool,
    name_filter: String, //implementing number filters is too hard for now
    //vertical scrolling
    entry_offset: usize,
    vertical_offset: usize,
    selected_group_id: Option<i64>
}

fn handle_input(window: &pancurses::Window, state: &mut State){
    match state.mode {
        Mode::BrowsingList => {
            match window.getch() {
                Some(pancurses::Input::Character('q')) => {
                    state.mode = Mode::Closed;
                },
                Some(pancurses::Input::Character('S')) => {
                    //cycle backwards on sorting types 
                    state.sort = match state.sort {
                        SortType::Id => SortType::Copies,
                        SortType::Depth => SortType::Id,
                        SortType::Size => SortType::Depth,
                        SortType::Device => SortType::Size,
                        SortType::Inode => SortType::Device,
                        SortType::Priority => SortType::Inode,
                        SortType::Name => SortType::Priority,
                        SortType::Copies => SortType::Name,
                    };
                    state.needs_list_refresh = true;
                    state.needs_screen_refresh = true;
                }
                Some(pancurses::Input::Character('s')) => {
                    //cycle forwards on sorting types
                    state.sort = match state.sort {
                        SortType::Id => SortType::Depth,
                        SortType::Depth => SortType::Size,
                        SortType::Size => SortType::Device,
                        SortType::Device => SortType::Inode,
                        SortType::Inode => SortType::Priority,
                        SortType::Priority => SortType::Name,
                        SortType::Name => SortType::Copies,
                        SortType::Copies => SortType::Id,
                    };
                    state.needs_list_refresh = true;
                    state.needs_screen_refresh = true;
                },
                Some(pancurses::Input::Character('f')) => {
                    //flip list
                    state.flipped = !state.flipped;
                    state.needs_screen_refresh = true;
                },
                Some(pancurses::Input::Character('/')) => {
                    state.mode = Mode::EditingFilter;
                    state.needs_screen_refresh = true;
                },
                Some(pancurses::Input::Character('j')) => {
                    let entries = state.shown.len();
                    state.entry_offset = if state.entry_offset + 1 < entries {
                        state.entry_offset + 1
                    } else {
                        entries - 1
                    };
                   
                    //adjust vertical offset
                    let max_entries = (window.get_max_y() - PADDING) / 2 ;
                    if state.entry_offset < state.vertical_offset {
                        state.vertical_offset = state.entry_offset;
                        state.needs_screen_refresh = true; 
                    } else if state.entry_offset >= state.vertical_offset + max_entries as usize {
                        state.vertical_offset = ((state.entry_offset as i32) - max_entries + 1) as usize;
                        state.needs_screen_refresh = true; 
                    }
                },
                Some(pancurses::Input::Character('k')) => {
                    state.entry_offset = if state.entry_offset > 0 {
                        state.entry_offset - 1
                    } else {
                        0 
                    };
                   
                    //adjust vertical offset
                    let max_entries = (window.get_max_y() - PADDING) / 2 ;
                    if state.entry_offset < state.vertical_offset {
                        state.vertical_offset = state.entry_offset;
                        state.needs_screen_refresh = true; 
                    } else if state.entry_offset >= state.vertical_offset + max_entries as usize {
                        state.vertical_offset = ((state.entry_offset as i32) - max_entries + 1) as usize;
                        state.needs_screen_refresh = true; 
                    }
                },
                Some(pancurses::Input::Character('\n')) => {
                    state.selected_group_id = Some(state.shown[state.entry_offset]);
                    state.mode = Mode::OpenedGroup;
                    state.needs_screen_refresh = true; 
                }
                _ => {}
            }
        },
        Mode::EditingFilter => {
            match window.getch() {
                Some(pancurses::Input::Character('\n')) => {
                    state.needs_list_refresh = true;
                    state.needs_screen_refresh = true;
                    state.mode = Mode::BrowsingList;
                },
                Some(pancurses::Input::Character('\u{7f}')) => {
                    //backspace
                    state.name_filter.pop();
                    state.needs_screen_refresh = true;
                }
                Some(pancurses::Input::Character(c)) => {
                    state.name_filter.push(c);
                    state.needs_screen_refresh = true;
                }
                _ => {}
            }
        },
        Mode::OpenedGroup => {
            match window.getch() {
                Some(pancurses::Input::Character('\u{1b}')) => {
                    //escape
                    state.mode = Mode::BrowsingList;
                    state.needs_screen_refresh = true;
                }
                /*
                Some(c) => {
                    dbg!(c);
                }
                */
                _ => {}
            } 
        },
        Mode::Closed => {},
    }
}

fn sort_pipeline<K, V, FnE, I, FnC, FnF, E>(groups: &HashMap<K,V>, extractor: FnE, comparator: FnC, finaliser: FnF) -> Vec<E> where
    FnE: Fn((&K,&V)) -> I, 
    FnC: Fn(&I,&I)->std::cmp::Ordering,
    FnF: Fn(&I)->E
{
    let mut intermediate : Vec<I> = groups.iter().map(extractor).collect();
    intermediate.sort_by(comparator);
    return intermediate.iter().map(finaliser).collect();
}

fn recalculate_list(state: &mut State) {
    state.vertical_offset = 0;
    state.entry_offset = 0;
    state.shown.clear();
    //phase 1: filter
    let filtered = state.groups.iter().filter(|(_, group)| 
        group.first.name.contains(&state.name_filter)
    ).collect();

    //phase 2: sort
    match state.sort {
        SortType::Id => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, _)| id.to_owned(),
                |id_a, id_b| id_a.cmp(id_b),
                |id| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Depth => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.depth),
                |(_, depth_a), (_, depth_b)| depth_a.cmp(depth_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Size => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.size),
                |(_, size_a), (_, size_b)| size_a.cmp(&size_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Device => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.device),
                |(_, device_a), (_, device_b)| device_a.cmp(&device_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Inode => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.inode),
                |(_, inode_a), (_, inode_b)| inode_a.cmp(&inode_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Priority => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.priority),
                |(_, priority_a), (_, priority_b)| priority_a.cmp(&priority_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Name => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.first.name.to_owned()),
                |(_, name_a), (_, name_b)| name_a.cmp(name_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
        SortType::Copies => {
            let mut sorted = sort_pipeline(&filtered, 
                |(id, group)| (id.to_owned(), group.rest.len() + 1),
                |(_, count_a), (_, count_b)| count_a.cmp(count_b),
                |(id,_)| id.to_owned().to_owned()
            );
            state.shown.append(&mut sorted);
        },
    }
}

fn render_visible_list<'a, T>(window: &pancurses::Window, state: &'a State, iterator: T) where T : Iterator<Item = &'a i64>{ 
    let mut row = 0;
    let max_cols = window.get_max_x() as usize;
    for id in iterator {
        if let Some(group) = state.groups.get(id) {
            window.mvprintw(row, 0, group.first.name.chars().take(max_cols).collect::<String>());
            row += 1;
            window.mv(row, 4);
            window.printw(format!("Id:{}\tDepth:{}\tSize:{}\tDevice:{}\tInode:{}\tPriority:{}\tCopies:{}",
                group.first.id,
                group.first.depth,
                group.first.size,
                group.first.device,
                group.first.inode,
                group.first.priority,
                group.rest.len() + 1
            ));
            row += 1;
        }
        if row >= window.get_max_y() - PADDING {
            break
        }
    } 
}

fn render_single_group(window: &pancurses::Window, state: &State, group_id: i64){
    let mut row = 0;
    let max_cols = window.get_max_x() as usize;

    window.mvprintw(row, 0, format!("Showing group {}", group_id));
    row += 1;
    let group = state.groups.get(&group_id);
    if group.is_some() {
        let group = group.unwrap();
        window.mvprintw(row, 0, group.first.name.to_string());
        row += 1;
        for other in group.rest.iter() {
            window.mvprintw(row, 0, other.name.to_string());
            row += 1;
            if row >= window.get_max_y() {
                break
            }

        }
    }
}

fn render(window: &pancurses::Window, state: &State){
    window.clear();

    match state.mode {
        Mode::BrowsingList | Mode::EditingFilter => {
            if state.flipped {
                render_visible_list(window, state, state.shown.iter().rev().skip(state.vertical_offset));
            } else {
                render_visible_list(window, state, state.shown.iter().skip(state.vertical_offset));
            }
        },
        Mode::OpenedGroup => {
            if let Some(id) = state.selected_group_id {
                render_single_group(window, state, id); 
            }
        },
        Mode::Closed => {},
    }
}

fn always_render(window: &pancurses::Window, state: &State){
    if state.mode != Mode::OpenedGroup {
        let bottom_right = format!("[{}/{}]", state.entry_offset + 1, state.shown.len());
        window.mvprintw(window.get_max_y() - PADDING + 1, window.get_max_x() - bottom_right.len() as i32, bottom_right);
        window.mvprintw(window.get_max_y() - PADDING + 1, 0, format!("Sorting (s): {}\tFilter (tab): {}", state.sort, state.name_filter));

        if state.mode != Mode::EditingFilter{
            window.mv((state.entry_offset as i32 - state.vertical_offset as i32) * 2,0);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <results.txt>", args[0]);
        std::process::exit(1);
    }
    
    let file = std::fs::File::open(&args[1]).expect("Cannot open file");
    let entries = rdfind::parse_results_file(file);
    println!("Number of entries: {}", entries.len());
    let groups = rdfind::group_entries(&entries);
    println!("Number of groups: {}", groups.len());
   
    let mut state = State {
        groups : groups,
        mode: Mode::BrowsingList,
        needs_screen_refresh: true,
        needs_list_refresh: true,
        shown: Vec::new(),
        sort: SortType::Size,
        flipped: false,
        name_filter: String::new(),
        entry_offset:0,
        vertical_offset:0,
        selected_group_id: None
    };

    let window = pancurses::initscr();
    pancurses::noecho();
    window.nodelay(true);

    while state.mode != Mode::Closed {
        handle_input(&window, &mut state); 
        if state.needs_list_refresh {
            recalculate_list(&mut state);
            state.needs_list_refresh = false;
        }
        if state.needs_screen_refresh {
            render(&window, &state);
            state.needs_screen_refresh = false;
        }
        always_render(&window, &state);
        window.refresh();
    }

    pancurses::endwin(); 
    
}
