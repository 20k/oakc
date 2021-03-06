use super::Target;
use std::{
    fs::{remove_file, write},
    io::{Error, ErrorKind, Result, Write},
};

pub struct Dcpu16;

impl Target for Dcpu16{
    fn prelude(&self) -> String {
        let mut fstr = String::from("SET PC, definitions_end\n");

        fstr.push_str(include_str!("std.d16"));
    
        fstr.push_str("\
        :definitions_end\n\
        JSR boot_screen\n\
        SET PC, start_program\n\
        ");
        
        fstr
    }

    fn postlude(&self) -> String {
        String::from(":end_file\n")
    }

    fn begin_entry_point(&self, var_size: i32, heap_size: i32) -> String {
        let mut fstr = String::from(":start_program\n");

        fstr.push_str(&format!("SET SP, {}\n", -(var_size + heap_size + 1)));

        fstr
    }

    fn end_entry_point(&self) -> String {
        // temporarily until hardware support is up and running for the stdio, makes it easier to debug
        String::from("SET PC, end_file\n")
    }

    fn push(&self, n: f64) -> String {
        let truncated = n as i16;

        // DCPU16 has a reverse stack at the end of memory
        // SET PUSH, 0 is equivalent to [--SP] = 0
        format!(
        "    ;push\n\
        SET PUSH, {}\n\
        ", truncated)
    }

    fn add(&self) -> String {
        // So, the stack pointer points to the last set element
        // a (the second arg) is handled first, b (the first arg) is handled second
        // Therefore we want ADD PEEK, POP

        // Pop executes first, gets the current value, and then shifts the stack pointer
        // PEEK then executes, looking at the stack pointer, and then adding back to that value

        String::from("    ;add\n\
        ADD PEEK, POP\n")
    }

    fn subtract(&self) -> String {
        //DCPU subtract is SUB b, a, which does b = b - a
        //the stack definition the OAK model does the second top element of the stack, - the top element of the stack
        //this conveniently maps to DCPU semantics

        String::from("    ;sub\n\
        SUB PEEK, POP\n")
    }

    fn divide(&self) -> String {
        //Assuming signed integers
        String::from("    ;div\n\
        DVI PEEK, POP\n")
    }

    fn multiply(&self) -> String {
        String::from("    ;mult\n\
        MLI PEEK, POP\n")
    }

    fn allocate(&self) -> String {
        //"HCF ; no allocator"
        String::from("\n    \
        ;allocate\n\
        SET Y, POP   ; pop value off stack to get requested memory size\n\
        SET PUSH, [heap_idx]  ; push heap address to stack\n\
        ADD [heap_idx], Y     ; increment 'heap pointer' by the number of words requested\n\
        ")
    }

    fn free(&self) -> String {
        String::from("\n\
        ;free\n\
        SET Y, POP\n\
        SET Y, POP\n\
        ; free doesn't actually free memory\n\
        ")
    }

    fn store(&self, size: i32)  -> String {
        let mut fstr = format!(";store\n\
                                SET I, POP ; Get address to store at\n\
                                XOR I, 0xFFFF\n");

        for i in 0..size {
            let val = -size + 1 + i;

            if(val >= 0) {
                fstr.push_str(&format!("SET [I + {}], POP\n", val));
            }
            //This exists because dcpu-ide doesn't support a binary minus, and neither my assembler nor dcpu-ide support a unary minus
            else {
                let pval = (val as i16) as u16;

                fstr.push_str(&format!("SET [I + {}], POP\n", pval));
            }
        }

        return fstr;
    }

    fn load(&self, size: i32) -> String {
        let mut fstr = String::from(";load\n\
                                     SET I, POP ; load\n\
                                     XOR I, 0xFFFF ; this is equivalent to doing (-I)-1\n"); 

        if size == 1 {
            fstr.push_str("SET PUSH, [I]\n");
        }
        else
        {
            for i in 0..size {
                //fstr.push_str("SET PUSH, [I + {}]", i); //cycle equivalent
                fstr.push_str("STD PUSH, [I]\n");
            }
        }

        fstr
    }

    fn fn_header(&self, name: String) -> String {
        String::new()
    }

    fn fn_definition(&self, name: String, body: String) -> String {
        format!(";startfunc\n\
        :{}\n\
        SET I, [callstack_idx]\n\
        SET [I], POP ; set traditional return value in [[callstack_idx]]\n\
        ADD [callstack_idx], 1\n\
        {}\n\
        SUB [callstack_idx], 1\n\
        SET I, [callstack_idx]\n\
        SET PC, [I]\n\
        ;endfunc\n\
        ", name, body)
    }

    fn call_fn(&self, name: String) -> String {
        format!("JSR {} ; native\n", name)
    }

    fn call_foreign_fn(&self, name: String) -> String {
        format!("JSR {} ; foreign\n", name) // Todo: DCPU ABI
    }

    fn begin_while(&self, loop_unique_id: i32) -> String {
        format!(";beginwhile\n\
                 :loop_start_{}\n\
                 IFE 0, POP\n\
                 SET PC, loop_end_{}\n\
                 ", loop_unique_id, loop_unique_id)
    }

    fn end_while(&self, loop_unique_id: i32) -> String {
        format!(";endwhile\n\
                 SET PC, loop_start_{}\n\
                 :loop_end_{}\n\
                 ", loop_unique_id, loop_unique_id)
    }

    fn compile(&self, code: String) -> Result<()> {
        if let Ok(_) = write("main.d16", code) {
            return Result::Ok(());
        }

        Result::Err(Error::new(ErrorKind::Other, "Could not write output"))
    }
}
