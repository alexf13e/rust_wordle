
use std::{collections::HashMap, io::{self, stdout, Write}, process::exit};
use std::fs;

use crossterm::{cursor, style::{Color, Print, ResetColor, SetForegroundColor}, terminal};
use rand;


enum GuessCorrectness {
    EXACT,
    PART,
    NONE
}

enum LetterState {
    UNKNOWN,
    PRESENT,
    ABSENT
}

enum InputType {
    OK,
    ERROR,
    EXIT
}

struct WordleGame {
    all_answer_words: Vec<String>,
    extra_guessable_words: Vec<String>,
    answer_word: String,
    guess_word: String,
    letter_states: HashMap<char, LetterState>,
    max_guesses: u8,
    current_guess_num: u8,
}

impl WordleGame {
    fn load_word_list(&mut self) {
        self.all_answer_words = fs::read_to_string("./wordle-La.txt")
            .expect("Failed to open wordle-LA.txt")
            .lines()
            .map(String::from)
            .collect();

        self.extra_guessable_words = fs::read_to_string("./wordle-Ta.txt")
            .expect("Failed to open wordle-TA.txt")
            .lines()
            .map(String::from)
            .collect();
    }

    fn new_game(&mut self) {
        self.reset_letter_states();
        self.new_answer_word();
        self.current_guess_num = 1;
    }

    fn new_answer_word(&mut self) {
        let word_index = (rand::random::<f32>() * self.all_answer_words.len() as f32).floor() as usize;
        self.answer_word = self.all_answer_words[word_index].clone();
    }

    fn reset_letter_states(&mut self) {
        for char_code in 97..=122 {
            //lowercase letters a-z
            let letter = char::from_u32(char_code).unwrap();
            self.letter_states.insert(letter, LetterState::UNKNOWN);
        }
    }

    fn print_keyboard(&mut self) {
        fn queue_row_print_commands(row: &str, letter_states: &HashMap<char, LetterState>) {
            for (_, b) in row.as_bytes().iter().enumerate() {
                let letter = *b as char;
                let colour_command = match letter_states.get(&letter).unwrap() {
                    LetterState::UNKNOWN => SetForegroundColor(Color::White),
                    LetterState::PRESENT => SetForegroundColor(Color::Yellow),
                    LetterState::ABSENT => SetForegroundColor(Color::DarkGrey)
                };
    
                crossterm::queue!(stdout(),
                    colour_command,
                    Print(format!("{letter} "))
                ).unwrap();
            }
        }

        //print each row, with each letter being coloured based on if it is in the word, not in the word, or hasn't been
        //checked yet
        let keyboard_row_1 = "qwertyuiop";
        let keyboard_row_2 = "asdfghjkl";
        let keyboard_row_3 = "zxcvbnm";

        //display keyboard 2 rows below current input
        crossterm::queue!(stdout(),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine)
        ).unwrap();
        queue_row_print_commands(&keyboard_row_1, &self.letter_states);
        
        //add spaces to start of row 2 and 3 to make printed keyboard layout more accurate
        crossterm::queue!(stdout(),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(" ")
        ).unwrap();
        queue_row_print_commands(&keyboard_row_2, &self.letter_states);
        
        crossterm::queue!(stdout(),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print("  ")
        ).unwrap();
        queue_row_print_commands(&keyboard_row_3, &self.letter_states);
        
        //set colours back to normal and move back up to the line for user input
        crossterm::queue!(stdout(), ResetColor, cursor::MoveToPreviousLine(4)).unwrap();
        stdout().flush().unwrap();
    }

    fn clear_keyboard(&mut self) {
        crossterm::queue!(stdout(),
            cursor::SavePosition,
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print("\n"),
            terminal::Clear(terminal::ClearType::CurrentLine),
            cursor::RestorePosition
        ).unwrap();
    }

    fn print_guess_num(&mut self) {
        print!("{}: ", self.current_guess_num);
        stdout().flush().unwrap();
    }

    fn input_guess_word(&mut self) {
        let mut input_word_buf = String::new();
        io::stdin()
            .read_line(&mut input_word_buf)
            .expect("Failed to read input");
    
        self.guess_word = input_word_buf.to_lowercase().trim().to_string();
    }

    fn check_guess_errors(&mut self) -> InputType {
        //check if user wanted to quit
        if self.guess_word == "q" {
            return InputType::EXIT;
        }
    
        //check input length is correct
        if self.guess_word.len() < self.answer_word.len() {
            self.print_user_input_error(String::from(format!("Word is too short, must have {} letters",
                self.answer_word.len())));
            return InputType::ERROR;
        }
        if self.guess_word.len() > self.answer_word.len() {
            self.print_user_input_error(String::from(format!("Word is too long, must have {} letters",
                self.answer_word.len())));
            return InputType::ERROR;
        }

        //check if input word is a real word (according to the almighty wordle list)
        if !self.all_answer_words.contains(&self.guess_word) && !self.extra_guessable_words.contains(&self.guess_word) {
            self.print_user_input_error(String::from(format!("{} is not in the dictionary", self.guess_word)));
            return InputType::ERROR;
        }

        InputType::OK
    }

    fn print_user_input_error(&mut self, message: String) {
        crossterm::execute!(stdout(),
            cursor::MoveToPreviousLine(1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(format!("{}: ", self.current_guess_num)),
            SetForegroundColor(Color::Red),
            cursor::MoveToColumn(15),
            Print(message),
            ResetColor,
            cursor::MoveToColumn(0)
        ).unwrap();
    }

    fn get_guess_correctness(&mut self) -> Vec::<GuessCorrectness> {
        let mut input_correctness = Vec::<GuessCorrectness>::new();
        
        let input_bytes = self.guess_word.as_bytes();
        let answer_bytes = self.answer_word.as_bytes();
    
        for (i, _) in input_bytes.iter().enumerate() {
            if input_bytes[i] == answer_bytes[i] {
                //character matches exactly
                input_correctness.push(GuessCorrectness::EXACT);
            }
            else if answer_bytes.contains(&input_bytes[i]) {
                //character is somewhere in word
                input_correctness.push(GuessCorrectness::PART);
            }
            else {
                //character is not in word
                input_correctness.push(GuessCorrectness::NONE);
            }
        }
    
        return input_correctness;
    }

    fn update_letter_states(&mut self, guess_correctness: &Vec::<GuessCorrectness>) {
        let input_bytes = self.guess_word.as_bytes();
        for (i, b) in input_bytes.iter().enumerate() {
            let letter = *b as char;
            let current_state = self.letter_states.get(&letter).unwrap();
            if matches!(*current_state, LetterState::UNKNOWN) {
                let new_state = match guess_correctness[i] {
                    GuessCorrectness::EXACT | GuessCorrectness::PART => LetterState::PRESENT,
                    GuessCorrectness::NONE => LetterState::ABSENT
                };

                self.letter_states.insert(letter, new_state);
            }
        }
    }

    fn print_guess_highlighted(&mut self, input_correctness: &Vec::<GuessCorrectness>) {    
        crossterm::execute!(stdout(),
            cursor::MoveToPreviousLine(1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(format!("{}: ", self.current_guess_num)),
        ).unwrap();
        
        for (i, &c) in self.guess_word.as_bytes().iter().enumerate() {
            let colour_command = match input_correctness[i] {
                GuessCorrectness::EXACT => SetForegroundColor(Color::Green),
                GuessCorrectness::PART => SetForegroundColor(Color::Yellow),
                GuessCorrectness::NONE => SetForegroundColor(Color::DarkGrey)
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

    fn run(&mut self) {
        println!("Wordle - {} attempts - type q to quit", self.max_guesses);

        loop {
            self.new_game();
            println!("The word is {} letters long", self.answer_word.len());

            while self.current_guess_num <= self.max_guesses { //guess num starts from 1
                self.print_keyboard();
                self.print_guess_num();

                self.input_guess_word();
                
                match self.check_guess_errors() {
                    InputType::OK => (), //continue as normal
                    InputType::ERROR => continue, //abort guess and make user input again
                    InputType::EXIT => { //user wanted to exit game
                        self.clear_keyboard();
                        exit(0);
                    } 
                }

                let guess_correctness = self.get_guess_correctness();
                self.update_letter_states(&guess_correctness);
                self.print_guess_highlighted(&guess_correctness);

                if self.guess_word == self.answer_word {
                    break;
                }
                
                self.current_guess_num = self.current_guess_num + 1;
            }

            self.clear_keyboard();

            if self.current_guess_num > self.max_guesses {
                //ran out of guesses
                println!("Ran out of guesses, the word was {}", self.answer_word);
            } else {
                //guessed the word
                println!("Correctly guessed the word");
            }
    
            print!("Would you like to play again (y/N): ");
            stdout().flush().unwrap();
    
            self.input_guess_word();
            if self.guess_word != "y" {
                break;
            }
        }
    }
}


fn init_game() -> WordleGame{
    let mut game = WordleGame {
        all_answer_words: Vec::<String>::new(),
        extra_guessable_words: Vec::<String>::new(),
        answer_word: String::new(),
        guess_word: String::new(),
        letter_states: HashMap::<char, LetterState>::new(),
        max_guesses: 6,
        current_guess_num: 1
    };

    game.load_word_list();
    
    return game;
}

fn main() {
    let mut game = init_game();
    game.run();
}