#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use nes::{
    asm::AsmLexer,
    bus::Bus,
    cpu_6502::Cpu6502,
    opcodes::{mode_to_operand_count, Mode, OpCode, ADDRESSING_MODE_TABLE, OPCODE_STRING_TABLE},
};
use std::{collections::VecDeque, error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

const BORDER_COLOR: Color = Color::Rgb(150, 150, 150);

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let registers_rect_width = 40;
    let instructions_rect_width = 40;
    let mut cpu = load_cpu();
    let mut last_drawn_tick_count = u64::MAX;
    let mut executed_instructions = VecDeque::new();

    loop {
        if last_drawn_tick_count != cpu.tick_count {
            // Only draw again if the cpu tick has changed.
            terminal.draw(|frame| {
                last_drawn_tick_count = cpu.tick_count;
                let frame_rect = frame.size();
                //
                // col 0                    1         2           3  main_rect_height
                //     |--------------------|---------|-----------|  -
                //     | ram                | instr   | registers |  |  - main_rect_inner_height
                //     |                    | uctions |           |  |  |
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
                let registers_rect = Rect::new(col2, 0, registers_rect_width, main_rect_height);

                let block =
                    Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
                frame.render_widget(block, frame_rect);

                let create_block = |title| {
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black).fg(BORDER_COLOR))
                        .title(Span::styled(
                            title,
                            Style::default().add_modifier(Modifier::BOLD),
                        ))
                };

                let ram_text = get_ram_text(&cpu, ram_rect_inner_width, main_rect_inner_height);
                let zero_page_rect = {
                    let mut rect = ram_rect.clone();
                    rect.height = ram_text.len() as u16 + 2;
                    rect
                };

                // RAM
                frame.render_widget(
                    Paragraph::new(ram_text)
                        .block(create_block("Zero Page RAM"))
                        .alignment(Alignment::Left),
                    zero_page_rect,
                );

                // Instructions.
                frame.render_widget(
                    Paragraph::new(get_instructions_text(
                        &cpu,
                        main_rect_inner_height,
                        &mut executed_instructions,
                    ))
                    .block(create_block("Instructions"))
                    .alignment(Alignment::Left),
                    instructions_rect,
                );

                // Registeres
                let registers_text = vec![
                    add_tick_count(cpu.tick_count),
                    add_register_span("A", cpu.a),
                    add_register_span("X", cpu.x),
                    add_register_span("Y", cpu.y),
                    add_pc_register_span(cpu.pc),
                    add_register_span("SP", cpu.s),
                    add_register_span("P", cpu.p),
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
        }

        // Handle all of the keyboard events.
        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => {
                    break;
                }
                Key::Char('n') => {
                    if !cpu.tick() {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
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
) -> Vec<Spans<'a>> {
    let mut spans_list: Vec<Spans> = vec![];
    let bus = cpu.bus.borrow();
    let mut pc = cpu.pc;

    // Make sure the VecDeque is sized correctly to the available of back buffer.
    let executed_len = height / 3;
    executed_instructions.truncate(executed_len as usize);

    let next_instructructions_len = height - executed_instructions.len() as u16 + height % 3;

    for spans in executed_instructions.iter().rev() {
        spans_list.push(spans.clone());
    }

    // A little utility to make it easy to read from the bus and increment the
    // program counter.
    let mut read = || {
        let value = bus.read_u8(pc);
        pc += 1;
        value
    };

    for i in 0..next_instructructions_len {
        let mut parts = vec![];
        let operation = read();
        let opcode = OPCODE_STRING_TABLE[operation as usize];
        let mode = ADDRESSING_MODE_TABLE[operation as usize];
        let base_style = {
            if i == 0 {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        };
        parts.push(Span::styled(opcode, base_style.fg(Color::Yellow)));

        let operand_count = mode_to_operand_count(mode);

        // Unroll the loop into 2 if statements.
        if operand_count == 1 {
            parts.push(Span::styled(
                format!(" {:02x}", read()),
                base_style.fg(Color::White),
            ));
        } else if operand_count == 2 {
            parts.push(Span::styled(
                format!(" {:02x} {:02x}\n", read(), read()),
                base_style.fg(Color::White),
            ));
        }

        if mode != Mode::None {
            parts.push(Span::styled(
                format!(" {:?}", mode),
                base_style.fg(Color::Yellow),
            ));
        }

        if i == 0 {
            let mut span_dimmed = parts.clone();
            for span in span_dimmed.iter_mut() {
                span.style = base_style.fg(Color::Rgb(170, 170, 170));
            }
            // Remember this instruction for the next tick.
            executed_instructions.push_front(Spans::from(span_dimmed));
        }

        spans_list.push(Spans::from(parts));
    }

    spans_list
}

fn get_ram_text(cpu: &Cpu6502, width: u16, _height: u16) -> Vec<Spans<'static>> {
    let mut spans = vec![];
    let bus = cpu.bus.borrow();
    let style = Style::default();
    let cyan = style.fg(Color::Rgb(0, 200, 200));
    let dim_white = style.fg(Color::Rgb(200, 200, 200));

    // Decide how many columns to make.
    let col_width = "$00 0011 2233 4455 6677 8899 aabb ccdd eeff ".len();
    let cols = (width / col_width as u16).max(1);

    // Compute the zero page view.
    // e.g.
    // $0000 0011 2233 4455 6677 8899 aabb ccdd eeff
    // $0010 0011 2233 4455 6677 8899 aabb ccdd eeff
    // $0020 0011 2233 4455 6677 8899 aabb ccdd eeff
    // ..... .... .... .... .... .... .... .... ....
    // $00F0 0011 2233 4455 6677 8899 aabb ccdd eeff

    spans.push(Spans::from(Span::styled(
        "    0011 2233 4455 6677 8899 aabb ccdd eeff ".repeat(cols as usize),
        //   0011 2233 4455 6677 8899 aabb ccdd eeff
        style.fg(Color::Rgb(200, 100, 200)),
    )));

    let mut parts = vec![];
    for i in 0..16 {
        // $00 0011 2233 4455 6677 8899 aabb ccdd eeff
        // ^^^
        parts.push(Span::styled(format!("${:x}0 ", i), cyan));
        for j in 0..8 {
            // $0000 0011 2233 4455 6677 8899 aabb ccdd eeff
            //       ^^^^
            parts.push(Span::styled(
                format!("{:04x} ", bus.read_u16(i * 8 + j * 2)),
                {
                    if j % 2 == 0 {
                        style.fg(Color::White)
                    } else {
                        dim_white
                    }
                },
            ));
        }

        if (i + 1) % cols == 0 {
            spans.push(Spans::from(parts.clone()));
            parts.clear();
        }
    }
    if parts.len() > 0 {
        spans.push(Spans::from(parts.clone()));
    }

    spans
}

fn load_cpu() -> Cpu6502 {
    let mut lexer = AsmLexer::new(
        "
            sec        ; Set the carry flag
            lda #$11   ; Load A with a value
            adc #$22   ; This should add all three values
                       ; = 0x01 + 0x11 + 0x22 = 0x34
            clc        ; Clear the carry bit

            adc #$01   ; Add to the A register
            inx        ; Increase the value of X in the register
            sta $00,x  ;

            adc #$01   ; Add to the A register
            inx        ; Increase the value of X in the register
            sta $00,x  ;

            adc #$01   ; Add to the A register
            inx        ; Increase the value of X in the register
            sta $00,x  ;

            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
            clc        ;
        ",
    );

    match lexer.parse() {
        Ok(_) => {
            let mut program = lexer.to_bytes().unwrap();
            program.push(OpCode::KIL as u8);
            Cpu6502::new({
                let bus = Bus::new_shared_bus();
                bus.borrow_mut().load_program(&program);
                bus
            })
        }
        Err(parse_error) => {
            parse_error.panic_nicely();
            panic!("");
        }
    }
}
