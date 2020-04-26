#![allow(dead_code)]
const MEM_SIZE: usize = 65536;

mod genawaiter_attempt {
    use super::*;
    use genawaiter::{stack::let_gen, yield_};

    struct CPU {
        pc: u16,
        x: u8,
        y: u8,
        a: u8,
        apu_counter: u32,
        cycles: u32,
        instruction_count: u32,
        mem: [u8; MEM_SIZE],
    }

    macro_rules! local_join {
        ($e:expr) => {
            loop {
                match $e.async_resume().await {
                    genawaiter::GeneratorState::Yielded(_) => {}
                    genawaiter::GeneratorState::Complete(_) => {
                        break;
                    }
                }
            }
        };
    }
    impl CPU {
        pub fn new() -> CPU {
            CPU {
                pc: 0,
                x: 0,
                y: 0,
                a: 0,
                apu_counter: 0,
                mem: [0xb9; MEM_SIZE],
                cycles: 0,
                instruction_count: 0,
            }
        }
        /*
        void CPU::executeInstruction() {
          opcode = readMemory(PC++);
          if(FlagM)
          switch(opcode) {  //8-bit accumulator instructions
          case 0xb9:
            address = readMemory(PC++);
            address = readMemory(PC++) | address << 8;
            if(address >> 8 != address + Y >> 8) wait(6);
            A = readMemory(address + Y);
          }
        }
        */
        pub async fn execute_instruction(&mut self) {
            //println!("execute instruction");
            let opcode = self.read_memory(self.pc).await;
            self.pc += 1;
            match opcode {
                // 8-bit accumulator instructions
                0xb9 => {
                    //println!("found 0xb9 instruction");
                    let address: u16 = self.read_memory(self.pc).await.into();
                    self.pc += 1;
                    let address: u16 = self.read_memory(self.pc).await as u16 | address << 8;
                    self.pc += 1;
                    if address >> 8 != address + self.y as u16 >> 8 {
                        self.wait(6).await;
                    }
                    self.a = self.read_memory(address + self.y as u16).await;
                }
                _ => {
                    // do nothing
                }
            }
            self.instruction_count += 1;
        }

        async fn read_memory(&mut self, addr: u16) -> u8 {
            //println!("read_memory");
            self.wait(2).await;
            let data = self.mem[addr as usize];
            self.wait(4).await;
            data
        }

        async fn wait(&mut self, clock_cycles: u32) {
            //println!("wait for {} cycles", clock_cycles);
            self.apu_counter += clock_cycles;
            let_gen!(g, {
                while self.apu_counter > 0 {
                    yield_!(());
                    self.apu_counter -= 1;
                    self.cycles += 1;
                    //println!("total cycles {}", self.cycles);
                }
            });
            local_join!(g);
        }
    }

    pub fn main(iters: usize) {
        use futures::executor::block_on_stream;

        let mut cpu = CPU::new();
        let_gen!(gen, {
            loop {
                cpu.execute_instruction().await;
                //println!("ran for 1 instruction");
                //println!("instruction count: {}", cpu.instruction_count);
                //println!("cycle count: {}", cpu.cycles);

                yield_!(());
            }
        });

        let stream = block_on_stream(gen);
        for _ in stream.take(iters) {}
    }
}

mod tokio_attempt {
    use super::*;

    struct CPU {
        pc: u16,
        x: u8,
        y: u8,
        a: u8,
        apu_counter: u32,
        cycles: u32,
        instruction_count: u32,
        mem: [u8; MEM_SIZE],
    }

    impl CPU {
        pub fn new() -> CPU {
            CPU {
                pc: 0,
                x: 0,
                y: 0,
                a: 0,
                apu_counter: 0,
                mem: [0xb9; MEM_SIZE],
                cycles: 0,
                instruction_count: 0,
            }
        }
        /*
        void CPU::executeInstruction() {
          opcode = readMemory(PC++);
          if(FlagM)
          switch(opcode) {  //8-bit accumulator instructions
          case 0xb9:
            address = readMemory(PC++);
            address = readMemory(PC++) | address << 8;
            if(address >> 8 != address + Y >> 8) wait(6);
            A = readMemory(address + Y);
          }
        }
        */
        pub async fn execute_instruction(&mut self) {
            //println!("execute instruction");
            let opcode = self.read_memory(self.pc).await;
            self.pc += 1;
            match opcode {
                // 8-bit accumulator instructions
                0xb9 => {
                    //println!("found 0xb9 instruction");
                    let address: u16 = self.read_memory(self.pc).await.into();
                    self.pc += 1;
                    let address: u16 = self.read_memory(self.pc).await as u16 | address << 8;
                    self.pc += 1;
                    if address >> 8 != address + self.y as u16 >> 8 {
                        self.wait(6).await;
                    }
                    self.a = self.read_memory(address + self.y as u16).await;
                }
                _ => {
                    // do nothing
                }
            }
            self.instruction_count += 1;
        }

        async fn read_memory(&mut self, addr: u16) -> u8 {
            //println!("read_memory");
            self.wait(2).await;
            let data = self.mem[addr as usize];
            self.wait(4).await;
            data
        }

        async fn wait(&mut self, clock_cycles: u32) {
            //println!("wait for {} cycles", clock_cycles);
            self.apu_counter += clock_cycles;
            while self.apu_counter > 0 {
                tokio::task::yield_now().await;
                self.apu_counter -= 1;
                self.cycles += 1;
                //println!("total cycles {}", self.cycles);
            }
        }
    }

    pub fn main(iters: usize) {
        let mut rt = tokio::runtime::Builder::new()
            .basic_scheduler()
            .build()
            .expect("couldn't build a tokio runtime");

        let mut cpu = CPU::new();
        rt.block_on(async move {
            for _ in 0..iters {
                cpu.execute_instruction().await;
                //println!("ran for 1 instruction");
                //println!("instruction count: {}", cpu.instruction_count);
                //println!("cycle count: {}", cpu.cycles);

                tokio::task::yield_now().await;
            }
        });
    }
}

mod async_std_attempt {
    use super::*;

    struct CPU {
        pc: u16,
        x: u8,
        y: u8,
        a: u8,
        apu_counter: u32,
        cycles: u32,
        instruction_count: u32,
        mem: [u8; MEM_SIZE],
    }

    impl CPU {
        pub fn new() -> CPU {
            CPU {
                pc: 0,
                x: 0,
                y: 0,
                a: 0,
                apu_counter: 0,
                mem: [0xb9; MEM_SIZE],
                cycles: 0,
                instruction_count: 0,
            }
        }
        /*
        void CPU::executeInstruction() {
          opcode = readMemory(PC++);
          if(FlagM)
          switch(opcode) {  //8-bit accumulator instructions
          case 0xb9:
            address = readMemory(PC++);
            address = readMemory(PC++) | address << 8;
            if(address >> 8 != address + Y >> 8) wait(6);
            A = readMemory(address + Y);
          }
        }
        */
        pub async fn execute_instruction(&mut self) {
            //println!("execute instruction");
            let opcode = self.read_memory(self.pc).await;
            self.pc += 1;
            match opcode {
                // 8-bit accumulator instructions
                0xb9 => {
                    //println!("found 0xb9 instruction");
                    let address: u16 = self.read_memory(self.pc).await.into();
                    self.pc += 1;
                    let address: u16 = self.read_memory(self.pc).await as u16 | address << 8;
                    self.pc += 1;
                    if address >> 8 != address + self.y as u16 >> 8 {
                        self.wait(6).await;
                    }
                    self.a = self.read_memory(address + self.y as u16).await;
                }
                _ => {
                    // do nothing
                }
            }
            self.instruction_count += 1;
        }

        async fn read_memory(&mut self, addr: u16) -> u8 {
            //println!("read_memory");
            self.wait(2).await;
            let data = self.mem[addr as usize];
            self.wait(4).await;
            data
        }

        async fn wait(&mut self, clock_cycles: u32) {
            //println!("wait for {} cycles", clock_cycles);
            self.apu_counter += clock_cycles;
            while self.apu_counter > 0 {
                async_std::task::yield_now().await;
                self.apu_counter -= 1;
                self.cycles += 1;
                //println!("total cycles {}", self.cycles);
            }
        }
    }

    pub fn main(iters: usize) {
        let task = async_std::task::spawn(async move {
            let mut cpu = CPU::new();
            for _ in 0..iters {
                cpu.execute_instruction().await;
                //println!("ran for 1 instruction");
                //println!("instruction count: {}", cpu.instruction_count);
                //println!("cycle count: {}", cpu.cycles);

                async_std::task::yield_now().await;
            }
        });
        async_std::task::block_on(task);
    }
}

mod enum_attempt {
    use super::*;

    struct CPU {
        pc: u16,
        x: u8,
        y: u8,
        a: u8,
        apu_counter: u32,
        cycles: u32,
        cycle: u32,
        subcycle: u32,
        instruction_count: u32,
        opcode: u8,
        address: u16,
        mem: [u8; MEM_SIZE],
    }

    impl CPU {
        pub fn new() -> CPU {
            CPU {
                pc: 0,
                x: 0,
                y: 0,
                a: 0,
                apu_counter: 0,
                mem: [0xb9; MEM_SIZE],
                cycles: 0,
                cycle: 1,
                subcycle: 1,
                opcode: 0,
                address: 0,
                instruction_count: 0,
            }
        }
        /*
        void CPU::executeInstructionCycle() {
          if(cycle == 1) {
            opcode = readMemory(PC++);
            cycle = 2;
            return;
          }
          if(FlagM)
          switch(opcode) {  //8-bit accumulator instructions
          case 0xb9:
            switch(cycle) {
            case 2:
              switch(subcycle) {
              case 1:
                subcycle = 2;
                return;
              case 2:
                address = readMemory(PC++);
                subcycle = 1;
                return;
              }
            case 3:
              switch(subcycle) {
              case 1:
                subcycle = 2;
                return;
              case 2:
                address = readMemory(PC++) | address << 8;
                subcycle = 1;
                return;
              }
            case 4:
              //possible penalty cycle when crossing 8-bit page boundaries:
              if(address >> 8 != address + Y >> 8) {
                return;
              }
              cycle++;  //cycle 4 not needed; fall through to cycle 5
            case 5:
              switch(subcycle) {
              case 1:
                subcycle = 2;
                return;
              case 2:
                A = readMemory(address + Y);
                subcycle = 1;
                cycle = 0;  //end of instruction; start a new instruction next time
                return;
              }
            }
          }
        }
*/
        pub fn execute_instruction(&mut self) -> bool {
            //println!("execute instruction");
            if self.cycle == 1 {
                //println!("fetch opcode");
                self.opcode = self.read_memory(self.pc);
                self.pc += 1;
                self.cycle = 2;
                return false;
            }
            match self.opcode {
                // 8-bit accumulator instructions
                0xb9 => {
                    //println!("decode 0xb9");
                    match self.cycle {
                    2 => match self.subcycle {
                        1 => {
                            //println!("waiting");
                            self.subcycle = 2;
                            return false;
                        }
                        2 => {
                            //println!("reading byte");
                            self.address = self.read_memory(self.pc) as u16;
                            self.pc += 1;
                            self.subcycle = 1;
                            self.cycle = 3;
                            return false;
                        }
                        _ => {}
                    }
                    3 => match self.subcycle {
                        1 => {
                            //println!("waiting to read other byte");
                            self.subcycle = 2;
                            return false;
                        }
                        2 => {
                            //println!("reading other byte");
                            self.address = self.read_memory(self.pc)  as u16| self.address << 8;
                            self.pc += 1;
                            self.subcycle = 1;
                            self.cycle = 4;
                            return false;
                        }
                        _ => {}
                    }
                    4 => {
                        // Taking a bit of liberty here
                        //println!("cycle 4");
                        self.cycle += 1;
                        if self.address >> 8 != self.address + self.y as u16 >> 8 {
                            return false;
                        }
                    }
                    5 => match self.subcycle {
                        1 => {
                            //println!("cycle 5 wait");
                            self.subcycle = 2;
                            return false;
                        }
                        2 => {
                            //println!("cycle 5 done");
                            self.a = self.read_memory(self.address + self.y as u16);
                            self.subcycle = 1;
                            self.cycle = 1;
                            self.instruction_count += 1;
                            return true;
                        }
                        _ => {}

                    }
                    _ => {}
                }}
                _ => {
                    // do nothing
                }
            }
            false
        }

        fn read_memory(&mut self, addr: u16) -> u8 {
            //println!("read_memory");
            self.wait(2);
            let data = self.mem[addr as usize];
            self.wait(4);
            data
        }

        fn wait(&mut self, clock_cycles: u32) {
            //println!("wait for {} cycles", clock_cycles);
            self.apu_counter += clock_cycles;
            while self.apu_counter > 0 {
                self.apu_counter -= 1;
                self.cycles += 1;
                //println!("total cycles {}", self.cycles);
            }
        }
    }

    pub fn main(iters: usize) {
        let mut cpu = CPU::new();
        for _ in 0..iters {
            while cpu.execute_instruction() == false {};
            //println!("ran for 1 instruction");
            //println!("instruction count: {}", cpu.instruction_count);
            //println!("cycle count: {}", cpu.cycles);
        }
    }
}
mod null_attempt {
    use super::*;

    struct CPU {
        pc: u16,
        x: u8,
        y: u8,
        a: u8,
        apu_counter: u32,
        cycles: u32,
        instruction_count: u32,
        mem: [u8; MEM_SIZE],
    }

    impl CPU {
        pub fn new() -> CPU {
            CPU {
                pc: 0,
                x: 0,
                y: 0,
                a: 0,
                apu_counter: 0,
                mem: [0xb9; MEM_SIZE],
                cycles: 0,
                instruction_count: 0,
            }
        }
        /*
        void CPU::executeInstruction() {
          opcode = readMemory(PC++);
          if(FlagM)
          switch(opcode) {  //8-bit accumulator instructions
          case 0xb9:
            address = readMemory(PC++);
            address = readMemory(PC++) | address << 8;
            if(address >> 8 != address + Y >> 8) wait(6);
            A = readMemory(address + Y);
          }
        }
        */
        pub fn execute_instruction(&mut self) {
            //println!("execute instruction");
            let opcode = self.read_memory(self.pc);
            self.pc += 1;
            match opcode {
                // 8-bit accumulator instructions
                0xb9 => {
                    //println!("found 0xb9 instruction");
                    let address: u16 = self.read_memory(self.pc).into();
                    self.pc += 1;
                    let address: u16 = self.read_memory(self.pc) as u16 | address << 8;
                    self.pc += 1;
                    if address >> 8 != address + self.y as u16 >> 8 {
                        self.wait(6);
                    }
                    self.a = self.read_memory(address + self.y as u16);
                }
                _ => {
                    // do nothing
                }
            }
            self.instruction_count += 1;
        }

        fn read_memory(&mut self, addr: u16) -> u8 {
            //println!("read_memory");
            self.wait(2);
            let data = self.mem[addr as usize];
            self.wait(4);
            data
        }

        fn wait(&mut self, clock_cycles: u32) {
            //println!("wait for {} cycles", clock_cycles);
            self.apu_counter += clock_cycles;
            while self.apu_counter > 0 {
                self.apu_counter -= 1;
                self.cycles += 1;
                //println!("total cycles {}", self.cycles);
            }
        }
    }

    pub fn main(iters: usize) {
        let mut cpu = CPU::new();
        for _ in 0..iters {
            cpu.execute_instruction();
            //println!("ran for 1 instruction");
            //println!("instruction count: {}", cpu.instruction_count);
            //println!("cycle count: {}", cpu.cycles);
        }
    }
}

macro_rules! timeit {
    ($name:expr, $b:block) => {{
        let start = std::time::Instant::now();
        println!("running {} variant:", $name);
        $b;
        let end = std::time::Instant::now();
        let elapsed = end.duration_since(start);
        println!("elapsed time: {:?}", elapsed);
        println!("----------------------------");
        ($name, elapsed)
    }};
}

fn main() {
    let count = 5_000_000usize;
    let times = vec![
        timeit!("genawaiter", {
            genawaiter_attempt::main(count);
        }),
        timeit!("tokio", {
            tokio_attempt::main(count);
        }),
        timeit!("async-std", {
            async_std_attempt::main(count);
        }),
        timeit!("enum", {
            enum_attempt::main(count);
        }),
        timeit!("null", {
            null_attempt::main(count);
        }),
    ];
    let names = times.iter().map(|(x, _)| *x).collect::<Vec<_>>().join(",");
    println!("{}", names);
    let row = times
        .iter()
        .map(|(_, y)| format!("{:?}", y.as_secs_f64()))
        .collect::<Vec<_>>()
        .join(",");
    println!("{}", row);
}
