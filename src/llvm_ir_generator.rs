use Stmt;
use std::fmt::Write;

struct CodeGenContext {
    regc  : u32,
    loopc : u32
}

fn write_header(ir : &mut String)
{
    write!(ir, "@memory = global [65536 x i8] zeroinitializer, align 16\n\n").unwrap();
    write!(ir, "@memory_idx = global i32 0, align 4\n\n").unwrap();
    write!(ir, "define i32 @main() {{\n").unwrap(); 
    write!(ir, "entry:\n").unwrap(); 
}

fn write_footer(ir : &mut String)
{
    write!(ir, "  ret i32 0\n").unwrap(); 
    write!(ir, "}}\n\n").unwrap(); 
    write!(ir, "declare i32 @putchar(i32)\n").unwrap(); 
    write!(ir, "declare i32 @getchar()\n").unwrap(); 
}

/*
 * Puts &memory[memory_idx] at register %(return - 1) where `return` is
 * the returned u32.
 */
fn write_get_memory_ref(ir : &mut String, context : &mut CodeGenContext) -> u32
{
    write!(ir, "  %{} = load i32, i32* @memory_idx, align 4\n", context.regc).unwrap();
    write!(ir, "  %{} = zext i32 %{} to i64\n", context.regc + 1, context.regc).unwrap();
    write!(ir, "  %{} = getelementptr inbounds [65536 x i8], [65536 x i8]* @memory, i64 0, i64 %{}\n", context.regc + 2, context.regc + 1).unwrap();
    context.regc += 3;
    return context.regc - 1;
}

fn write_move(ir : &mut String, context : &mut CodeGenContext, n : i32)
{
    write!(ir, "  %{} = load i32, i32* @memory_idx, align 4\n", context.regc).unwrap();
    write!(ir, "  %{} = add i32 %{}, {}\n", context.regc + 1, context.regc, n).unwrap();
    write!(ir, "  store i32 %{}, i32* @memory_idx, align 4\n\n", context.regc + 1).unwrap();
    context.regc += 2;
}

fn write_add(ir : &mut String, context : &mut CodeGenContext, n : i32)
{
    let mem_ref = write_get_memory_ref(ir, context);
    write!(ir, "  %{} = load i8, i8* %{}, align 1\n", context.regc, mem_ref).unwrap();
    write!(ir, "  %{} = add i8 %{}, {}\n", context.regc + 1, context.regc, n).unwrap();
    write!(ir, "  store i8 %{}, i8* %{}, align 1\n\n", context.regc + 1, mem_ref).unwrap();
    context.regc += 2;
}

fn write_getc(ir : &mut String, context : &mut CodeGenContext)
{
    write!(ir, "  %{} = call i32 @getchar()\n", context.regc).unwrap();
    let value = context.regc + 1;
    write!(ir, "  %{} = trunc i32 %{} to i8\n", value, context.regc).unwrap();
    context.regc += 2;
    let mem_ref = write_get_memory_ref(ir, context);
    write!(ir, "  store i8 %{}, i8* %{}, align 1\n\n", value, mem_ref).unwrap();
}

fn write_putc(ir : &mut String, context : &mut CodeGenContext)
{
    write_get_memory_ref(ir, context);
    write!(ir, "  %{} = load i8, i8* %{}, align 1\n", context.regc, context.regc - 1).unwrap();
    write!(ir, "  %{} = zext i8 %{} to i32\n", context.regc + 1, context.regc).unwrap();
    write!(ir, "  %{} = call i32  @putchar(i32 %{})\n\n", context.regc + 2, context.regc + 1).unwrap();
    context.regc += 3;
}

fn write_loop_begin(ir : &mut String, context : &mut CodeGenContext) -> u32
{
    let loop_num = context.loopc;
    write!(ir, "  br label %loop_cond{}\n", loop_num).unwrap();
    write!(ir, "loop_cond{}:\n", loop_num).unwrap();
    write_get_memory_ref(ir, context);
    write!(ir, "  %{} = load i8, i8* %{}, align 1\n", context.regc, context.regc - 1).unwrap();
    write!(ir, "  %{} = icmp eq i8 %{}, 0\n", context.regc + 1, context.regc).unwrap();
    write!(ir, "  br i1 %{}, label %loop_end{}, label %loop_begin{}\n", context.regc + 1, loop_num, loop_num).unwrap();
    write!(ir, "loop_begin{}:\n", loop_num).unwrap();
    context.regc += 2;
    context.loopc += 1;
    return context.loopc - 1;
}

fn write_loop_end(ir : &mut String, loop_num : u32)
{
    write!(ir, "  br label %loop_cond{}\n", loop_num).unwrap();
    write!(ir, "loop_end{}:\n\n", loop_num).unwrap();
}

fn write_code(ir : &mut String, code : &[Stmt], context : &mut CodeGenContext)
{
    for stmt in code {
        match stmt {
            Stmt::Move(n)     => write_move(ir, context, *n),
            Stmt::Add(n)      => write_add(ir, context, *n),
            Stmt::Input       => write_getc(ir, context),
            Stmt::Output      => write_putc(ir, context),
            Stmt::Loop(loop_code) => {
                let loop_num = write_loop_begin(ir, context);
                write_code(ir, loop_code, context);
                write_loop_end(ir, loop_num);
            }
        }
    }
}

pub fn code_gen(code : &[Stmt]) -> String
{
    let mut ir : String = String::new();

    let mut context = CodeGenContext {
        regc:  0,
        loopc: 0,
    };

    write_header(&mut ir);
    write_code(&mut ir, code, &mut context);
    write_footer(&mut ir);

    //println!("{}", ir);
    return ir;
}
