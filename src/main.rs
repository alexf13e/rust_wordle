
use std::{io::{self, stdout, Write}, process::exit};
use std::fs;

use crossterm::{cursor, style::{Color, Print, ResetColor, SetForegroundColor}, terminal};
use rand;

enum Correctness {
    EXACT,
    PART,
    NONE
}

const MAX_GUESSES: u8 = 6;

fn main() {
    let words = fs::read_to_string("./wordle-La.txt").expect("Failed to open wordle-LA.txt");
    let words = words.split('\n').collect::<Vec<&str>>();
    let num_words = words.len();

    println!("Wordle - {MAX_GUESSES} attempts - type q to quit");

    loop {
        let word_index = (rand::random::<f32>() * num_words as f32).floor() as usize;
        let answer_word = words[word_index];
        let answer_length = answer_word.len();

        let mut num_guesses_remaining = MAX_GUESSES;
        while num_guesses_remaining > 0 {
            //print guess count
            let current_guess_num = MAX_GUESSES - num_guesses_remaining + 1;
            print!("{}: ", current_guess_num);
            stdout().flush().unwrap();

            //get user input
            let mut input_word_buf = String::new();
            io::stdin()
            .read_line(&mut input_word_buf)
            .expect("Failed to read input");
        
            let input_word = input_word_buf.to_lowercase();
            let input_word = input_word.trim();

            if input_word == "q" {
                exit(0);
            }
        
            //check input length is correct
            if input_word.len() < answer_length {
                print_user_input_error(String::from(format!("Word is too short, must have {answer_length} letters")), &current_guess_num);
                continue;
            }
            if input_word.len() > answer_length {
                print_user_input_error(String::from(format!("Word is too long, must have {answer_length} letters")), &current_guess_num);
                continue;
            }
            if !words.contains(&input_word) {
                print_user_input_error(String::from(format!("{input_word} is not in the dictionary")), &current_guess_num);
                continue;
            }
            
            //check which input characters are in the answer
            let input_correctness = get_correctness(&input_word, &answer_word);

            //show how correct the guess was
            print_guess_correctness(&input_word, &input_correctness, &current_guess_num);

            //check if word was exact match
            if input_word == answer_word {
                break;
            }
            
            num_guesses_remaining = num_guesses_remaining - 1;
        }

        if num_guesses_remaining <= 0 {
            //ran out of guesses
            println!("Ran out of guesses, the word was {answer_word}");
        } else {
            //guessed the word
            println!("Correctly guessed the word");
        }

        print!("Would you like to play again (y/N): ");
        stdout().flush().unwrap();

        let mut input_play_again = String::new();
            io::stdin()
            .read_line(&mut input_play_again)
            .expect("Failed to read input");

        input_play_again = String::from(input_play_again.trim());
        if input_play_again.to_lowercase() != "y" {
            break;
        }
    }
}

fn print_user_input_error(message: String, current_guess_num: &u8) {
    crossterm::execute!(stdout(),
        cursor::MoveToPreviousLine(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
        Print(format!("{current_guess_num}: ")),
        SetForegroundColor(Color::Red),
        cursor::MoveToColumn(15),
        Print(message),
        ResetColor,
        cursor::MoveToColumn(0)
    ).unwrap();
}

fn get_correctness(input_word: &str, answer_word: &str) -> Vec::<Correctness> {
    let mut input_correctness = Vec::<Correctness>::new();
    
    let input_bytes = input_word.as_bytes();
    let answer_bytes = answer_word.as_bytes();

    for (i, _) in input_bytes.iter().enumerate() {
        if input_bytes[i] == answer_bytes[i] {
            //character matches exactly
            input_correctness.push(Correctness::EXACT);
        }
        else if answer_bytes.contains(&input_bytes[i]) {
            //character is in word
            input_correctness.push(Correctness::PART);
        }
        else {
            //character not in word
            input_correctness.push(Correctness::NONE);
        }
    }

    return input_correctness;
}

fn print_guess_correctness(input_word: &str, input_correctness: &Vec::<Correctness>, current_guess_num: &u8) {    
    crossterm::execute!(stdout(),
        cursor::MoveToPreviousLine(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
        Print(format!("{current_guess_num}: ")),
    ).unwrap();
    
    for (i, &c) in input_word.as_bytes().iter().enumerate() {
        let colour_command = match input_correctness[i] {
            Correctness::EXACT => SetForegroundColor(Color::Green),
            Correctness::PART => SetForegroundColor(Color::Yellow),
            Correctness::NONE => SetForegroundColor(Color::DarkGrey)
        };

        let c = char::from(c);
        crossterm::execute!(stdout(),
            colour_command,
            Print(c),
            ResetColor
        ).unwrap();
    }

    println!();
}
