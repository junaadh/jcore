use std::{
    env, fs,
    io::{stdin, Read},
};

use jcore::vm::Machine;

fn main() {
    let mut machine = Machine::new();
    /*
        abi: 32bit instructions
        |---- ----|------------------------|


        arithmetic instructions: opcode: 0001 | 1001
                                    imm
                              |---------------|
        |---- ----|-----|-----|-----|         |
            op       rd    rs   rm

        store load instructions: 0011 | 1011,

                  | rs  |                   |
        |---- ----|-----|-------------------|
            op      rd          imm

        branch instructions: 0101 | 1101

        |---- ----|----|--------------------|
            op     cond         imm

        syscall instructions: 0111 | 1111

        |---- ----|------------------------|
            op              imm

        misc instructions: 0110
        NOP is misc 0b_0110_1111 | 0x6f

        |---- ----|------------------------|
            op
    */

    /*
       ldr r0, #10
       push r0

       ldr r0, #1
       push r0

       pop r1 ; r1 = 10
       pop r2 ; r2 = 1

       add r0, r1, r2 ; r0 = r1 + r2
       push r0
    */

    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        println!("Usage: {} <input>", &args[0]);
    }

    let file = &args[1];
    let mut buffer = Vec::new();

    if file == "-" {
        let mut stdin = stdin().lock();
        stdin.read_to_end(&mut buffer).unwrap();
    } else {
        fs::File::open(file)
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
    }

    buffer
        // .chunks_exact(4)
        // .map(|x| {
        //     u32::from_le_bytes(unsafe {
        //         *(x.as_ptr() as *const [u8; 4])
        //     })
        // })
        .iter()
        .enumerate()
        .try_for_each(|(idx, &word)| {
            machine.mem.write(idx as u32, word)
        })
        .unwrap();

    machine.state();
    println!("{}", "-".repeat(20));
    machine.run(true).unwrap();

    // use jcore::opcode::{Instruction::*, Operand::*};
    // use jcore::register::*;
    // let prog = vec![
    //     Ldr(R0, Imm(10)),
    //     Push(Reg(R0)),
    //     Ldr(R0, Imm(1)),
    //     Push(Reg(R0)),
    //     Pop(Reg(R1)),
    //     Pop(Reg(R2)),
    //     Add(R0, R1, Reg(R2)),
    //     Push(Reg(R0)),
    // ];
    // let prog_len = prog.len();

    // let prog_code = prog
    //     .into_iter()
    //     .map(|x| {
    //         let i = u32::try_from(x).unwrap();
    //         println!("{i} => {i:08x} => {i:032b}");
    //         i
    //     })
    //     .flat_map(|x| {
    //         let b = x.to_le_bytes();
    //         println!("{b:x?}");
    //         b
    //     })
    //     .collect::<Vec<_>>();

    // prog_code
    //     .into_iter()
    //     .enumerate()
    //     .for_each(|(i, x)| machine.mem.write(i as u32, x).unwrap());

    // for _ in 0..prog_len {
    //     machine.step().unwrap();
    //     machine.state();
    // }

    // assert_eq!(machine[SP], 0x400 - 4);

    // println!("{}", machine.mem.read_u32(machine[SP]).unwrap());
    // println!("{:?}", machine.mem);

    // machine.mem.write(0, 00).unwrap();
    // machine.mem.write(1, 11).unwrap();

    // let one = machine.mem.read(0).unwrap();
    // let two = machine.mem.read(1).unwrap();

    // let one_u16 = machine.mem.read_u16(0).unwrap();

    // println!("0x{one:x}, 0x{two:x} | 0x{one_u16:x}");
    // machine.step().unwrap();
    // machine.step().unwrap();
    // machine.step().unwrap();
    // machine.step().unwrap();

    // let mut stack = Stack::<u32, 10>::new();

    // for i in 0..10 {
    //     stack.push(69 * i).unwrap();
    // println!("{stack:?}");
    // println!("{stack:#?}");
    // }

    // println!("{}", stack.peek().unwrap_or_default());
    // println!("{}", stack.peek_at(stack.len - 1).unwrap());

    // let phases = ['|', '/', '-', '\\'];
    // let mut i = 0;
    // while true {
    //     let idx = i % phases.len();
    //     print!("\r {} ", phases[idx]);
    //     std::io::stdout().flush().unwrap();

    //     i += 1;
    //     std::thread::sleep(std::time::Duration::from_millis(100));
    // }
}
