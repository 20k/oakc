use super::Target;
use std::{
    fs::{remove_file, write}
};

pub struct Dcpu16;

impl Target for Dcpu16{
    fn prelude(&self) -> String {
        let mut fstr = String::from("SET PC, definitions_end\n");

        fstr.push_str(include_str!("std.d16"));
    
        fstr.push_str("\
        :definitions_end\n\
        SET Z, heap_idx\n\
        SET PC, start_program\n\
        ");
        
        fstr
    }

    fn postlude(&self) -> String {
        String::from(":end_file\n")
    }

    fn begin_entry_point(&self, var_size: i32, heap_size: i32) -> String {
        String::from(":start_program\nSET Z, heap_idx\n")
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

        String::from("ADD PEEK, POP\n")
    }

    fn subtract(&self) -> String {
        //DCPU subtract is SUB b, a, which does b = b - a
        //the stack definition the OAK model does the second top element of the stack, - the top element of the stack
        //this conveniently maps to DCPU semantics

        String::from("SUB PEEK, POP\n")
    }

    fn divide(&self) -> String {
        //Assuming signed integers
        String::from("DVI PEEK, POP\n")
    }

    fn multiply(&self) -> String {
        String::from("MLI PEEK, POP\n")
    }

    fn allocate(&self) -> String {
        //"HCF ; no allocator"
        String::from("\n    \
        ;allocate\n\
        SET Y, POP   ; pop value off stack to get requested memory size\n\
        SET PUSH, Z  ; push heap address to stack\n\
        ADD Z, Y     ; increment 'heap pointer' by the number of words requested\n\
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
        let simple_assembly = false;

        if(!simple_assembly) {
            let mut fstr = format!("SET I, POP ; Get address to store at\n\
                                    XOR I, 0xFFFF\n");
    
            for i in 0..size {
                let val = -size + 1 + i;
    
                // The assembler I'm using to test doesn't currently support unary arguments
                if(val >= 0) {
                    fstr.push_str(&format!("SET [I + {}], POP\n", val));
                }
                else {
                    fstr.push_str(&format!("SET [I - {}], POP\n", -val));
                }
            }
    
            return fstr;
        }
        // This branch is easier on an assembler because it doesn't require expression support. Its a cycle slower though
        else {
            let mut fstr = format!("SET I, POP ; Get address to store at\n\
                                    XOR I, 0xFFFF\n\
                                    SUB I, {} ; take 2s complement by adding 1, subtract size\n", -size + 1);

            for i in 0..size {
                fstr.push_str("STI [I], POP\n");
            }

            return fstr;
        }
    }

    fn load(&self, size: i32) -> String {
        let mut fstr = String::from("SET I, POP ; load\n\
                                     XOR I, 0xFFFF ; this is equivalent to doing (-I)-1\n"); 

        for i in 0..size {
            //fstr.push_str("SET PUSH, [I + {}]", i); //cycle equivalent
            fstr.push_str("STD PUSH, [I]\n");
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

    fn begin_while(&self) -> String {
        //Labels would be a much better approach here, but would need a unique numbering system
        //PC here is set to AFTER this instruction
        String::from("
                      SET I, [callstack_idx]\n\
                      SET [I], PC ; store loop start\n\
                      ADD [callstack_idx], 1\n\
                      ")
    }

    fn end_while(&self) -> String {
        String::from("\n\
        SUB [callstack_idx], 1\n\
        IFN 0, POP\n\
        SET I, [callstack_idx]\n\
        SET PC, [I]\n\
        ")
    }

    fn compile(&self, code: String) -> bool {
        if let Ok(_) = write("main.dasm16", code) {
            return true;
        }

        false
    }
}
