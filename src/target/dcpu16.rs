use super::Target;
use std::{
    fs::{remove_file, write}
};

pub struct Dcpu16;

impl Target for Dcpu16{
    fn prelude(&self) -> String {
        /*format!(
            "SET PC, variables_end
            :stack_size
            DAT {}
            :heap_size
            DAT {}
            :stack_idx
            DAT 0x9000
            :heap_idx
            DAT 0x4000
            :variables_end
            SET Z, heap_idx
            SET PC, start_program
            ", var_size, heap_size)*/

        String::from("SET PC, variables_end\n\
        :heap_idx\n\
        DAT 0x4000\n\
        :variables_end\n\
        SET Z, heap_idx\n\
        SET PC, start_program\n\
        ")
    }

    fn postlude(&self) -> String {
        String::new()
    }

    fn begin_entry_point(&self, var_size: i32, heap_size: i32) -> String {
        String::from(":start_program\n")
    }

    fn end_entry_point(&self) -> String {
        String::new()
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
        String::from("\n    \
        ;free\n\
        SET Y, POP\n\
        SET Y, POP\n    \
        ; free doesn't actually free memory\n\
        ")
    }

    fn store(&self, size: i32)  -> String {
        let mut fstr = String::from("SET I, POP ; store\n");

        for i in 0..size {
            fstr.push_str("STI [I], POP\n"); // STI increases I by 1 
        }

        fstr.push_str("\n");

        fstr
    }

    fn load(&self, size: i32) -> String {
        let mut fstr = String::from("SET I, POP ; load\n");

        for i in 0..size {
            fstr.push_str("STI PUSH, [I]\n");
        }

        fstr
    }

    fn fn_header(&self, name: String) -> String {
        String::new()
    }

    fn fn_definition(&self, name: String, body: String) -> String {
        format!(";startfunc\n\
        :{}\n\
        {}\n\
        SET PC, POP\n\
        ;endfunc\n\
        ", name, body)
    }

    fn call_fn(&self, name: String) -> String {
        format!("JSR {}\n", name) // Is this valid? Will Oak look up stack frames?
    }

    fn call_foreign_fn(&self, name: String) -> String {
        format!("JSR {}\n", name) // Todo: DCPU ABI
    }

    fn begin_while(&self) -> String {
        //Labels would be a much better approach here, but would need a unique numbering system
        //PC here is set to AFTER this instruction
        String::from("SET PUSH, PC ; store loop start\n")
    }

    fn end_while(&self) -> String {
        String::from("\n\
        IFN 0, POP\n\
        SET PC, PEEK ; stored loop start\n\
        SET X, POP ; pop stored loop start\n\
        ")
    }

    fn compile(&self, code: String) -> bool {
        if let Ok(_) = write("main.dasm16", code) {
            return true;
        }

        false
    }
}
