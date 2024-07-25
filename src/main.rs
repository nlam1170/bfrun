use std::{fs, io::{Read, Write}, process::exit};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        eprintln!("expected a single file path as argument");
        exit(1);
    }

    let file_name = &args[1];
    let mut program = match Program::from_file(file_name) {
        Ok(p) => p,
        Err(e) => panic!("Eror parsing brainfuck source file: {e:?}")
    };

    if let Err(e) = program.run() {
         panic!("Error running brainfuck program: {e:?}");
    }
}

// > 	Increment the data pointer by one (to point to the next cell to the right).
// < 	Decrement the data pointer by one (to point to the next cell to the left).
// + 	Increment the byte at the data pointer by one.
// - 	Decrement the byte at the data pointer by one.
// . 	Output the byte at the data pointer.
// , 	Accept one byte of input, storing its value in the byte at the data pointer.
// [ 	If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
// ] 	If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.

//valid operation types
#[allow(dead_code)]
#[derive(Debug)]
enum Command {
    ChangeByte(u8),
    ChangePointer(isize),
    Output,
    Input,
    JumpIfZero(usize),
    JumpIfNonZero(usize),
}

#[allow(dead_code)]
#[derive(Debug)]
struct Program {
    memory: [u8; 30_000],
    pointer: usize,
    program_counter: usize,
    commands: Vec<Command>,
}

impl Program {
    fn from_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let byte_content = fs::read(file_name)?;
        let mut commands = Vec::new();
        let mut bracket_stack = Vec::new();

        for b in byte_content {
            let command = match b {
                
                b'+' | b'-' => {
                    let change = if b == b'+' { 1 } else { 1u8.wrapping_neg() };
                    if let Some(Command::ChangeByte(count)) = commands.last_mut() {
                        *count = count.wrapping_add(change);
                        continue;
                    }
                    Command::ChangeByte(change)
                }

                b'>' | b'<' => {
                    let change = if b == b'>' { 1 } else { -1 };
                    if let Some(Command::ChangePointer(value)) = commands.last_mut() {
                        *value += change;
                        continue;
                    }
                    Command::ChangePointer(change)
                }

                b'[' => {
                    let current_index = commands.len();
                    bracket_stack.push(current_index);
                    Command::JumpIfZero(0)
                }

                b']' => {
                    let current_index = commands.len();
                    match bracket_stack.pop() {
                        Some(pair_addr) => {
                            commands[pair_addr] = Command::JumpIfZero(current_index);
                            Command::JumpIfNonZero(pair_addr)
                        }
                        None => return Err("Syntax Error: Brackets are unbalanced".into())
                    }
                }

                b',' => Command::Input,
                b'.' => Command::Output,
                _ => continue
            };
            commands.push(command);
        }

        if let Some(_) = bracket_stack.pop() {
            return Err("Syntax Error: Brackets are unbalanced".into());
        }

        Ok(Self{
            memory: [0;30_000],
            pointer: 0,
            program_counter: 0,
            commands
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = std::io::stdout().lock();
        let mut stdin = std::io::stdin().lock();

        'main: loop {
            match self.commands[self.program_counter] {
                Command::ChangeByte(value) => { self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(value); },
                
                Command::ChangePointer(value) => {
                    let len = self.memory.len() as isize;
                    let value = (len + value % len) as usize;
                    self.pointer = (self.pointer + value) % len as usize;
                },
                
                Command::JumpIfZero(pair_addr) => {
                    if self.memory[self.pointer] == 0 {
                        self.program_counter = pair_addr;
                    }
                }
                
                Command::JumpIfNonZero(pair_addr) => {
                    if self.memory[self.pointer] != 0 {
                        self.program_counter = pair_addr
                    }
                }

                Command::Output => {
                    let value = self.memory[self.pointer];
                    // Writing a non-UTF-8 byte sequence on Windows error out.
                    if !cfg!(target_os = "windows") || value < 128 {
                        stdout.write_all(&[value])?;
                        stdout.flush()?;
                    }
                }

                Command::Input => {
                    loop {
                        let err = stdin.read_exact(&mut self.memory[self.pointer..self.pointer + 1]);
                        match err.as_ref().map_err(|e| e.kind()) {
                            Err(std::io::ErrorKind::UnexpectedEof) => {
                                self.memory[self.pointer] = 0;
                            }
                            _ => err?,
                        }
                        
                        if cfg!(target_os = "windows") && self.memory[self.pointer] == b'\r' {
                            continue;
                        }
                        break;
                    }
                } 
            }

            self.program_counter += 1;

            if self.commands.len() == self.program_counter {
                break 'main;
            }
        }
        Ok(())
    }
}