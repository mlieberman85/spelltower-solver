use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io;
use std::io::BufRead;
use rayon::prelude::*;
use std::sync::{Mutex};
use tesseract::plumbing::{TessBaseAPI, Pix};
use std::ffi::CString;
use std::env;

// TODO: Generalize below functions to handle more than just the 7x7 puzzle mode.
// TODO: General cleanup and use SRP.
// FIXME: Make the code safer by removing unwrap
// NOTE: I'm leaving the older less efficient functions in here for reference and educational
//       purposes

/// Depth first search for a matrix where it's assumed every element of the matrix is adjacent to
/// other elements in all directions (up, down, left, right, and the 4 diagonals).
fn dfs(arr: &Vec<Vec<char>>, i: i8, j: i8, visited: &mut Vec<Vec<bool>>, target: &[char]) -> bool {
    if target.len() == 0 {
        true
    } else if i < 0 ||
        j < 0 ||
        j >= arr[0].len() as i8 ||
        i >= arr.len() as i8 ||
        visited[i as usize][j as usize]  ||
        arr[i as usize][j as usize] != target[0].to_uppercase().collect::<Vec<_>>()[0] {
        false
    } else {
        visited[i as usize][j as usize] = true;

        let found = dfs(arr, i-1, j, visited, &target[1..]) ||
            dfs(arr, i-1, j-1, visited, &target[1..]) ||
            dfs(arr, i-1, j+1, visited, &target[1..]) ||
            dfs(arr, i+1, j, visited, &target[1..]) ||
            dfs(arr, i+1, j-1, visited, &target[1..]) ||
            dfs(arr, i+1, j+1, visited, &target[1..]) ||
            dfs(arr, i, j-1, visited, &target[1..]) ||
            dfs(arr, i, j+1, visited, &target[1..]);

        visited[i as usize][j as usize] = false;
        return found
    }
}

/// Helper funciton for calling the depth first search.
fn dfs_caller(arr: &Vec<Vec<char>>, i: i8, j: i8, target: &[char]) -> bool {
    dfs(arr, i, j, &mut vec![vec![false; 7]; 7], target)
}

/// Old, single threaded function that finds all words in the grid.
fn get_words_old(arr: &Vec<Vec<char>>, word_dict: HashMap<char, Vec<String>>) -> HashSet<String> {
    let mut words: HashSet<String> = HashSet::new();
    for i in 0..7 {
        for j in 0..7 {
            let k: char = arr[i][j].to_lowercase().collect::<Vec<_>>()[0];
            // TODO: is there a better way to handle the below rather than using "unwrap"
            let v = &Vec::new();
            let l: &Vec<String> = word_dict.get(&k).unwrap_or(v);
            for target in l {
                let t_clone = target.clone();
                if target.chars().count() >= 3 && dfs_caller(arr, i as i8, j as i8, target.chars().collect::<Vec<char>>().as_slice()) {
                    //println!("{:}", t_clone);
                    words.insert(t_clone);
                }
            }
        }
    }
    words
}

/// Old, multithreaded function that finds all words in the grid but uses a mutex locked HashSet for
/// word insertion.
fn get_words_old2(arr: &Vec<Vec<char>>, word_dict: HashMap<char, Vec<String>>) -> HashSet<String> {
    let words: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    let iv = vec![0,1,2,3,4,5,6];
    let jv = vec![0,1,2,3,4,5,6];
    iv.par_iter().for_each(|i|{
        jv.par_iter().for_each(|j|{
            let k: char = arr[*i as usize][*j as usize].to_lowercase().collect::<Vec<_>>()[0];
            // TODO: is there a better way to handle the below rather than using "unwrap"
            let v: &Vec<String> = &Vec::new();
            let l: &Vec<String> = word_dict.get(&k).unwrap_or(v);
            for target in l {
                if target.chars().count() >= 3 && dfs_caller(arr, *i as i8, *j as i8, target.chars().collect::<Vec<char>>().as_slice()) {
                    let mut locked = words.lock().unwrap();
                    locked.insert(target.clone());
                }
            }
        })
    });
    let x = words.lock().unwrap().clone(); x
}

/// Multithreaded function that does a map reduce
fn get_words(arr: &Vec<Vec<char>>, word_dict: HashMap<char, Vec<String>>) -> HashSet<String> {
    let iv = vec![0,1,2,3,4,5,6];
    let jv = vec![0,1,2,3,4,5,6];
    iv.par_iter().flat_map(|i| {
        jv.par_iter().flat_map(|j| {
            let k: char = arr[*i as usize][*j as usize].to_lowercase().collect::<Vec<_>>()[0];
            // TODO: is there a better way to handle the below rather than using "unwrap"
            let l: Vec<String> = word_dict.get(&k).unwrap_or(&Vec::new()).to_vec();
            l.par_iter()
                .map(|w| w.to_string())
                .filter(|target| {
                    target.chars().count() >= 3 && dfs_caller(arr, *i as i8, *j as i8, target.chars().collect::<Vec<char>>().as_slice())
            }).collect::<HashSet<String>>()
        }).collect::<HashSet<String>>()
    }).collect::<HashSet<String>>()
}

fn load_dict(filename: String) -> Result<HashMap<char, Vec<String>>, std::io::Error> {
    let letters: Vec<char> = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
                                  'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
    ];
    let file = File::open(filename)?;
    let lines: io::Result<Vec<String>> = io::BufReader::new(file).lines().collect();
    let mut dict: HashMap<char, Vec<String>> = HashMap::new();
    for letter in letters {
        dict.insert(letter, Vec::new());
    }

    let words = lines?;
    for word in words {
        // FIXME: Is there a better way to handle this other than using unwrap?
        let w = word.chars().next().unwrap().clone();
        let mut mw = word.clone();
        dict.get_mut(&w).unwrap().push(mw);
    }

    Ok(dict)
}

fn load_image_to_matrix(filename: String) -> Vec<Vec<char>> {
    // FIXME: Below is required over using the leptess high level API since it doesn't seem to
    //        support changing parameters yet.
    let mut tba = TessBaseAPI::new();
    // Using OEM_TESSERACT_ONLY since by default Tesseract tries to view the image as actual words
    // and sentences. We want it to only
    tba.init_4(None, Some(&CString::new("eng").unwrap()), tesseract_sys::TessOcrEngineMode_OEM_TESSERACT_ONLY).unwrap();
    tba.set_image_2(&Pix::read(&CString::new(filename).unwrap()).unwrap());
    // Below is needed in order to force tesseract to not recognize "_" or "-" when it's a letter
    // like "H" which it sometimes sees as I-I or similar.
    tba.set_variable(&CString::new("tessedit_char_whitelist").unwrap(),
                     &CString::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ").unwrap()).unwrap();
    let text = tba.get_utf8_text().unwrap();
    let string = text.as_ref().to_str().unwrap().to_string();
    let split = string.split('\n').collect::<Vec<&str>>();
    let grid: Vec<Vec<char>> = split[0..7].into_iter().map(|line| {
        // Below filter is needed because sometimes Tesseract detects whitespace between the
        // characters
        line.chars().filter(|c| c != &' ').collect()
    }).collect();
    // TODO: Implement better error handling
    if grid.len() != 7 || grid[0].len() != 7 {
        panic!("Image couldn't be scanned correctly. Here is grid\n{:?}", grid)
    }

    grid
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // Reminder to self first arg is always the binary
    let algo_str = "Optional Algorithm:
            0 - Multithreaded map/reduce - Default
            1 - Multithreaded with Mutex based HashSet
            2 - Single threaded with HashSet
        ";
    if args.len() > 4 || args.len() < 3 {
        println!("Error: Expected 2-3 arguments\
        Usage: {} <path to dictionary> <path to spelltower screenshot> <optional algorithm selector>
        {}", args[0], algo_str);
    } else {
        let algo_selection = if args.len() == 4 {
            match args[3].as_str()  {
                "0" => get_words,
                "1" => get_words_old2,
                "2" => get_words_old,
                _ => panic!("Invalid algorithm selection, expected:
                {}", algo_str)
            }
        } else {
            get_words
        };
        let dict = load_dict(args[1].clone()).unwrap();
        let grid = &load_image_to_matrix(args[2].clone());
        let mut words = algo_selection(grid, dict).into_iter().collect::<Vec<_>>();
        words.sort_by_key(|k| {
            k.chars().collect::<Vec<char>>().len()
        });

        println!("{:?}", words);
    }
}

