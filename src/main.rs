use std::collections::HashMap;


enum DupType {
    FirstOccurrence,
    WithinSameTree,
    OutsideTree,
    Unknown
}

struct RDFindEntry {
    duptype: DupType,
    id:i64,
    depth: i64,
    size: usize,
    device: i64,
    inode: u64,
    priority: i64,
    name: String
}

struct RDGroup<'a> {
    first: &'a RDFindEntry,
    rest: Vec<&'a RDFindEntry>
}

struct State<'a> {
    groups: HashMap<i64, RDGroup<'a>>,
    running: bool,
    needs_refresh: bool
}

fn parse_results_file(file: std::fs::File) -> Vec<RDFindEntry>{
    let mut result : Vec<RDFindEntry> = Vec::new(); 

    let buf_reader = std::io::BufReader::new(file); 

    for raw_line in std::io::BufRead::lines(buf_reader) {
        if raw_line.is_err() {
            continue
        }
        let line = raw_line.unwrap();
        if line.starts_with("#") {
            continue
        }

        let mut parts = line.splitn(8, ' ');
        let duptype = parts.next().unwrap();
        let id = parts.next().unwrap();
        let depth = parts.next().unwrap();
        let size = parts.next().unwrap();
        let device = parts.next().unwrap();
        let inode = parts.next().unwrap();
        let priority = parts.next().unwrap();
        let name = parts.next().unwrap();

        let entry = RDFindEntry {
            duptype : match duptype {
                "DUPTYPE_FIRST_OCCURRENCE" => DupType::FirstOccurrence,
                "DUPTYPE_WITHIN_SAME_TREE" => DupType::WithinSameTree,
                "DUPTYPE_OUTSIDE_TREE" => DupType::OutsideTree,
                _ => DupType::Unknown
            },
            id: id.parse().unwrap(),
            depth: depth.parse().unwrap(),
            size: size.parse().unwrap(),
            device: device.parse().unwrap(),
            inode: inode.parse().unwrap(),
            priority: priority.parse().unwrap(),
            name: name.to_string()
        };

        result.push(entry);
    }

    return result;
}

fn group_entries(entries: &Vec<RDFindEntry>) -> HashMap<i64, RDGroup<'_>>{
    let mut result : HashMap<i64, RDGroup> = HashMap::new();

    for entry in entries.iter() {
        if entry.id >= 0 {
            let group = RDGroup {
                first: entry,
                rest: Vec::new()
            };
            result.insert(entry.id, group);
        } else {
            let id = entry.id * -1;
            let duplicates = result.get_mut(&id).expect("Cannot find entry to group to");
            duplicates.rest.push(entry);
        }
    } 

    return result;
}

fn handle_input(window: &pancurses::Window, state: &mut State){
    match window.getch() {
        Some(pancurses::Input::Character('q')) => {
            state.running = false;
        }
        _ => {}
    }
}

fn render(window: &pancurses::Window, state: &State){
    window.clear();

    let mut row = 0;
    let max_cols = window.get_max_x() as usize;
    //note: they are not sorted
    for (_, group) in state.groups.iter() {
        window.mvprintw(row, 0, group.first.name.chars().take(max_cols).collect::<String>());
        row += 1;
        window.mv(row, 4);
        window.printw(group.first.id.to_string());
        window.printw(" ");
        window.printw(group.first.size.to_string());
        row += 1;

        if row >= window.get_max_y() {
            break
        }
    }

    window.refresh();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <results.txt>", args[0]);
        std::process::exit(1);
    }
    
    let file = std::fs::File::open(&args[1]).expect("Cannot open file");
    let entries = parse_results_file(file);
    println!("Number of entries: {}", entries.len());
    let groups = group_entries(&entries);
    println!("Number of groups: {}", groups.len());
   
    let mut state = State {
        groups : groups,
        running: true,
        needs_refresh: true
    };

    let window = pancurses::initscr();
    pancurses::noecho();
    window.nodelay(true);

    while state.running {
        handle_input(&window, &mut state); 
        if state.needs_refresh {
            render(&window, &state);
            state.needs_refresh = false;
        }
    }

    pancurses::endwin(); 
    
}
