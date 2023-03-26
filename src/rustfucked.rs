use std::env;
use std::process;
use std::io;
use std::io::stdout;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

mod llvm_ir_generator;

const USAGE_STR : &str = "Usage: ./rustfuck <file>";
const TAPE_SIZE : usize = 65536;

#[derive(PartialEq, Eq, Debug)]
pub enum Stmt 
{
    Move(i32),
    Add(i32),
    Input,
    Output,
    Loop(Vec<Stmt>)
}

struct ProgramState {
    ptr  : i32,
    tape : [i32; TAPE_SIZE]
}

fn exit_with_error(msg : &str)
{
    println!("Error: {}", msg);
    process::exit(1);
}

/* Parse into brainfuck program representation */
fn parse(src : &[u8], start_idx : usize) -> (Vec<Stmt>, usize)
{
    let mut code : Vec<Stmt> = Vec::new();
    let mut i = start_idx;
    while i < src.len() {
        let c = src[i] as char;
        
        /* Handle loop entry */
        if c == '[' {
            let (loop_code, idx_after_loop) = parse(src, i + 1);
            code.push(Stmt::Loop(loop_code));
            i = idx_after_loop;
            continue;
        }
        
        /* Handle loop exit */
        if c == ']' {
            return (code, i + 1);
        }

        /* handle regular statements */
        let maybe_statement = match c {
            '>' => Some(Stmt::Move(1)),
            '<' => Some(Stmt::Move(-1)),
            '+' => Some(Stmt::Add(1)),
            '-' => Some(Stmt::Add(-1)),
            ',' => Some(Stmt::Input),
            '.' => Some(Stmt::Output),
             _  => None
        };

        /* add to program representation */
        if let Some(s) = maybe_statement {
            if code.len() == 0 {
                code.push(s);
            } else {
                let last_idx = code.len() - 1;
                match (&code[last_idx], &s) {
                    (Stmt::Move(n), Stmt::Move(m)) => code[last_idx] = Stmt::Move(n + m),
                    (Stmt::Add(n),  Stmt::Add(m))  => code[last_idx] = Stmt::Add(n + m),
                    (_, _)                         => code.push(s)
                }
            }
        }

        i += 1;
    }

    return (code, 0);
}

fn execute(code : &[Stmt], state : &mut ProgramState) {
    let mut idx = 0;
    let modulo = |v, m| { ((v % m) + m) % m };
    while idx < code.len() {
        match &code[idx] {
            Stmt::Move(n) => state.ptr += n,
            Stmt::Add(n)  => {
                state.tape[state.ptr as usize] += n;
                state.tape[state.ptr as usize]  =
                        modulo(state.tape[state.ptr as usize], 256);
            },
            Stmt::Input   => {
                let input: i32 = std::io::stdin()
                    .bytes() 
                    .next()
                    .and_then(|result| result.ok())
                    .map(|byte| byte as i32)
                    .unwrap();
                state.tape[state.ptr as usize] = modulo(input, 256);
            },
            Stmt::Output  => {
                print!("{}", state.tape[state.ptr as usize] as u8 as char);
                _ = stdout().flush();
            },
            Stmt::Loop(code) => {
                if state.tape[state.ptr as usize] > 0 {
                    execute(&code, state);
                    continue;
                }
            }
        }
        idx += 1;
    }
}

fn main() -> io::Result<()> 
{
    /* read args */
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        exit_with_error(USAGE_STR);
    }

    /* read brainfuck file */
    let filepath = &args[1];
    let file = File::open(filepath)?;
    let mut src = Vec::<u8>::new();
    BufReader::new(file).read_to_end(&mut src)?;

    /* Parse into brainfuck program representation */
    let (program, _) = parse(&src, 0);
  
    /* Debug print program */
    //println!("{:?}", program);

    /* Execute program in interpreter */
    //let mut state = ProgramState {
    //    ptr: 0,
    //    tape: [0; TAPE_SIZE]
    //};
    //execute(&program, &mut state);

    /* Code generation */
    let outfile = filepath
            .split('/').last().unwrap() // strip path
            .split('.').nth(0).unwrap() // strip extension
            .to_owned() + ".ll";        // append llvm ir extension
    llvm_ir_generator::code_gen(&program, &outfile);

    Ok(())
}
