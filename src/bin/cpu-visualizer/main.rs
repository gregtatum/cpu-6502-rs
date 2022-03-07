mod load_cpu;
#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use cpu_6502::{
    asm::AddressToLabel,
    cpu_6502::Cpu6502,
    log::{init_log, log},
    opcodes::{Mode, ADDRESSING_MODE_TABLE, OPCODE_STRING_TABLE},
};
use std::io::stdout;
use std::io::Write;
use std::{
    collections::{HashMap, VecDeque},
    env,
    error::Error,
    io::{self, Stdout},
};
use termion::{
    event::Key,
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

const BORDER_COLOR: Color = Color::Rgb(150, 150, 150);
const CYAN: Color = Color::Rgb(0, 200, 200);
const MAGENTA: Color = Color::Rgb(200, 100, 200);
const GRAY: Color = Color::Rgb(170, 170, 170);
const DIM_WHITE: Color = Color::Rgb(200, 200, 200);

fn parse_cli_args() -> String {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!(
                "The CPU visualizer expects the first argument to be a path to a raw .asm file."
            );
            eprintln!(
                "cargo run --bin cpu-visualizer src/bin/cpu-visualizer/asm/add-with-carry.asm"
            );
            std::process::exit(1);
        }
    }
}

/// Determines how the Visualizer operates.
#[derive(PartialEq, Clone, Debug, Copy)]
enum VisMode {
    Visualizer,
    Help,
    AddPageMemory,
    Quit,
}

struct Visualizer {
    last_drawn_tick_count: u64,
    last_drawn_mode: Option<VisMode>,
    cpu: Cpu6502,
    address_to_label: HashMap<u16, String>,
    mode: VisMode,
    events: Events,
    executed_instructions: VecDeque<Spans<'static>>,
    add_page_address: String,
    draw_is_dirty: bool,
    last_size: Rect,
    pages: Vec<u8>,
}

type VisTerminal =
    Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;

impl Visualizer {
    pub fn new() -> Result<Visualizer, Box<dyn Error>> {
        // Load the CPU first, as this can exit the process.
        let filename = parse_cli_args();
        log(&format!("Loading file {}", filename));
        let (cpu, address_to_label) = load_cpu::load_cpu(&filename);
        let mut events = Events::new();
        // Our event processing handles exiting.
        events.disable_exit_key();

        Ok(Visualizer {
            last_drawn_tick_count: u64::MAX,
            last_drawn_mode: None,
            cpu,
            address_to_label,
            mode: VisMode::Visualizer,
            events,
            executed_instructions: VecDeque::new(),
            add_page_address: String::new(),
            draw_is_dirty: false,
            last_size: Default::default(),
            pages: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = {
            let stdout = io::stdout().into_raw_mode()?;
            let stdout = MouseTerminal::from(stdout);
            let stdout = AlternateScreen::from(stdout);
            let backend = TermionBackend::new(stdout);
            Terminal::new(backend)?
        };

        loop {
            let size = terminal.size().expect("Unable to get the terminal size");

            // Check if the state is dirty and needs to be redrawn.
            if self.last_drawn_tick_count != self.cpu.tick_count
                || Some(self.mode) != self.last_drawn_mode
                || self.last_size != size
                || self.draw_is_dirty
            {
                match self.mode {
                    VisMode::Visualizer => {
                        self.draw_cpu_visualizer(&mut terminal)?;
                    }
                    VisMode::Help => {
                        self.draw_help(&mut terminal)?;
                    }
                    VisMode::AddPageMemory => {
                        self.draw_add_page_memory(&mut terminal)?;
                    }
                    VisMode::Quit => return Ok(()),
                };
                self.draw_is_dirty = false;
                self.last_size = size;
            }

            self.last_drawn_mode = Some(self.mode);

            self.process_events()?;
        }
    }

    fn draw_help(&mut self, terminal: &mut VisTerminal) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            let help = vec![
                //
                "   n - next instruction",
                " 1-9 - next instructions exponentionally ",
                " h/? - show help",
                "   q - quit",
                "   a - add a page of memory",
                "   r - remove a page of memory",
            ];
            let mut width = 0;
            for s in help.iter() {
                width = width.max(s.len());
            }
            frame.render_widget(
                Paragraph::new(help.join("\n"))
                    .block(create_block("Help"))
                    .alignment(Alignment::Left),
                Rect::new(0, 0, width as u16 + 2, help.len() as u16 + 2),
            );
        })?;
        Ok(())
    }

    fn draw_add_page_memory(
        &mut self,
        terminal: &mut VisTerminal,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|frame| {
            frame.set_cursor(
                // Put cursor past the end of the input text
                self.add_page_address.len() as u16 + 1,
                // Move one line down, from the border to the input line
                1,
            );

            let title = "Add a page of memory";

            frame.render_widget(
                Paragraph::new(format!("{}", self.add_page_address.clone()))
                    .block(create_block(title))
                    .alignment(Alignment::Left),
                Rect::new(0, 0, (title.len() + 2) as u16, 3),
            );
        })?;
        Ok(())
    }

    fn draw_cpu_visualizer(
        &mut self,
        terminal: &mut VisTerminal,
    ) -> Result<(), Box<dyn Error>> {
        let registers_rect_width = 40;
        let instructions_rect_width = 40;

        terminal.draw(|frame| {
            self.last_drawn_tick_count = self.cpu.tick_count;
            let frame_rect = frame.size();
            //
            // col 0                    1         2           3  main_rect_height
            //     |--------------------|---------|-----------|  -
            //     | zero page          | instr   | registers |  |  - main_rect_inner_height
            //     |                    | uctions |           |  |  |
            //     |                    |         |           |  |  |
            //     |--------------------|         |           |  |  |
            //     | stack              |         |           |  |  |
            //     |                    |         |           |  |  |
            //     |                    |         |           |  |  -
            //     |--------------------|---------|-----------|  -
            let col0 = 0;
            let col3 = frame_rect.width;
            let col2 = col3 - registers_rect_width;
            let col1 = col2 - instructions_rect_width;

            let main_rect_height = frame_rect.height;
            let main_rect_inner_height = main_rect_height - 2;

            let ram_rect_width =
                frame_rect.width - registers_rect_width - instructions_rect_width;
            let ram_rect_inner_width = ram_rect_width - 2;
            let ram_rect = Rect::new(col0, 0, ram_rect_width, main_rect_height);

            let instructions_rect =
                Rect::new(col1, 0, instructions_rect_width, main_rect_height);

            let registers_rect =
                Rect::new(col2, 0, registers_rect_width, main_rect_height);

            let block = Block::default()
                .style(Style::default().bg(Color::Black).fg(Color::White));
            frame.render_widget(block, frame_rect);

            let zero_page_text = get_ram_page_text(
                &self.cpu,
                0,
                ram_rect_inner_width,
                main_rect_inner_height,
            );
            let zero_page_rect = {
                let mut rect = ram_rect;
                rect.height = zero_page_text.len() as u16 + 2;
                rect
            };

            // Zero Page RAM
            frame.render_widget(
                Paragraph::new(zero_page_text)
                    .block(create_block("Zero Page RAM"))
                    .alignment(Alignment::Left),
                zero_page_rect,
            );

            let stack_page_text = get_ram_page_text(
                &self.cpu,
                0x01,
                ram_rect_inner_width,
                main_rect_inner_height,
            );
            let stack_page_rect = {
                let mut rect = ram_rect;
                rect.y = zero_page_rect.height;
                rect.height = stack_page_text.len() as u16 + 2;
                rect
            };
            // Stack Page RAM
            frame.render_widget(
                Paragraph::new(stack_page_text)
                    .block(create_block("Stack Page RAM"))
                    .alignment(Alignment::Left),
                stack_page_rect,
            );

            // Add the pages of memory.
            for (i, page) in self.pages.iter().enumerate() {
                let text = get_ram_page_text(
                    &self.cpu,
                    *page,
                    ram_rect_inner_width,
                    main_rect_inner_height,
                );
                let rect = {
                    let mut rect = ram_rect;
                    rect.y = zero_page_rect.height * (i as u16 + 2);
                    rect.height = text.len() as u16 + 2;
                    rect
                };
                // Only render it if there is enough space.
                if rect.y + rect.height < frame_rect.height {
                    frame.render_widget(
                        Paragraph::new(text)
                            .block(create_block(&format!("Page ${:02x}", page)))
                            .alignment(Alignment::Left),
                        rect,
                    );
                }
            }

            // Instructions.
            frame.render_widget(
                Paragraph::new(get_instructions_text(
                    &self.cpu,
                    main_rect_inner_height,
                    &mut self.executed_instructions,
                    &self.address_to_label,
                ))
                .block(create_block("Instructions"))
                .alignment(Alignment::Left),
                instructions_rect,
            );

            // Registeres
            let registers_text = vec![
                add_tick_count(self.cpu.tick_count),
                add_register_span("A", self.cpu.a),
                add_register_span("X", self.cpu.x),
                add_register_span("Y", self.cpu.y),
                add_pc_register_span(self.cpu.pc),
                add_register_span("SP", self.cpu.s),
                add_register_span("P", self.cpu.p),
                add_status_register_info("NV__DIZC"),
                add_status_register_info("||  ||||"),
                add_status_register_info("||  |||+- Carry"),
                add_status_register_info("||  ||+-- Zero"),
                add_status_register_info("||  |+--- Interrupt Disable"),
                add_status_register_info("||  +---- Decimal"),
                add_status_register_info("|+-------- Overflow"),
                add_status_register_info("+--------- Negative"),
            ];

            frame.render_widget(
                Paragraph::new(registers_text)
                    .block(create_block("CPU Registers"))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true }),
                registers_rect,
            );
        })?;
        Ok(())
    }

    fn process_events(&mut self) -> Result<(), Box<dyn Error>> {
        // Handle all of the keyboard events.
        if let Event::Input(key) = self.events.next()? {
            match self.mode {
                VisMode::Visualizer => match key {
                    Key::Char('a') => {
                        log("Go to add page memory");
                        self.add_page_address = "0x".into();
                        self.mode = VisMode::AddPageMemory;
                    }
                    Key::Char('r') => {
                        log("Remove a page of memory");
                        self.pages.pop();
                        self.draw_is_dirty = true;
                    }
                    Key::Char('q') => {
                        log("Quit");
                        self.mode = VisMode::Quit;
                    }
                    Key::Char('h') | Key::Char('?') => {
                        log("Go to help");
                        self.mode = VisMode::Help;
                    }
                    Key::Char('n') | Key::Char('1') => {
                        log(&format!("Next instruction ${:x}", self.cpu.pc));
                        if !self.cpu.tick() {
                            log("CPU instructions ended, quitting.");
                            self.mode = VisMode::Quit;
                        }
                    }
                    // Skip through instructions much quicker.
                    Key::Char(c) => {
                        if let Some(n) = c.to_digit(10) {
                            if n != 0 {
                                let count = (n + 1).pow(2);
                                log(&format!("Next {} instructions", count));
                                for _ in 0..count {
                                    if !self.cpu.tick() {
                                        log("CPU instructions ended, quitting.");
                                        self.mode = VisMode::Quit;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                VisMode::Help => match key {
                    Key::Char('q') => {
                        log("Quit");
                        self.mode = VisMode::Quit;
                    }
                    _ => {
                        log("Go to visualizer from Help");
                        self.mode = VisMode::Visualizer;
                    }
                },
                VisMode::AddPageMemory => match key {
                    Key::Char('\n') => {
                        if self.add_page_address.len() == 2 {
                            log("No page address entered");
                            self.add_page_address.clear();
                            self.mode = VisMode::Visualizer;
                        } else {
                            log(&format!("Add page memory ${}", &self.add_page_address));
                            let page = u8::from_str_radix(
                                &self.add_page_address[2..self.add_page_address.len()],
                                16,
                            )
                            .expect("Unable to parse hex string");
                            log(&format!("Page parsed ${:x}", page));
                            if page < 0x20 {
                                self.pages.push(page);
                            } else {
                                log(&format!("Page is not in range ${:x}", page));
                            }
                            self.add_page_address.clear();
                            self.mode = VisMode::Visualizer;
                        }
                    }
                    Key::Backspace => {
                        if self.add_page_address.len() > 2 {
                            self.add_page_address.pop();
                            self.draw_is_dirty = true;
                            log(&format!(
                                "Page address changed ${}",
                                self.add_page_address
                            ));
                        }
                    }
                    Key::Char('q') | Key::Esc => {
                        log("Go back to visualizer");
                        self.add_page_address.clear();
                        self.mode = VisMode::Visualizer;
                    }
                    Key::Char(c) => {
                        let is_hex_digit =
                            (c >= 'a' && c <= 'f') || (c >= '0' && c <= '9');
                        if is_hex_digit && self.add_page_address.len() < 4 {
                            self.add_page_address.push(c);
                            self.draw_is_dirty = true;
                            log(&format!(
                                "Page address changed ${}",
                                self.add_page_address
                            ));
                        }
                    }
                    _ => {}
                },
                VisMode::Quit => {}
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(move |x| {
        stdout()
            .into_raw_mode()
            .unwrap()
            .suspend_raw_mode()
            .unwrap();
        write!(
            stdout().into_raw_mode().unwrap(),
            "{}",
            termion::screen::ToMainScreen
        )
        .unwrap();
        write!(stdout(), "{:?}", x).unwrap();
    }));

    init_log();
    let mut visualizer = Visualizer::new()?;
    visualizer.run()
}

fn add_register_span(name: &str, value: u8) -> Spans {
    let mut parts = vec![];
    if name.len() == 1 {
        parts.push(Span::styled("·", Style::default().fg(Color::Black)));
    }
    parts.push(Span::styled(
        name,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    parts.push(Span::styled(": 0x", Style::default().fg(Color::DarkGray)));
    parts.push(Span::styled(
        format!("{:02x}", value),
        Style::default().fg(Color::White),
    ));
    parts.push(Span::styled(" 0b", Style::default().fg(Color::DarkGray)));
    parts.push(Span::styled(
        format!("{:08b}", value),
        Style::default().fg(Color::White),
    ));

    Spans::from(parts)
}

fn add_pc_register_span(value: u16) -> Spans<'static> {
    let mut parts = vec![];
    parts.push(Span::styled(
        "PC",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    parts.push(Span::styled(": 0x", Style::default().fg(Color::DarkGray)));
    parts.push(Span::styled(
        format!("{:04x}", value),
        Style::default().fg(Color::White),
    ));

    Spans::from(parts)
}

fn add_tick_count(count: u64) -> Spans<'static> {
    let mut parts = vec![];
    parts.push(Span::styled(
        "Ticks: ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    parts.push(Span::styled(
        count.to_string(),
        Style::default().fg(Color::White),
    ));

    Spans::from(parts)
}

fn add_status_register_info(info: &str) -> Spans {
    let mut parts = vec![];
    parts.push(Span::styled(
        "·          ",
        Style::default().fg(Color::Black),
    ));
    parts.push(Span::styled(info, Style::default().fg(Color::DarkGray)));
    Spans::from(parts)
}

fn get_instructions_text<'a>(
    cpu: &'a Cpu6502,
    height: u16,
    executed_instructions: &'a mut VecDeque<Spans<'static>>,
    address_to_label: &AddressToLabel,
) -> Vec<Spans<'a>> {
    let mut spans_list: Vec<Spans> = vec![];
    let bus = cpu.bus.borrow();
    let mut pc = cpu.pc;

    // Make sure the VecDeque is sized correctly to the available of back buffer.
    let executed_len = height / 3;
    executed_instructions.truncate(executed_len as usize);

    let next_instructructions_len =
        height - executed_instructions.len() as u16 + height % 3;

    for spans in executed_instructions.iter().rev() {
        spans_list.push(spans.clone());
    }

    for i in 0..next_instructructions_len {
        let mut parts = vec![];

        let base_style = {
            if i == 0 {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        };

        // label:
        // ^^^^^^
        //   $4027 clc
        if let Some(pc_label) = address_to_label.get(&pc) {
            let mut span =
                Span::styled(format!("{}: ", pc_label), base_style.fg(MAGENTA));

            // Is this selected?
            if i == 0 {
                // Remember this for in the list of executed instructions.
                let mut dim_span = span.clone();
                dim_span.style = base_style.fg(GRAY);
                executed_instructions.push_front(Spans::from(dim_span));

                // Bold the current label too.
                span.style = span.style.clone().add_modifier(Modifier::BOLD);
            }

            spans_list.push(Spans::from(span));
        };

        let instruction_pc = pc;

        // label:
        //   $4027 clc
        //   ^^^^^
        parts.push(Span::styled(
            format!("  ${:02x} ", pc.clone()),
            base_style.fg(CYAN),
        ));

        let operation = bus.read_u8(pc);
        pc = pc.wrapping_add(1);

        let opcode = OPCODE_STRING_TABLE[operation as usize];
        let mode = ADDRESSING_MODE_TABLE[operation as usize];
        parts.push(Span::styled(opcode, base_style.fg(Color::Yellow)));

        // let get_u16 = || {
        //     let value = bus.read_u8(pc);
        //     pc += 1
        // };
        let mut get_u8 = || {
            let value = bus.read_u8(pc);
            pc += 1;
            value
        };
        let mut add_operand = |string| {
            parts.push(Span::styled(string, base_style.fg(Color::White)));
        };

        match mode {
            Mode::Absolute
            | Mode::AbsoluteIndexedX
            | Mode::AbsoluteIndexedY
            | Mode::Indirect => {
                let a = bus.read_u8(pc);
                let b = bus.read_u8(pc + 1);
                pc += 2;
                let value = u16::from_le_bytes([a, b]);

                let mut address_style = base_style.fg(Color::White);

                //   $4023 jmp section2 $4029
                //             ^^^^^^^^
                if let Some(label) = address_to_label.get(&value) {
                    parts.push(Span::styled(
                        format!(" {}", label),
                        base_style.fg(MAGENTA),
                    ));
                    // Dim out the address.
                    address_style = base_style.fg(GRAY);
                };

                if mode == Mode::Indirect {
                    //   $4023 jmp ($4029)
                    //             ^
                    parts.push(Span::styled("(", base_style.fg(Color::White)));
                }

                //   $4023 jmp section2 $4029
                //                      ^^^^^
                //   $4023 jmp $4029
                //             ^^^^^
                parts.push(Span::styled(format!(" ${:04x}\n", value), address_style));

                // Handle indexed modes.
                if mode == Mode::AbsoluteIndexedX {
                    //   $4023 jmp $4029,X
                    //                  ^^
                    parts.push(Span::styled(",X", base_style.fg(Color::White)));
                }
                if mode == Mode::AbsoluteIndexedX {
                    //   $4023 jmp $4029,Y
                    //                  ^^
                    parts.push(Span::styled(",Y", base_style.fg(Color::White)));
                }

                if mode == Mode::Indirect {
                    //   $4023 jmp ($4029)
                    //                   ^
                    parts.push(Span::styled(")", base_style.fg(Color::White)));
                }
            }

            // u8 operands:
            Mode::Immediate => add_operand(format!(" #${:02x}\n", get_u8())),
            Mode::ZeroPage => add_operand(format!(" ${:02x}\n", get_u8())),
            Mode::ZeroPageX => add_operand(format!(" ${:02x},X\n", get_u8())),
            Mode::ZeroPageY => add_operand(format!(" ${:02x},Y\n", get_u8())),
            Mode::IndirectX => add_operand(format!(" (${:02x},X)\n", get_u8())),
            Mode::IndirectY => add_operand(format!(" (${:02x}),Y\n", get_u8())),

            Mode::Relative => {
                let relative_value = get_u8() as i8;
                let address: u16 = (instruction_pc as i32 + relative_value as i32) as u16;

                match address_to_label.get(&address) {
                    Some(label) => {
                        parts.push(Span::styled(
                            format!(" {}", label),
                            base_style.fg(MAGENTA),
                        ));
                        // Dim out the address.
                        parts.push(Span::styled(
                            format!(" {:+}\n", relative_value),
                            base_style.fg(GRAY),
                        ))
                    }
                    None => add_operand(format!(" {:+}\n", get_u8() as i8)),
                }
            }

            Mode::Implied | Mode::None | Mode::RegisterA => {}
        }

        if i == 0 {
            let mut span_dimmed = parts.clone();
            for span in span_dimmed.iter_mut() {
                span.style = base_style.fg(GRAY);
            }
            // Remember this instruction for the next tick.
            executed_instructions.push_front(Spans::from(span_dimmed));
        }

        spans_list.push(Spans::from(parts));
    }

    spans_list
}

fn get_ram_page_text(
    cpu: &Cpu6502,
    page_u8: u8,
    width: u16,
    _height: u16,
) -> Vec<Spans<'static>> {
    let mut spans = vec![];
    let bus = cpu.bus.borrow();
    let style = Style::default();
    let cyan = style.fg(CYAN);
    let dim_white = style.fg(DIM_WHITE);

    // Decide how many columns to make.
    let col_width = "$0000 0011 2233 4455 6677 8899 aabb ccdd eeff ".len();
    let cols = (width / col_width as u16).max(1);

    // Compute the page view.
    // e.g.
    // $0000 0011 2233 4455 6677 8899 aabb ccdd eeff
    // $0010 0011 2233 4455 6677 8899 aabb ccdd eeff
    // $0020 0011 2233 4455 6677 8899 aabb ccdd eeff
    // ..... .... .... .... .... .... .... .... ....
    // $00F0 0011 2233 4455 6677 8899 aabb ccdd eeff

    spans.push(Spans::from(Span::styled(
        "       0 1  2 3  4 5  6 7  8 9  a b  c d  e f ".repeat(cols as usize),
        //     0011 2233 4455 6677 8899 aabb ccdd eeff
        style.fg(MAGENTA),
    )));

    let mut parts = vec![];
    let page_u16: u16 = (page_u8 as u16) << 8;
    let stack_address = cpu.s as u16 + 0x100;
    for i in 0..16 {
        // $00 0011 2233 4455 6677 8899 aabb ccdd eeff
        // ^^^
        parts.push(Span::styled(format!("${:02x}{:x}_ ", page_u8, i), cyan));
        for j in 0..8 {
            let address = page_u16 + i * 16 + j * 2;
            let [le, be] = bus.read_u16(address).to_le_bytes();
            // $0000 0011 2233 4455 6677 8899 aabb ccdd eeff
            //       ^^^^
            let color = if j % 2 == 0 {
                style.fg(Color::White)
            } else {
                dim_white
            };
            parts.push(Span::styled(
                format!("{:02x}", le),
                if address == stack_address + 1 as u16 {
                    style.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    color
                },
            ));
            parts.push(Span::styled(
                format!("{:02x} ", be),
                if address == stack_address {
                    style.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    color
                },
            ));
        }

        if (i + 1) % cols == 0 {
            spans.push(Spans::from(parts.clone()));
            parts.clear();
        }
    }
    if !parts.is_empty() {
        spans.push(Spans::from(parts.clone()));
    }

    spans
}

fn create_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(BORDER_COLOR))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}
