use crate::debugger_command::DebuggerCommand;
use crate::inferior::{Inferior,Status, self};
use nix::{sys::signal};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::dwarf_data::{DwarfData, Error as DwarfError, Line};

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: Vec<usize>,
    
}
enum BreakpointArgType {
    Line(usize),
    FuncName(String),
    Addr(usize),
    Unknown,
}
fn parse_breakpoint_arg(raw_addr: &str) -> BreakpointArgType {
    if raw_addr.to_lowercase().starts_with('*') {
        let raw_addr_without_0x = if raw_addr.to_lowercase().starts_with("0x") {
            &raw_addr[2..]
        } else {
            &raw_addr
        };
        return match usize::from_str_radix(raw_addr_without_0x, 16).ok() {
            Some(addr) =>  {
                BreakpointArgType::Addr(addr)
            }
            None => BreakpointArgType::Unknown
        }
    } 
    if let Some(line) = usize::from_str_radix(raw_addr, 10).ok() {
        return BreakpointArgType::Line(line);
    }
    BreakpointArgType::FuncName(raw_addr.to_string())
}
enum StepStatus {
    Exit,
    Ok,
}
impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);
        debug_data.print();

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: vec![],
        }
    }
    fn parse_address(&mut self, raw_addr: &str) -> Option<usize> {
        match parse_breakpoint_arg(raw_addr) {
            BreakpointArgType::Line(line) => {
                println!("Function get_addr_for_line dosen't work.");
                self.debug_data.get_addr_for_line(None, line)
            }
            BreakpointArgType::FuncName(func) => {
                self.debug_data.get_addr_for_function(None, func.as_str())
            }
            BreakpointArgType::Addr(addr) => {
                Some(addr)
            }
            BreakpointArgType::Unknown => None
        }
    }
    fn resume(&mut self)  {
        match self.inferior.as_mut().unwrap().resume().unwrap() {
            Status::Stopped(s, rip) => {
                println!("Child stopped (signal {})", s);
                if let Some(line) = DwarfData::get_line_from_addr(&self.debug_data, rip) {
                    println!("Stopped at {}", line);
                } else {
                    println!("Stopped at ???");
                }
            }
            Status::Exited(e) => {
                self.inferior.take();
                println!("Child exited (status {})", e);
            }
            Status::Signaled(s) => {
                println!("Signaled {}", s);
            }
        }
    }
    
    fn stopped_at_breakpoint(&mut self) -> Option<usize> {
        if let Some(inferior) = self.inferior.as_mut() {
            let regs = inferior.get_regs()?;
            match inferior.get_breakpoint(&(regs.rip as usize)) {
                Some(_) => {
                    Some(regs.rip as usize)
                }
                None => None,
            }
        } else {
            None
        }
    }
    fn continue_breakpoint(&mut self, addr: &usize) -> Result<StepStatus, nix::Error> {
        let inferior = self.inferior.as_mut().unwrap();
        inferior.unset_breakpoint_t(addr)?;
        inferior.ptrace_step()?;
        match inferior.wait(None)? {
            Status::Stopped(signal::SIGTRAP, _) => {
                inferior.set_breakpoint_t(addr)?;
                Ok(StepStatus::Ok)
            }
            Status::Exited(e) => {
                self.inferior.take();
                println!("Child exited (status {})", e);
                Ok(StepStatus::Exit)
            }
            Status::Stopped(s, _) => {
                println!("Child stopped (signal {})", s);
                Ok(StepStatus::Exit)
            }
            _ => {
                panic!("Something went wrong when continue breakpoint");
            }
        }
    }
    fn continue_normal(&mut self) -> Result<StepStatus, nix::Error> {
            let inferior = self.inferior.as_mut().unwrap();
            inferior.ptrace_step()?;
            match inferior.wait(None)? {
                Status::Stopped(signal::SIGTRAP, _) => {
                    Ok(StepStatus::Ok)
                }
                Status::Exited(e) => {
                    self.inferior.take();
                    println!("Child exited (status {})", e);
                    Ok(StepStatus::Exit)
                }
                Status::Stopped(s, _) => {
                    println!("Child stopped (signal {})", s);
                    Ok(StepStatus::Exit)
                }
                _ => {
                    panic!("Something went wrong when excute code");
                }
            }

    }
    fn single_step(&mut self) -> Result<StepStatus, nix::Error> {
        match self.stopped_at_breakpoint() {
            Some(addr) => {
                self.continue_breakpoint(&addr)?;
                Ok(StepStatus::Ok)
            }
            None => {
                self.continue_normal()
            }
        }
        
    }

    fn current_line(&mut self) -> Option<Line> {
        match self.inferior.as_mut() {
            None => {
                None
            }
            Some(inferior) => {
                let regs = inferior.get_regs()?;
                let addr = regs.rip as usize;
                self.debug_data.get_line_from_addr(addr)
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // if pre inferior still alive, kill it
                    if let Some(mut inferior) = self.inferior.take() {
                        inferior.kill().unwrap();
                    }

                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        self.resume();
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if let Some(mut i) = self.inferior.take() {
                        i.kill().unwrap();
                    }
                    return;
                }
                DebuggerCommand::Continue => {
                    if self.inferior.is_none() {
                        println!("No running program!");
                        continue;
                    }
                    if let Some(addr) = self.stopped_at_breakpoint() {
                        match self.continue_breakpoint(&addr).unwrap() {
                            StepStatus::Exit => {
                                continue;
                            }
                            StepStatus::Ok => {

                            }
                        }
                    }
                    self.resume();
                }
                DebuggerCommand::Breakpoint(b) => {
                    let addr = if let Some(addr_t) = self.parse_address(b.as_str()) {
                        addr_t
                    } else {
                        println!("Unknown address");
                        continue;
                    };

                    println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);

                    self.breakpoints.push(addr);
                    if let Some(inferior) = self.inferior.as_mut() {
                        match inferior.set_breakpoint(&addr) {
                            Ok(_) => {
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                }
                DebuggerCommand::Backtrace => {
                    if let Some(inferior) = self.inferior.as_mut() {
                        match inferior.print_backtrace(&self.debug_data) {
                            Ok(_) => {

                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                }
                DebuggerCommand::Next => {
                    if self.inferior.is_none() {
                        println!("No running program!");
                        continue;
                    }
                    let old_line = self.current_line().unwrap();
                    while let Ok(step_status) = self.single_step() {
                        let now_line = &self.current_line();
                        let mut continue_flag = match now_line {
                            None => true,
                            Some(now_line) => {
                                old_line.number == now_line.number
                                    && 
                                old_line.file == now_line.file
                            }
                        };
                        match step_status {
                            StepStatus::Exit => continue_flag = false,
                            StepStatus::Ok => {}
                        }
                        if continue_flag {
                            continue;
                        }
                        match now_line {
                            None => {
                            }
                            Some(line) => {
                                println!("Stopped at {}", line);
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
