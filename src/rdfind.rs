use std::collections::HashMap;

pub enum DupType {
    FirstOccurrence,
    WithinSameTree,
    OutsideTree,
    Unknown
}

pub struct RDFindEntry {
    pub duptype: DupType,
    pub id:i64,
    pub depth: i64,
    pub size: usize,
    pub device: i64,
    pub inode: u64,
    pub priority: i64,
    pub name: String
}

pub struct RDGroup<'a> {
    pub first: &'a RDFindEntry,
    pub rest: Vec<&'a RDFindEntry>
}

pub fn parse_results_file(file: std::fs::File) -> Vec<RDFindEntry>{
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

pub fn group_entries(entries: &Vec<RDFindEntry>) -> HashMap<i64, RDGroup<'_>>{
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
