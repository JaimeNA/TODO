use std::fs;
use std::io;

use text_colorizer::*;
use crossterm::*;

const FILENAME: &str = "TODO.json";

fn read_file() -> io::Result<Vec<String>> {
    let data = match fs::read_to_string(FILENAME) {
        Ok(v) => v, 
        Err(e) => return Err(e),
    };

    if !data.is_empty() {
        let list: Vec<String> = serde_json::from_str(&data).unwrap();

        return Ok(list);
    }

    Ok(vec![])
}

fn write_file(list: &Vec<String>) -> io::Result<()> {

    let data = serde_json::to_string(list).unwrap();

    match fs::write(FILENAME, data) {
        Ok(_) => Ok(()), 
        Err(e) => Err(e)
    }

}

fn print_list(list: &Vec<String>) {
    println!("{} list: \n", "TODO".green());

    let mut i: u32 = 1;

    for task in list {
        print!("{}. {}", i, task);

        i += 1;
    }
}

fn prompt_input() -> u8 {

    println!("\nOptions:
        1. Add task
        2. Remove task
        3. Display list
        4. Exit program\n");

    println!("Please enter a number and press {}", "ENTER".green());

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    let trimmed = input.trim();
    match trimmed.parse::<u8>() {
        Ok(i) => i,
        Err(..) => 5,
    }
    
}

fn add_task(list: &mut Vec<String>) {

    println!("New task:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    list.push(input);

}

fn remove_task(list: &mut Vec<String>) {
    print_list(&list);

    println!("Select which task to remove: ");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    let trimmed = input.trim();
    match trimmed.parse::<usize>() {
        Ok(i) => {

            if i <= list.len() {
                list.remove(i - 1);
            }else{
                eprintln!("Please select a valid index");
            }

        },
        Err(e) => {
            eprintln!("{} failed to parse argument: {}", "Error:".red().bold(), e);

            std::process::exit(1);
        },
    };
}

fn main() {
 
    let mut stdout = stdout();

  stdout.execute(terminal::Clear(terminal::ClearType::All))?;

  for y in 0..40 {
    for x in 0..150 {
      if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
        // in this loop we are more efficient by not flushing the buffer.
        stdout
          .queue(cursor::MoveTo(x,y))?
          .queue(style::PrintStyledContent( "█".magenta()))?;
      }
    }
  }
  stdout.flush()?;
/*
    println!("\n\n {} - This is a simple todo program", "WELCOME".green().bold());

    let mut list = read_file().expect("reading file");

    if list.is_empty() {

        println!("\nThere are no TODO tasks.\n");

    } else {

        println!("\n Current tasks: \n");
        print_list(&list);
    }

    let mut choice: u8 = 0;

    while choice != 4 {

        choice = prompt_input();

        match choice{
            1 => add_task(&mut list),
            2 => remove_task(&mut list),
            3 => print_list(&list),
            4 => match write_file(&list) {
                Ok(()) => println!("File saved successfully"),
                Err(e) => eprintln!("{} saving file: {}", "ERROR".red(), e),
            }
            _ => println!("No option selected, invalid input"),
        };
    }*/
    
}
